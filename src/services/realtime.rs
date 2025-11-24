use crate::client::{BosBaseInner, USER_AGENT};
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex as StdMutex};
use std::thread;
use std::time::{Duration, Instant};
use std::panic::AssertUnwindSafe;
use urlencoding::encode;

type Callback = Arc<dyn Fn(Value) + Send + Sync>;

#[derive(Clone)]
pub struct RealtimeService {
    inner: Arc<RealtimeInner>,
}

struct RealtimeInner {
    client: Arc<BosBaseInner>,
    client_id: Mutex<String>,
    subscriptions: Mutex<HashMap<String, Vec<RealtimeListener>>>,
    stop: AtomicBool,
    ready: (StdMutex<bool>, Condvar),
    handle: Mutex<Option<thread::JoinHandle<()>>>,
    counter: AtomicU64,
}

#[derive(Clone)]
struct RealtimeListener {
    id: String,
    callback: Callback,
}

impl RealtimeService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            inner: Arc::new(RealtimeInner {
                client,
                client_id: Mutex::new(String::new()),
                subscriptions: Mutex::new(HashMap::new()),
                stop: AtomicBool::new(false),
                ready: (StdMutex::new(false), Condvar::new()),
                handle: Mutex::new(None),
                counter: AtomicU64::new(0),
            }),
        }
    }

    pub fn subscribe<F>(
        &self,
        topic: &str,
        callback: F,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<impl FnOnce(), ClientResponseError>
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        if topic.is_empty() {
            return Err(ClientResponseError::new(
                String::new(),
                400,
                json!({"message": "topic must be set"}),
                false,
                None,
            ));
        }
        let key = self.build_subscription_key(topic, &query, &headers);
        let listener_id = format!("l-{}", self.inner.counter.fetch_add(1, Ordering::SeqCst) + 1);
        {
            let mut subs = self.inner.subscriptions.lock();
            subs.entry(key.clone())
                .or_default()
                .push(RealtimeListener {
                    id: listener_id.clone(),
                    callback: Arc::new(callback),
                });
        }
        self.ensure_thread();
        self.ensure_connected(Duration::from_secs(10))?;
        self.submit_subscriptions();
        let topic_string = topic.to_string();
        let svc = self.clone();
        Ok(move || {
            let _ = svc.unsubscribe_by_topic_and_id(&topic_string, &listener_id);
        })
    }

    pub fn unsubscribe(&self, topic: Option<String>) {
        if let Some(topic) = topic {
            let mut subs = self.inner.subscriptions.lock();
            subs.retain(|key, _| key != &topic && !key.starts_with(&(topic.clone() + "?")));
            if subs.is_empty() {
                self.disconnect();
            } else {
                self.submit_subscriptions();
            }
        } else {
            self.inner.subscriptions.lock().clear();
            self.disconnect();
        }
    }

    pub fn unsubscribe_by_prefix(&self, prefix: &str) {
        let mut subs = self.inner.subscriptions.lock();
        subs.retain(|key, _| !key.starts_with(prefix));
        if subs.is_empty() {
            self.disconnect();
        } else {
            self.submit_subscriptions();
        }
    }

    fn unsubscribe_by_topic_and_id(&self, topic: &str, id: &str) -> bool {
        let mut subs = self.inner.subscriptions.lock();
        let mut changed = false;
        for key in subs.clone().keys() {
            if key != topic && !key.starts_with(&(topic.to_string() + "?")) {
                continue;
            }
            if let Some(listeners) = subs.get_mut(key) {
                listeners.retain(|l| l.id != id);
                changed = true;
                if listeners.is_empty() {
                    subs.remove(key);
                }
            }
        }
        if changed {
            if subs.is_empty() {
                self.disconnect();
            } else {
                self.submit_subscriptions();
            }
        }
        changed
    }

    pub fn disconnect(&self) {
        self.inner.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.inner.handle.lock().take() {
            let _ = handle.join();
        }
        *self.inner.client_id.lock() = String::new();
        let (lock, cvar) = &self.inner.ready;
        *lock.lock().unwrap() = false;
        cvar.notify_all();
    }

    fn ensure_thread(&self) {
        let mut handle = self.inner.handle.lock();
        if handle.is_some() {
            return;
        }
        self.inner.stop.store(false, Ordering::SeqCst);
        let inner = self.inner.clone();
        *handle = Some(thread::spawn(move || run_loop(inner)));
    }

    fn ensure_connected(&self, timeout: Duration) -> Result<(), ClientResponseError> {
        let (lock, cvar) = &self.inner.ready;
        let mut ready = lock.lock().unwrap();
        if *ready {
            return Ok(());
        }
        let start = Instant::now();
        while !*ready {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(ClientResponseError::new(
                    String::new(),
                    0,
                    json!({"message": "Realtime connection not established"}),
                    true,
                    None,
                ));
            }
            let remaining = timeout - elapsed;
            let (new_ready, _) = cvar
                .wait_timeout(ready, remaining)
                .expect("condvar poisoned");
            ready = new_ready;
        }
        Ok(())
    }

    fn submit_subscriptions(&self) {
        let client_id = self.inner.client_id.lock().clone();
        let subs: Vec<String> = {
            let subs = self.inner.subscriptions.lock();
            subs.keys().cloned().collect()
        };
        if client_id.is_empty() || subs.is_empty() {
            return;
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = json!({
            "clientId": client_id,
            "subscriptions": subs,
        });
        let _ = self.inner.client.send("/api/realtime", opts);
    }

    fn build_subscription_key(
        &self,
        topic: &str,
        query: &HashMap<String, Value>,
        headers: &HashMap<String, String>,
    ) -> String {
        let mut key = topic.to_string();
        let mut options = serde_json::Map::new();
        if !query.is_empty() {
            let mut qmap = serde_json::Map::new();
            for (k, v) in query.iter() {
                qmap.insert(k.clone(), v.clone());
            }
            options.insert("query".into(), Value::Object(qmap));
        }
        if !headers.is_empty() {
            let mut header_obj = serde_json::Map::new();
            for (k, v) in headers {
                header_obj.insert(k.clone(), Value::String(v.clone()));
            }
            options.insert("headers".into(), Value::Object(header_obj));
        }
        if !options.is_empty() {
            let serialized = serde_json::to_string(&Value::Object(options)).unwrap_or_default();
            let suffix = format!("options={}", encode(&serialized));
            if key.contains('?') {
                key.push('&');
                key.push_str(&suffix);
            } else {
                key.push('?');
                key.push_str(&suffix);
            }
        }
        key
    }
}

fn run_loop(inner: Arc<RealtimeInner>) {
    let backoff = [
        Duration::from_millis(200),
        Duration::from_millis(500),
        Duration::from_millis(1000),
        Duration::from_millis(2000),
        Duration::from_millis(5000),
    ];
    let mut attempt = 0usize;
    let base_url = inner.client.build_url("/api/realtime", &HashMap::new());

    while !inner.stop.load(Ordering::SeqCst) {
        let mut req = inner
            .client
            .http
            .get(&base_url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-store")
            .header("Accept-Language", inner.client.lang.clone())
            .header("User-Agent", USER_AGENT);
        if inner.client.auth_store.is_valid() {
            req = req.header("Authorization", inner.client.auth_store.token());
        }

        match req.send() {
            Ok(resp) if resp.status().is_success() => {
                attempt = 0;
                listen(inner.clone(), resp);
            }
            _ => {
                handle_disconnect(&inner);
                let delay = backoff[std::cmp::min(attempt, backoff.len() - 1)];
                attempt += 1;
                thread::sleep(delay);
            }
        }

        if inner.stop.load(Ordering::SeqCst) {
            break;
        }
        handle_disconnect(&inner);
        if inner.subscriptions.lock().is_empty() {
            break;
        }
    }
}

fn listen(inner: Arc<RealtimeInner>, resp: reqwest::blocking::Response) {
    let mut reader = BufReader::new(resp);
    let mut event = Event::default();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        if inner.stop.load(Ordering::SeqCst) {
            return;
        }
        let line = line.trim_end_matches(&['\r', '\n'][..]).to_string();
        if line.is_empty() {
            dispatch_event(inner.clone(), &event);
            event = Event::default();
            continue;
        }
        if line.starts_with(':') {
            continue;
        }
        if let Some((field, value)) = line.split_once(':') {
            let value = value.trim_start();
            match field {
                "event" => event.event = if value.is_empty() { "message".into() } else { value.into() },
                "data" => {
                    event.data.push_str(value);
                    event.data.push('\n');
                }
                "id" => event.id = value.into(),
                _ => {}
            }
        }
    }
}

#[derive(Default)]
struct Event {
    event: String,
    data: String,
    id: String,
}

fn dispatch_event(inner: Arc<RealtimeInner>, evt: &Event) {
    let name = if evt.event.is_empty() {
        "message"
    } else {
        evt.event.as_str()
    };
    let mut payload = Value::Object(serde_json::Map::new());
    if !evt.data.trim().is_empty() {
        if let Ok(val) = serde_json::from_str::<Value>(evt.data.trim_end_matches('\n')) {
            payload = val;
        }
    }

    if name == "PB_CONNECT" {
        let client_id_val = payload
            .get("clientId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| if evt.id.is_empty() { None } else { Some(evt.id.clone()) });
        if let Some(client_id) = client_id_val {
            *inner.client_id.lock() = client_id;
            let (lock, cvar) = &inner.ready;
            let mut ready = lock.lock().unwrap();
            *ready = true;
            cvar.notify_all();
        }
        submit_subscriptions_inner(&inner);
        return;
    }

    let listeners = {
        let subs = inner.subscriptions.lock();
        subs.get(name).cloned().unwrap_or_default()
    };
    for listener in listeners {
        let cb = listener.callback.clone();
        let payload_clone = payload.clone();
        let _ = std::panic::catch_unwind(AssertUnwindSafe(move || (cb)(payload_clone)));
    }
}

fn submit_subscriptions_inner(inner: &Arc<RealtimeInner>) {
    let client_id = inner.client_id.lock().clone();
    if client_id.is_empty() {
        return;
    }
    let subs: Vec<String> = {
        let subs = inner.subscriptions.lock();
        subs.keys().cloned().collect()
    };
    if subs.is_empty() {
        return;
    }
    let mut opts = SendOptions::default();
    opts.method = "POST".into();
    opts.body = json!({ "clientId": client_id, "subscriptions": subs });
    let _ = inner.client.send("/api/realtime", opts);
}

fn handle_disconnect(inner: &Arc<RealtimeInner>) {
    *inner.client_id.lock() = String::new();
    let (lock, cvar) = &inner.ready;
    let mut ready = lock.lock().unwrap();
    *ready = false;
    cvar.notify_all();
}

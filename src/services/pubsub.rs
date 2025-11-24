use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::services::BaseService;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::panic::AssertUnwindSafe;
use tungstenite::{connect, Message};

#[derive(Clone, Debug)]
pub struct PubSubMessage {
    pub id: String,
    pub topic: String,
    pub created: String,
    pub data: Value,
}

#[derive(Clone)]
pub struct PubSubService {
    base: BaseService,
    subscriptions: Arc<Mutex<HashMap<String, Vec<Arc<dyn Fn(PubSubMessage) + Send + Sync>>>>>,
    ready: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    sender: Arc<Mutex<Option<Sender<String>>>>,
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl PubSubService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            ready: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            sender: Arc::new(Mutex::new(None)),
            handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn publish(
        &self,
        topic: &str,
        data: Value,
    ) -> Result<PubSubMessage, ClientResponseError> {
        if topic.is_empty() {
            return Err(ClientResponseError::new(
                String::new(),
                400,
                json!({"message": "topic must be set"}),
                false,
                None,
            ));
        }
        self.ensure_socket()?;
        let payload = json!({
            "type": "publish",
            "topic": topic,
            "data": data.clone(),
        });
        self.send_envelope(payload);
        Ok(PubSubMessage {
            id: String::new(),
            topic: topic.to_string(),
            created: String::new(),
            data,
        })
    }

    pub fn subscribe<F>(&self, topic: &str, callback: F) -> Result<impl FnOnce(), ClientResponseError>
    where
        F: Fn(PubSubMessage) + Send + Sync + 'static,
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
        let mut first_listener = false;
        {
            let mut subs = self.subscriptions.lock();
            let listeners = subs.entry(topic.to_string()).or_default();
            if listeners.is_empty() {
                first_listener = true;
            }
            listeners.push(Arc::new(callback));
        }
        self.ensure_socket()?;
        if first_listener {
            let payload = json!({
                "type": "subscribe",
                "topic": topic,
            });
            self.send_envelope(payload);
        }

        let topic_owned = topic.to_string();
        let svc = self.clone();
        Ok(move || {
            let mut should_unsub = false;
            {
                let mut subs = svc.subscriptions.lock();
                if let Some(listeners) = subs.get_mut(&topic_owned) {
                    if !listeners.is_empty() {
                        listeners.pop();
                    }
                    if listeners.is_empty() {
                        subs.remove(&topic_owned);
                        should_unsub = true;
                    }
                }
            }
            if should_unsub {
                svc.send_envelope(json!({"type": "unsubscribe", "topic": topic_owned}));
            }
            if svc.subscriptions.lock().is_empty() {
                svc.disconnect();
            }
        })
    }

    pub fn unsubscribe(&self, topic: Option<String>) {
        let mut subs = self.subscriptions.lock();
        if let Some(topic) = topic {
            subs.remove(&topic);
            let payload = json!({
                "type": "unsubscribe",
                "topic": topic,
            });
            self.send_envelope(payload);
        } else {
            subs.clear();
            self.send_envelope(json!({"type": "unsubscribe"}));
        }
        if subs.is_empty() {
            self.disconnect();
        }
    }

    pub fn disconnect(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.lock().take() {
            let _ = handle.join();
        }
        self.ready.store(false, Ordering::SeqCst);
    }

    fn ensure_socket(&self) -> Result<(), ClientResponseError> {
        if self.ready.load(Ordering::SeqCst) {
            return Ok(());
        }
        let mut handle = self.handle.lock();
        if handle.is_some() {
            return Ok(());
        }
        self.stop.store(false, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel::<String>();
        *self.sender.lock() = Some(tx);
        let inner = PubSubThreadState {
            client: self.base.client.clone(),
            subscriptions: self.subscriptions.clone(),
            ready: self.ready.clone(),
            stop: self.stop.clone(),
            receiver: rx,
        };
        *handle = Some(thread::spawn(move || socket_loop(inner)));
        Ok(())
    }

    fn send_envelope(&self, payload: Value) {
        if let Some(sender) = self.sender.lock().clone() {
            let _ = sender.send(payload.to_string());
        }
    }
}

struct PubSubThreadState {
    client: Arc<BosBaseInner>,
    subscriptions: Arc<Mutex<HashMap<String, Vec<Arc<dyn Fn(PubSubMessage) + Send + Sync>>>>>,
    ready: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    receiver: mpsc::Receiver<String>,
}

fn socket_loop(state: PubSubThreadState) {
    loop {
        if state.stop.load(Ordering::SeqCst) {
            break;
        }
        let url = build_ws_url(&state.client);
        match connect(url) {
            Ok((mut socket, _)) => {
                state.ready.store(true, Ordering::SeqCst);
                socket
                    .send(Message::Text(json!({"type": "hello"}).to_string()))
                    .ok();
                loop {
                    if state.stop.load(Ordering::SeqCst) {
                        let _ = socket.close(None);
                        break;
                    }
                    // send pending messages
                    while let Ok(msg) = state.receiver.try_recv() {
                        let _ = socket.send(Message::Text(msg));
                    }
                    match socket.read() {
                        Ok(Message::Text(text)) => handle_message(&state, &text),
                        Ok(_) => {}
                        Err(_) => break,
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                state.ready.store(false, Ordering::SeqCst);
            }
            Err(_) => {
                state.ready.store(false, Ordering::SeqCst);
            }
        }
        if state.stop.load(Ordering::SeqCst) || state.subscriptions.lock().is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(300));
    }
    state.ready.store(false, Ordering::SeqCst);
}

fn build_ws_url(client: &Arc<BosBaseInner>) -> String {
    let mut query = HashMap::new();
    if client.auth_store.is_valid() {
        query.insert("token".to_string(), json!(client.auth_store.token()));
    }
    let mut url = client.build_url("/api/pubsub", &query);
    if url.starts_with("https://") {
        url = url.replacen("https://", "wss://", 1);
    } else if url.starts_with("http://") {
        url = url.replacen("http://", "ws://", 1);
    } else {
        url = format!("ws://{}", url);
    }
    url
}

fn handle_message(state: &PubSubThreadState, payload: &str) {
    let parsed: Value = serde_json::from_str(payload).unwrap_or_else(|_| json!({}));
    let topic = parsed
        .get("topic")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let data = parsed.get("data").cloned().unwrap_or(Value::Null);
    let msg = PubSubMessage {
        id: parsed
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        topic: topic.clone(),
        created: parsed
            .get("created")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        data,
    };
    let listeners = {
        let subs = state.subscriptions.lock();
        subs.get(&topic).cloned().unwrap_or_default()
    };
    for cb in listeners {
        let msg_clone = msg.clone();
        let _ = std::panic::catch_unwind(AssertUnwindSafe(move || (cb)(msg_clone)));
    }
}

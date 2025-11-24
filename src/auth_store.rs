use crate::utils::base64_url_decode;
use chrono::Utc;
use parking_lot::Mutex;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::panic::AssertUnwindSafe;

type Listener = Arc<dyn Fn(String, Value) + Send + Sync>;

#[derive(Default)]
pub struct AuthStore {
    token: Mutex<String>,
    record: Mutex<Value>,
    listeners: Mutex<Vec<(usize, Listener)>>,
    next_id: AtomicUsize,
}

impl AuthStore {
    pub fn token(&self) -> String {
        self.token.lock().clone()
    }

    pub fn record(&self) -> Value {
        self.record.lock().clone()
    }

    pub fn is_valid(&self) -> bool {
        let token = self.token.lock();
        if token.is_empty() {
            return false;
        }
        Self::is_jwt_valid(&token)
    }

    pub fn save(&self, token: impl Into<String>, record: Value) {
        let token = token.into();
        let mut callbacks = Vec::new();
        {
            let mut t = self.token.lock();
            let mut r = self.record.lock();
            *t = token.clone();
            *r = record.clone();
            callbacks.extend(self.listeners.lock().iter().cloned());
        }
        for (_, cb) in callbacks {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| cb(token.clone(), record.clone())));
        }
    }

    pub fn clear(&self) {
        self.save("", Value::Null);
    }

    pub fn add_listener(&self, listener: Listener) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        self.listeners.lock().push((id, listener));
        id
    }

    pub fn remove_listener(&self, id: usize) {
        let mut listeners = self.listeners.lock();
        listeners.retain(|(lid, _)| *lid != id);
    }

    fn is_jwt_valid(token: &str) -> bool {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        if let Some(decoded) = base64_url_decode(parts[1]) {
            if let Ok(json) = serde_json::from_slice::<Value>(&decoded) {
                if let Some(exp) = json.get("exp") {
                    if let Some(exp_num) = exp.as_i64() {
                        let now = Utc::now().timestamp();
                        return exp_num > now;
                    }
                }
            }
        }
        false
    }
}

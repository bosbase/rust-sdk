use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::utils::encode_path_segment;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct CacheService {
    base: BaseService,
}

impl CacheService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn list(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        let data = self.base.client.send("/api/cache", opts)?;
        if data.is_object() {
            if let Some(items) = data.get("items") {
                return Ok(items.clone());
            }
        }
        Ok(data)
    }

    pub fn create(
        &self,
        name: &str,
        size_bytes: Option<i32>,
        default_ttl_seconds: Option<i32>,
        read_timeout_ms: Option<i32>,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = body;
        payload["name"] = json!(name);
        if let Some(size) = size_bytes {
            payload["sizeBytes"] = json!(size);
        }
        if let Some(ttl) = default_ttl_seconds {
            payload["defaultTTLSeconds"] = json!(ttl);
        }
        if let Some(timeout) = read_timeout_ms {
            payload["readTimeoutMs"] = json!(timeout);
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/cache", opts)
    }

    pub fn update(
        &self,
        name: &str,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("/api/cache/{}", encode_path_segment(name)), opts)
    }

    pub fn remove(
        &self,
        name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("/api/cache/{}", encode_path_segment(name)), opts)
            .map(|_| ())
    }

    pub fn set_entry(
        &self,
        cache: &str,
        key: &str,
        value: Value,
        ttl_seconds: Option<i32>,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        body["value"] = value;
        if let Some(ttl) = ttl_seconds {
            body["ttlSeconds"] = json!(ttl);
        }
        let mut opts = SendOptions::default();
        opts.method = "PUT".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "/api/cache/{}/entries/{}",
                encode_path_segment(cache),
                encode_path_segment(key)
            ),
            opts,
        )
    }

    pub fn get_entry(
        &self,
        cache: &str,
        key: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "/api/cache/{}/entries/{}",
                encode_path_segment(cache),
                encode_path_segment(key)
            ),
            opts,
        )
    }

    pub fn renew_entry(
        &self,
        cache: &str,
        key: &str,
        ttl_seconds: Option<i32>,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(ttl) = ttl_seconds {
            body["ttlSeconds"] = json!(ttl);
        }
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "/api/cache/{}/entries/{}",
                encode_path_segment(cache),
                encode_path_segment(key)
            ),
            opts,
        )
    }

    pub fn delete_entry(
        &self,
        cache: &str,
        key: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!(
                    "/api/cache/{}/entries/{}",
                    encode_path_segment(cache),
                    encode_path_segment(key)
                ),
                opts,
            )
            .map(|_| ())
    }
}

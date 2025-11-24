use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::types::{VectorBatchInsertOptions, VectorCollectionConfig, VectorDocument, VectorSearchOptions};
use crate::utils::encode_path_segment;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct VectorService {
    base: BaseService,
}

impl VectorService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn create_collection(
        &self,
        name: &str,
        config: Option<VectorCollectionConfig>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = serde_json::Map::new();
        payload.insert("name".into(), Value::String(name.to_string()));
        if let Some(cfg) = config {
            payload.insert("config".into(), cfg.to_json());
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = Value::Object(payload);
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send("/api/vector/collections", opts)
    }

    pub fn list_collections(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/vector/collections", opts)
    }

    pub fn delete_collection(
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
            .send(
                &format!(
                    "/api/vector/collections/{}",
                    encode_path_segment(name)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn insert(
        &self,
        document: VectorDocument,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = document.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/vector/documents", opts)
    }

    pub fn batch_insert(
        &self,
        options: VectorBatchInsertOptions,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = options.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send("/api/vector/documents/batch", opts)
    }

    pub fn get(
        &self,
        id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("/api/vector/documents/{}", encode_path_segment(id)),
            opts,
        )
    }

    pub fn update(
        &self,
        id: &str,
        document: VectorDocument,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = document.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("/api/vector/documents/{}", encode_path_segment(id)),
            opts,
        )
    }

    pub fn remove(
        &self,
        id: &str,
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
                &format!("/api/vector/documents/{}", encode_path_segment(id)),
                opts,
            )
            .map(|_| ())
    }

    pub fn list(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/vector/documents", opts)
    }

    pub fn search(
        &self,
        options: VectorSearchOptions,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = options.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/vector/search", opts)
    }
}

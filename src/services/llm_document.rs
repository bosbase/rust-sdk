use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::types::{LLMDocument, LLMDocumentUpdate, LLMQueryOptions};
use crate::utils::encode_path_segment;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct LLMDocumentService {
    base: BaseService,
}

impl LLMDocumentService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn list_collections(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        let data = self
            .base
            .client
            .send("/api/llm-documents/collections", opts)?;
        if data.is_array() {
            Ok(data)
        } else {
            Ok(Value::Array(vec![data]))
        }
    }

    pub fn create_collection(
        &self,
        name: &str,
        metadata: Option<HashMap<String, String>>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut payload = json!({});
        if let Some(meta) = metadata {
            payload["metadata"] = json!(meta);
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!(
                    "/api/llm-documents/collections/{}",
                    encode_path_segment(name)
                ),
                opts,
            )
            .map(|_| ())
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
                    "/api/llm-documents/collections/{}",
                    encode_path_segment(name)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn insert(
        &self,
        collection: &str,
        document: LLMDocument,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = document.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("/api/llm-documents/{}", encode_path_segment(collection)),
            opts,
        )
    }

    pub fn get(
        &self,
        collection: &str,
        document_id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<LLMDocument, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        let data = self.base.client.send(
            &format!(
                "/api/llm-documents/{}/{}",
                encode_path_segment(collection),
                encode_path_segment(document_id)
            ),
            opts,
        )?;
        let doc: LLMDocument = serde_json::from_value(data.clone())
            .unwrap_or_else(|_| LLMDocument {
                id: document_id.to_string(),
                content: String::new(),
                metadata: None,
            });
        Ok(doc)
    }

    pub fn update(
        &self,
        collection: &str,
        document_id: &str,
        document: LLMDocumentUpdate,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = document.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "/api/llm-documents/{}/{}",
                encode_path_segment(collection),
                encode_path_segment(document_id)
            ),
            opts,
        )
    }

    pub fn remove(
        &self,
        collection: &str,
        document_id: &str,
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
                    "/api/llm-documents/{}/{}",
                    encode_path_segment(collection),
                    encode_path_segment(document_id)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn list(
        &self,
        collection: &str,
        page: Option<i32>,
        per_page: Option<i32>,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(p) = page {
            query.insert("page".into(), json!(p));
        }
        if let Some(pp) = per_page {
            query.insert("perPage".into(), json!(pp));
        }
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("/api/llm-documents/{}", encode_path_segment(collection)),
            opts,
        )
    }

    pub fn query(
        &self,
        collection: &str,
        options: LLMQueryOptions,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = options.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "/api/llm-documents/{}/documents/query",
                encode_path_segment(collection)
            ),
            opts,
        )
    }
}

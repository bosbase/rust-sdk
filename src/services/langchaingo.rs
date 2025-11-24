use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::types::{LangChaingoCompletionRequest, LangChaingoRAGRequest};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct LangChaingoService {
    base: BaseService,
}

impl LangChaingoService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn completions(
        &self,
        payload: LangChaingoCompletionRequest,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send("/api/langchaingo/completions", opts)
    }

    pub fn rag(
        &self,
        payload: LangChaingoRAGRequest,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/langchaingo/rag", opts)
    }

    pub fn query_documents(
        &self,
        payload: LangChaingoRAGRequest,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload.to_json();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send("/api/langchaingo/documents/query", opts)
    }

    pub fn sql(
        &self,
        payload: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/langchaingo/sql", opts)
    }
}

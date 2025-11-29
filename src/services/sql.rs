use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct SQLService {
    base: BaseService,
}

impl SQLService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn execute(
        &self,
        query: &str,
        query_params: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(ClientResponseError::new(
                self.base
                    .client
                    .build_url("/api/sql/execute", &HashMap::new()),
                400,
                json!({"message": "query is required"}),
                false,
                None,
            ));
        }

        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = json!({ "query": trimmed });
        opts.query = query_params;
        opts.headers = headers;

        self.base.client.send("/api/sql/execute", opts)
    }
}

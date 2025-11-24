use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct GraphQLService {
    base: BaseService,
}

impl GraphQLService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn send_query(
        &self,
        query_string: &str,
        variables: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".into(), Value::String(query_string.to_string()));
        if !(variables.is_null() || variables.as_object().map_or(false, |o| o.is_empty())) {
            payload.insert("variables".into(), variables);
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = Value::Object(payload);
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/graphql", opts)
    }
}

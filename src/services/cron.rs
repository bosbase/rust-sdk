use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::utils::encode_path_segment;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct CronService {
    base: BaseService,
}

impl CronService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn get_full_list(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/crons", opts)
    }

    pub fn run(
        &self,
        job_id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("/api/crons/{}", encode_path_segment(job_id)), opts)
    }
}

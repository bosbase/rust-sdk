use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::{FileAttachment, SendOptions};
use crate::services::BaseService;
use crate::utils::encode_path_segment;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct BackupService {
    base: BaseService,
}

impl BackupService {
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
        self.base.client.send("/api/backups", opts)
    }

    pub fn create(
        &self,
        basename: Option<String>,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(name) = basename {
            query.insert("basename".into(), json!(name));
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/backups", opts)
    }

    pub fn upload(
        &self,
        file: FileAttachment,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.files = vec![file];
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/backups/upload", opts)
    }

    pub fn remove(
        &self,
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
                &format!("/api/backups/{}", encode_path_segment(key)),
                opts,
            )
            .map(|_| ())
    }

    pub fn restore(
        &self,
        key: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("/api/backups/{}/restore", encode_path_segment(key)),
            opts,
        )
    }

    pub fn get_download_url(&self, token: &str, key: &str) -> String {
        let mut query = HashMap::new();
        query.insert("token".into(), json!(token));
        self.base
            .client
            .build_url(&format!("/api/backups/{}", encode_path_segment(key)), &query)
    }
}

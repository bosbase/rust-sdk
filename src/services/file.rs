use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use crate::utils::encode_path_segment;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileService {
    base: BaseService,
}

impl FileService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn get_url(
        &self,
        record: Value,
        filename: String,
        thumb: Option<String>,
        token: Option<String>,
        download: Option<bool>,
        mut query: HashMap<String, Value>,
    ) -> String {
        if let Some(thumb) = thumb {
            query.insert("thumb".into(), json!(thumb));
        }
        if let Some(token) = token {
            query.insert("token".into(), json!(token));
        }
        if let Some(download) = download {
            query.insert("download".into(), json!(download));
        }

        let record_id = record
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let mut collection = record
            .get("collectionName")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if collection.is_empty() {
            if let Some(name) = record.get("@collectionName").and_then(|v| v.as_str()) {
                collection = name.to_string();
            }
        }
        if collection.is_empty() {
            if let Some(cid) = record.get("collectionId").and_then(|v| v.as_str()) {
                collection = cid.to_string();
            }
        }

        let path = format!(
            "/api/files/{}/{}/{}",
            encode_path_segment(&collection),
            encode_path_segment(&record_id),
            encode_path_segment(&filename)
        );
        self.base.client.build_url(&path, &query)
    }

    pub fn get_token(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/files/token", opts)
    }
}

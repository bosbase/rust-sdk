use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::{FileAttachment, SendOptions};
use crate::utils::{build_relative_url, encode_path_segment};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Debug)]
struct QueuedBatchRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Value,
    files: Vec<FileAttachment>,
}

#[derive(Clone)]
pub struct BatchService {
    client: Arc<BosBaseInner>,
    requests: Arc<Mutex<Vec<QueuedBatchRequest>>>,
}

impl BatchService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            client,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn collection(&self, collection: impl Into<String>) -> SubBatchService {
        SubBatchService {
            batch: self.clone(),
            collection: collection.into(),
        }
    }

    pub fn queue_request(
        &self,
        method: impl Into<String>,
        url: impl Into<String>,
        headers: HashMap<String, String>,
        body: Value,
        files: Vec<FileAttachment>,
    ) {
        let mut reqs = self.requests.lock();
        reqs.push(QueuedBatchRequest {
            method: method.into(),
            url: url.into(),
            headers,
            body: if body.is_null() { json!({}) } else { body },
            files,
        });
    }

    pub fn send(
        &self,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = if body.is_null() { json!({}) } else { body };
        payload["requests"] = json!([]);
        let mut attachments = Vec::new();

        let mut reqs = self.requests.lock();
        for (idx, req) in reqs.iter().enumerate() {
            let item = json!({
                "method": req.method,
                "url": req.url,
                "headers": req.headers,
                "body": req.body,
            });
            payload["requests"].as_array_mut().unwrap().push(item);

            for file in req.files.iter() {
                let mut copy = file.clone();
                copy.field = format!("requests.{}.{}", idx, copy.field);
                attachments.push(copy);
            }
        }

        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        opts.files = attachments;
        let response = self.client.send("/api/batch", opts);
        reqs.clear();
        response
    }
}

#[derive(Clone)]
pub struct SubBatchService {
    batch: BatchService,
    collection: String,
}

impl SubBatchService {
    fn collection_url(&self) -> String {
        format!(
            "/api/collections/{}/records",
            encode_path_segment(&self.collection)
        )
    }

    pub fn create(
        &self,
        body: Value,
        mut query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        expand: Option<String>,
        fields: Option<String>,
    ) {
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let url = build_relative_url(&self.collection_url(), &query);
        self.batch
            .queue_request("POST", url, HashMap::new(), body, files);
    }

    pub fn upsert(
        &self,
        body: Value,
        mut query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        expand: Option<String>,
        fields: Option<String>,
    ) {
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let url = build_relative_url(&self.collection_url(), &query);
        self.batch
            .queue_request("PUT", url, HashMap::new(), body, files);
    }

    pub fn update(
        &self,
        record_id: &str,
        body: Value,
        mut query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        expand: Option<String>,
        fields: Option<String>,
    ) {
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let url = build_relative_url(
            &format!("{}/{}", self.collection_url(), encode_path_segment(record_id)),
            &query,
        );
        self.batch
            .queue_request("PATCH", url, HashMap::new(), body, files);
    }

    pub fn remove(&self, record_id: &str, body: Value, query: HashMap<String, Value>) {
        let url = build_relative_url(
            &format!("{}/{}", self.collection_url(), encode_path_segment(record_id)),
            &query,
        );
        self.batch
            .queue_request("DELETE", url, HashMap::new(), body, Vec::new());
    }
}

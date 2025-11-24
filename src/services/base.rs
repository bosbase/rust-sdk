use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::{FileAttachment, SendOptions};
use crate::utils::encode_path_segment;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct BaseService {
    pub(crate) client: Arc<BosBaseInner>,
}

impl BaseService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self { client }
    }
}

#[derive(Clone)]
pub struct BaseCrudService {
    pub(crate) client: Arc<BosBaseInner>,
    crud_path: String,
}

impl BaseCrudService {
    pub(crate) fn new(client: Arc<BosBaseInner>, crud_path: impl Into<String>) -> Self {
        Self {
            client,
            crud_path: crud_path.into(),
        }
    }

    pub fn base_crud_path(&self) -> &str {
        &self.crud_path
    }

    pub fn get_list(
        &self,
        page: i32,
        per_page: i32,
        skip_total: bool,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        filter: Option<String>,
        sort: Option<String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        let mut params = query;
        params.insert("page".into(), json!(page));
        params.insert("perPage".into(), json!(per_page));
        params.insert("skipTotal".into(), json!(skip_total));
        if let Some(filter) = filter {
            params.insert("filter".into(), json!(filter));
        }
        if let Some(sort) = sort {
            params.insert("sort".into(), json!(sort));
        }
        if let Some(expand) = expand {
            params.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            params.insert("fields".into(), json!(fields));
        }
        let mut opts = SendOptions::default();
        opts.query = params;
        opts.headers = headers;
        self.client.send(self.base_crud_path(), opts)
    }

    pub fn get_one(
        &self,
        record_id: &str,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        if record_id.is_empty() {
            return Err(ClientResponseError::new(
                self.client.build_url(&format!("{}/", self.base_crud_path()), &query),
                404,
                json!({
                    "code": 404,
                    "message": "Missing required record id.",
                    "data": {}
                }),
                false,
                None,
            ));
        }
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.client.send(
            &format!("{}/{}", self.base_crud_path(), encode_path_segment(record_id)),
            opts,
        )
    }

    pub fn get_first_list_item(
        &self,
        filter: String,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        let data = self.get_list(
            1,
            1,
            true,
            query,
            headers,
            Some(filter),
            None,
            expand,
            fields,
        )?;
        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
            if let Some(first) = items.first() {
                return Ok(first.clone());
            }
        }
        Err(ClientResponseError::new(
            String::new(),
            404,
            json!({
                "code": 404,
                "message": "The requested resource wasn't found.",
                "data": {}
            }),
            false,
            None,
        ))
    }

    pub fn get_full_list(
        &self,
        batch: i32,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        filter: Option<String>,
        sort: Option<String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        if batch <= 0 {
            return Err(ClientResponseError::new(
                String::new(),
                400,
                json!({ "message": "batch must be > 0" }),
                false,
                None,
            ));
        }
        let mut result = Vec::new();
        let mut page = 1;
        loop {
            let data = self.get_list(
                page,
                batch,
                true,
                query.clone(),
                headers.clone(),
                filter.clone(),
                sort.clone(),
                expand.clone(),
                fields.clone(),
            )?;
            let items = data
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let per_page = data
                .get("perPage")
                .and_then(|v| v.as_i64())
                .unwrap_or(batch as i64);
            for item in items.iter() {
                result.push(item.clone());
            }
            if items.is_empty() || (items.len() as i64) < per_page {
                break;
            }
            page += 1;
        }
        Ok(Value::Array(result))
    }

    pub fn create(
        &self,
        body: Value,
        mut query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        opts.files = files;
        self.client.send(self.base_crud_path(), opts)
    }

    pub fn update(
        &self,
        record_id: &str,
        body: Value,
        mut query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        opts.files = files;
        self.client.send(
            &format!("{}/{}", self.base_crud_path(), encode_path_segment(record_id)),
            opts,
        )
    }

    pub fn remove(
        &self,
        record_id: &str,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.client
            .send(
                &format!("{}/{}", self.base_crud_path(), encode_path_segment(record_id)),
                opts,
            )
            .map(|_| ())
    }
}

use crate::auth_store::AuthStore;
use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::{FileAttachment, SendOptions};
use crate::services::{BaseCrudService, RealtimeService};
use crate::utils::{base64_url_decode, encode_path_segment};
use crate::BosBase;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct RecordService {
    base: BaseCrudService,
    collection: String,
    base_collection_path: String,
    realtime: RealtimeService,
}

impl RecordService {
    pub(crate) fn new(
        client: Arc<BosBaseInner>,
        collection: String,
        realtime: RealtimeService,
    ) -> Self {
        let base_collection_path =
            format!("/api/collections/{}", encode_path_segment(&collection));
        let crud_path = format!("{}/records", base_collection_path);
        Self {
            base: BaseCrudService::new(client, crud_path),
            collection,
            base_collection_path,
            realtime,
        }
    }

    pub fn base_collection_path(&self) -> &str {
        &self.base_collection_path
    }

    // realtime
    pub fn subscribe<F>(
        &self,
        topic: &str,
        callback: F,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<impl FnOnce(), ClientResponseError>
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        if topic.is_empty() {
            return Err(ClientResponseError::new(
                String::new(),
                400,
                json!({"message": "topic must be set"}),
                false,
                None,
            ));
        }
        let full_topic = format!("{}/{}", self.collection, topic);
        self.realtime.subscribe(&full_topic, callback, query, headers)
    }

    pub fn unsubscribe(&self, topic: Option<String>) {
        if let Some(topic) = topic {
            self.realtime
                .unsubscribe(Some(format!("{}/{}", self.collection, topic)));
        } else {
            self.realtime.unsubscribe_by_prefix(&self.collection);
        }
    }

    // helpers
    pub fn get_count(
        &self,
        filter: Option<String>,
        expand: Option<String>,
        fields: Option<String>,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<i64, ClientResponseError> {
        if let Some(filter) = filter {
            query.insert("filter".into(), json!(filter));
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
        let data = self.base.client.send(
            &format!("{}/count", self.base.base_crud_path()),
            opts,
        )?;
        Ok(data.get("count").and_then(|v| v.as_i64()).unwrap_or(0))
    }

    pub fn list_auth_methods(
        &self,
        fields: Option<String>,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        query.insert(
            "fields".into(),
            json!(fields.unwrap_or_else(|| "mfa,otp,password,oauth2".into())),
        );
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("{}/auth-methods", self.base_collection_path()), opts)
    }

    pub fn auth_with_password(
        &self,
        identity: &str,
        password: &str,
        expand: Option<String>,
        fields: Option<String>,
        mut body: Value,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        body["identity"] = json!(identity);
        body["password"] = json!(password);
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
        let res = self.base.client.send(
            &format!("{}/auth-with-password", self.base_collection_path()),
            opts,
        )?;
        Ok(self.handle_auth_response(res))
    }

    pub fn auth_with_otp(
        &self,
        otp_id: &str,
        password: &str,
        expand: Option<String>,
        fields: Option<String>,
        mut body: Value,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        body["otpId"] = json!(otp_id);
        body["password"] = json!(password);
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
        let res = self.base.client.send(
            &format!("{}/auth-with-otp", self.base_collection_path()),
            opts,
        )?;
        Ok(self.handle_auth_response(res))
    }

    pub fn auth_with_oauth2_code(
        &self,
        provider: &str,
        code: &str,
        code_verifier: &str,
        redirect_url: &str,
        create_data: Option<Value>,
        mut body: Value,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        body["provider"] = json!(provider);
        body["code"] = json!(code);
        body["codeVerifier"] = json!(code_verifier);
        body["redirectURL"] = json!(redirect_url);
        if let Some(create) = create_data {
            body["createData"] = create;
        }
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
        let res = self.base.client.send(
            &format!("{}/auth-with-oauth2", self.base_collection_path()),
            opts,
        )?;
        Ok(self.handle_auth_response(res))
    }

    pub fn auth_refresh(
        &self,
        body: Value,
        mut query: HashMap<String, Value>,
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
        let res = self.base.client.send(
            &format!("{}/auth-refresh", self.base_collection_path()),
            opts,
        )?;
        Ok(self.handle_auth_response(res))
    }

    pub fn request_password_reset(
        &self,
        email: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["email"] = json!(email);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/request-password-reset", self.base_collection_path()),
                opts,
            )
            .map(|_| ())
    }

    pub fn confirm_password_reset(
        &self,
        token: &str,
        password: &str,
        password_confirm: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["token"] = json!(token);
        body["password"] = json!(password);
        body["passwordConfirm"] = json!(password_confirm);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/confirm-password-reset", self.base_collection_path()),
                opts,
            )
            .map(|_| ())
    }

    pub fn request_verification(
        &self,
        email: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["email"] = json!(email);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/request-verification", self.base_collection_path()),
                opts,
            )
            .map(|_| ())
    }

    pub fn confirm_verification(
        &self,
        token: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["token"] = json!(token);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/confirm-verification", self.base_collection_path()),
                opts,
            )?;
        self.mark_verified(token);
        Ok(())
    }

    pub fn request_email_change(
        &self,
        new_email: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["newEmail"] = json!(new_email);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/request-email-change", self.base_collection_path()),
                opts,
            )
            .map(|_| ())
    }

    pub fn confirm_email_change(
        &self,
        token: &str,
        password: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        body["token"] = json!(token);
        body["password"] = json!(password);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!("{}/confirm-email-change", self.base_collection_path()),
                opts,
            )?;
        self.clear_if_same_token(token);
        Ok(())
    }

    pub fn request_otp(
        &self,
        email: &str,
        mut body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        body["email"] = json!(email);
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!("{}/request-otp", self.base_collection_path()),
            opts,
        )
    }

    pub fn list_external_auths(
        &self,
        record_id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/records/{}/external-auths",
                self.base_collection_path(),
                encode_path_segment(record_id)
            ),
            opts,
        )
    }

    pub fn unlink_external_auth(
        &self,
        record_id: &str,
        provider: &str,
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
                    "{}/records/{}/external-auths/{}",
                    self.base_collection_path(),
                    encode_path_segment(record_id),
                    encode_path_segment(provider)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn impersonate(
        &self,
        record_id: &str,
        duration: i32,
        mut body: Value,
        mut query: HashMap<String, Value>,
        mut headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<BosBase, ClientResponseError> {
        body["duration"] = json!(duration);
        if let Some(expand) = expand {
            query.insert("expand".into(), json!(expand));
        }
        if let Some(fields) = fields {
            query.insert("fields".into(), json!(fields));
        }

        if !headers.contains_key("Authorization") && self.base.client.auth_store.is_valid() {
            headers.insert(
                "Authorization".into(),
                self.base.client.auth_store.token(),
            );
        }

        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;

        let new_client = BosBase::with_options(
            self.base.client.base_url.clone(),
            Some(self.base.client.lang.clone()),
            Some(Arc::new(AuthStore::default())),
            Some(self.base.client.timeout),
        );
        let res = new_client.inner.send(
            &format!(
                "{}/impersonate/{}",
                self.base_collection_path(),
                encode_path_segment(record_id)
            ),
            opts,
        )?;
        if let Some(token) = res.get("token").and_then(|v| v.as_str()) {
            let record = res
                .get("record")
                .cloned()
                .unwrap_or_else(|| json!({}));
            new_client.auth_store().save(token.to_string(), record);
        }
        Ok(new_client)
    }

    // CRUD overrides to sync auth record
    pub fn update(
        &self,
        record_id: &str,
        body: Value,
        query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        let item = self
            .base
            .update(record_id, body, query, files, headers, expand, fields)?;
        self.maybe_update_auth_record(item.clone());
        Ok(item)
    }

    pub fn delete(
        &self,
        record_id: &str,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        self.base.remove(record_id, body, query, headers)?;
        if self.is_auth_record(record_id) {
            self.base.client.auth_store.clear();
        }
        Ok(())
    }

    // Delegated CRUD helpers
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
        self.base
            .get_list(page, per_page, skip_total, query, headers, filter, sort, expand, fields)
    }

    pub fn get_one(
        &self,
        record_id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base.get_one(record_id, query, headers, expand, fields)
    }

    pub fn get_first_list_item(
        &self,
        filter: String,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .get_first_list_item(filter, query, headers, expand, fields)
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
        self.base
            .get_full_list(batch, query, headers, filter, sort, expand, fields)
    }

    pub fn create(
        &self,
        body: Value,
        query: HashMap<String, Value>,
        files: Vec<FileAttachment>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .create(body, query, files, headers, expand, fields)
    }

    // helpers
    fn handle_auth_response(&self, data: Value) -> Value {
        if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
            if let Some(record) = data.get("record") {
                self.base
                    .client
                    .auth_store
                    .save(token.to_string(), record.clone());
            }
        }
        data
    }

    fn maybe_update_auth_record(&self, item: Value) {
        let current = self.base.client.auth_store.record();
        if current.is_null() {
            return;
        }
        let id = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if id != item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
        {
            return;
        }
        let collection_id = current
            .get("collectionId")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let collection_name = current
            .get("collectionName")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if collection_id != self.collection && collection_name != self.collection {
            return;
        }

        let mut merged = current.clone();
        if let Some(map) = item.as_object() {
            for (k, v) in map {
                merged[k] = v.clone();
            }
        }
        if let (Some(cur_expand), Some(item_expand)) =
            (current.get("expand"), item.get("expand"))
        {
            let mut combined = cur_expand.clone();
            if let (Some(cur_map), Some(item_map)) =
                (cur_expand.as_object(), item_expand.as_object())
            {
                let mut merged_map = cur_map.clone();
                for (k, v) in item_map {
                    merged_map.insert(k.clone(), v.clone());
                }
                combined = Value::Object(merged_map);
            }
            merged["expand"] = combined;
        }
        let token = self.base.client.auth_store.token();
        self.base.client.auth_store.save(token, merged);
    }

    fn is_auth_record(&self, record_id: &str) -> bool {
        let current = self.base.client.auth_store.record();
        if current.is_null() {
            return false;
        }
        let same_id = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            == record_id;
        let same_collection = {
            let cid = current
                .get("collectionId")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let cname = current
                .get("collectionName")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            cid == self.collection || cname == self.collection
        };
        same_id && same_collection
    }

    fn mark_verified(&self, token: &str) {
        let current = self.base.client.auth_store.record();
        if current.is_null() {
            return;
        }
        let payload = self.decode_token_payload(token);
        if payload.is_null() {
            return;
        }
        let same_id = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            == payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
        let same_collection = current
            .get("collectionId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            == payload
                .get("collectionId")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
        if same_id && same_collection && !current.get("verified").and_then(|v| v.as_bool()).unwrap_or(false) {
            let mut updated = current.clone();
            updated["verified"] = Value::Bool(true);
            let token = self.base.client.auth_store.token();
            self.base.client.auth_store.save(token, updated);
        }
    }

    fn clear_if_same_token(&self, token: &str) {
        let current = self.base.client.auth_store.record();
        if current.is_null() {
            return;
        }
        let payload = self.decode_token_payload(token);
        if payload.is_null() {
            return;
        }
        let same_id = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            == payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
        let same_collection = current
            .get("collectionId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            == payload
                .get("collectionId")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
        if same_id && same_collection {
            self.base.client.auth_store.clear();
        }
    }

    fn decode_token_payload(&self, token: &str) -> Value {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() < 2 {
            return Value::Null;
        }
        if let Some(decoded) = base64_url_decode(parts[1]) {
            serde_json::from_slice::<Value>(&decoded).unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    }
}

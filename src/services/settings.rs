use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct SettingsService {
    base: BaseService,
}

impl SettingsService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseService::new(client),
        }
    }

    pub fn get_all(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/settings", opts)
    }

    pub fn update(
        &self,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = body;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/settings", opts)
    }

    pub fn test_s3(
        &self,
        filesystem: Option<String>,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        if let Some(fs) = filesystem {
            query.insert("filesystem".into(), json!(fs));
        }
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/settings/test/s3", opts)
    }

    pub fn test_email(
        &self,
        collection: &str,
        to_email: &str,
        template_name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let payload = json!({
            "collectionIdOrName": collection,
            "toEmail": to_email,
            "template": template_name
        });
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send("/api/settings/test/email", opts)
    }

    pub fn generate_apple_client_secret(
        &self,
        client_id: &str,
        team_id: &str,
        key_id: &str,
        private_key: &str,
        duration: i32,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let payload = json!({
            "clientId": client_id,
            "teamId": team_id,
            "keyId": key_id,
            "privateKey": private_key,
            "duration": duration
        });
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send("/api/settings/apple/generate-client-secret", opts)
    }
}

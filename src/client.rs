use crate::auth_store::AuthStore;
use crate::errors::ClientResponseError;
use crate::request::{AfterSendHook, BeforeSendHook, SendOptions};
use crate::services::{
    BackupService, BatchService, CacheService, CollectionService, CronService, FileService,
    GraphQLService, HealthService, LangChaingoService, LLMDocumentService, LogService,
    PubSubService, RealtimeService, RecordService, SettingsService, VectorService,
};
use crate::utils::build_relative_url;
use chrono::DateTime;
use parking_lot::Mutex;
use reqwest::blocking::Client as HttpClient;
use reqwest::blocking::multipart::Form;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Method;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub(crate) const USER_AGENT: &str = "bosbase-rust-sdk/0.1.0";

pub(crate) struct BosBaseInner {
    pub base_url: String,
    pub lang: String,
    pub timeout: Duration,
    pub auth_store: Arc<AuthStore>,
    pub before_send: Mutex<Option<BeforeSendHook>>,
    pub after_send: Mutex<Option<AfterSendHook>>,
    pub http: HttpClient,
}

impl BosBaseInner {
    pub fn build_url(&self, path: &str, query: &HashMap<String, Value>) -> String {
        let rel = build_relative_url(path, query);
        let mut base = self.base_url.clone();
        if !base.ends_with('/') {
            base.push('/');
        }
        let rel = rel.trim_start_matches('/');
        format!("{}{}", base, rel)
    }

    pub fn send(&self, path: &str, mut options: SendOptions) -> Result<Value, ClientResponseError> {
        let mut url = self.build_url(path, &options.query);

        if let Some(hook) = &*self.before_send.lock() {
            hook(&mut url, &mut options);
            url = self.build_url(path, &options.query);
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Accept-Language".into(), self.lang.clone());
        headers.insert("User-Agent".into(), USER_AGENT.to_string());
        for (k, v) in options.headers.iter() {
            headers.insert(k.clone(), v.clone());
        }
        if !headers.contains_key("Authorization") && self.auth_store.is_valid() {
            headers.insert("Authorization".into(), self.auth_store.token());
        }

        let method = options
            .method
            .parse::<Method>()
            .unwrap_or_else(|_| Method::GET);
        let timeout = options.timeout.unwrap_or(self.timeout);
        let mut req = self.http.request(method, &url).timeout(timeout);
        for (key, value) in headers.iter() {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                req = req.header(name, val);
            }
        }

        if !options.files.is_empty() {
            let mut form = Form::new();
            if let Some(map) = options.body.as_object() {
                for (key, val) in map {
                    let text = val
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| val.to_string());
                    form = form.text(key.clone(), text);
                }
            }
            for file in options.files.into_iter() {
                let bytes = file.data.clone();
                let mut part =
                    reqwest::blocking::multipart::Part::bytes(bytes.clone()).file_name(file.filename.clone());
                part = match part.mime_str(&file.content_type) {
                    Ok(p) => p,
                    Err(_) => reqwest::blocking::multipart::Part::bytes(bytes).file_name(file.filename),
                };
                form = form.part(file.field, part);
            }
            req = req.multipart(form);
        } else if !options.body.is_null() {
            req = req.json(&options.body);
        }

        let resp = req.send().map_err(|err| {
            ClientResponseError::new(
                url.clone(),
                0,
                json!({ "message": err.to_string() }),
                err.is_timeout(),
                Some(err.to_string()),
            )
        })?;

        let status = resp.status();
        let status_code = status.as_u16();
        let mut headers_out = HashMap::new();
        for (name, value) in resp.headers() {
            headers_out.insert(
                name.to_string(),
                value.to_str().unwrap_or_default().to_string(),
            );
        }
        let bytes = resp.bytes().unwrap_or_default();
        let mut data: Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).to_string()));

        if status.is_client_error() || status.is_server_error() {
            return Err(ClientResponseError::new(
                url,
                status_code,
                data,
                false,
                None,
            ));
        }

        if let Some(after) = &*self.after_send.lock() {
            data = after(status_code, &headers_out, &data);
        }
        Ok(data)
    }

    pub fn filter(&self, expr: &str, params: &HashMap<String, Value>) -> String {
        if params.is_empty() {
            return expr.to_string();
        }
        let mut result = expr.to_string();
        for (key, value) in params {
            let placeholder = format!("{{:{}}}", key);
            let replacement = match value {
                Value::String(s) => {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                        format!("'{}'", dt.format("%Y-%m-%d %H:%M:%S"))
                    } else {
                        format!("'{}'", s.replace('\'', "\\'"))
                    }
                }
                Value::Null => "null".to_string(),
                Value::Bool(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Value::Number(n) => n.to_string(),
                _ => {
                    let serialized = value.to_string().replace('\'', "\\'");
                    format!("'{}'", serialized)
                }
            };
            result = result.replace(&placeholder, &replacement);
        }
        result
    }
}

#[derive(Clone)]
pub struct BosBase {
    pub(crate) inner: Arc<BosBaseInner>,
    pub collections: CollectionService,
    pub files: FileService,
    pub logs: LogService,
    pub realtime: RealtimeService,
    pub pubsub: PubSubService,
    pub settings: SettingsService,
    pub health: HealthService,
    pub backups: BackupService,
    pub crons: CronService,
    pub vectors: VectorService,
    pub langchaingo: LangChaingoService,
    pub llm_documents: LLMDocumentService,
    pub caches: CacheService,
    pub graphql: GraphQLService,
}

impl BosBase {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_options(base_url, None, None, None)
    }

    pub fn with_options(
        base_url: impl Into<String>,
        lang: Option<String>,
        auth_store: Option<Arc<AuthStore>>,
        timeout: Option<Duration>,
    ) -> Self {
        let mut base = base_url.into();
        if base.is_empty() {
            base = "/".to_string();
        } else {
            base = base.trim_end_matches('/').to_string();
        }
        let auth_store = auth_store.unwrap_or_default();
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        let http = HttpClient::builder()
            .timeout(timeout)
            .build()
            .expect("failed to build HTTP client");
        let inner = Arc::new(BosBaseInner {
            base_url: base,
            lang: lang.unwrap_or_else(|| "en-US".to_string()),
            timeout,
            auth_store: auth_store.clone(),
            before_send: Mutex::new(None),
            after_send: Mutex::new(None),
            http,
        });

        let realtime = RealtimeService::new(inner.clone());
        let pubsub = PubSubService::new(inner.clone());

        Self {
            inner: inner.clone(),
            collections: CollectionService::new(inner.clone()),
            files: FileService::new(inner.clone()),
            logs: LogService::new(inner.clone()),
            realtime: realtime.clone(),
            pubsub: pubsub.clone(),
            settings: SettingsService::new(inner.clone()),
            health: HealthService::new(inner.clone()),
            backups: BackupService::new(inner.clone()),
            crons: CronService::new(inner.clone()),
            vectors: VectorService::new(inner.clone()),
            langchaingo: LangChaingoService::new(inner.clone()),
            llm_documents: LLMDocumentService::new(inner.clone()),
            caches: CacheService::new(inner.clone()),
            graphql: GraphQLService::new(inner.clone()),
        }
    }

    pub fn close(&self) {
        self.realtime.disconnect();
        self.pubsub.disconnect();
    }

    pub fn collection(&self, collection: impl Into<String>) -> RecordService {
        RecordService::new(self.inner.clone(), collection.into(), self.realtime.clone())
    }

    pub fn filter(&self, expr: &str, params: HashMap<String, Value>) -> String {
        self.inner.filter(expr, &params)
    }

    pub fn build_url(&self, path: &str, query: &HashMap<String, Value>) -> String {
        self.inner.build_url(path, query)
    }

    pub fn create_batch(&self) -> BatchService {
        BatchService::new(self.inner.clone())
    }

    pub fn get_file_url(
        &self,
        record: Value,
        filename: impl Into<String>,
        thumb: Option<String>,
        token: Option<String>,
        download: Option<bool>,
        query: HashMap<String, Value>,
    ) -> String {
        self.files
            .get_url(record, filename.into(), thumb, token, download, query)
    }

    pub fn auth_store(&self) -> Arc<AuthStore> {
        self.inner.auth_store.clone()
    }

    pub fn set_before_send(&self, hook: Option<BeforeSendHook>) {
        *self.inner.before_send.lock() = hook;
    }

    pub fn set_after_send(&self, hook: Option<AfterSendHook>) {
        *self.inner.after_send.lock() = hook;
    }
}

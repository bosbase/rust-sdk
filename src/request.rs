use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// File part used in multipart requests.
#[derive(Debug, Clone)]
pub struct FileAttachment {
    pub field: String,
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl FileAttachment {
    pub fn new(field: impl Into<String>, filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            field: field.into(),
            filename: filename.into(),
            content_type: "application/octet-stream".into(),
            data,
        }
    }
}

/// Options passed to the HTTP client.
#[derive(Debug, Clone)]
pub struct SendOptions {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, Value>,
    pub body: Value,
    pub files: Vec<FileAttachment>,
    pub timeout: Option<Duration>,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: Value::Null,
            files: Vec::new(),
            timeout: None,
        }
    }
}

pub type BeforeSendHook = Arc<dyn Fn(&mut String, &mut SendOptions) + Send + Sync>;
pub type AfterSendHook =
    Arc<dyn Fn(u16, &HashMap<String, String>, &Value) -> Value + Send + Sync>;

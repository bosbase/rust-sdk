use serde_json::Value;
use thiserror::Error;

/// Normalized HTTP error mirroring the JS SDK shape.
#[derive(Debug, Error, Clone)]
#[error("ClientResponseError(status={status}, url={url}, response={response}, is_abort={is_abort}, original_error={original_error:?})")]
pub struct ClientResponseError {
    pub url: String,
    pub status: u16,
    pub response: Value,
    pub is_abort: bool,
    pub original_error: Option<String>,
}

impl ClientResponseError {
    pub fn new(
        url: impl Into<String>,
        status: u16,
        response: Value,
        is_abort: bool,
        original_error: Option<String>,
    ) -> Self {
        Self {
            url: url.into(),
            status,
            response,
            is_abort,
            original_error,
        }
    }
}

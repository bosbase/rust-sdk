//! BosBase Rust SDK mirroring the JavaScript SDK.
//! 
//! The entrypoint is [`BosBase`]. Construct it with the base URL of your
//! BosBase server and then use the exposed services (`collections`,
//! `files`, `realtime`, `pubsub`, etc.) to interact with the API.

pub mod auth_store;
pub mod client;
pub mod errors;
pub mod request;
pub mod services;
pub mod types;
pub mod utils;

pub use crate::auth_store::AuthStore;
pub use crate::client::BosBase;
pub use crate::errors::ClientResponseError;
pub use crate::request::{AfterSendHook, BeforeSendHook, FileAttachment, SendOptions};
pub use crate::types::*;

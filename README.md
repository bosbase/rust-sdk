BosBase Rust SDK
================

This crate mirrors the JavaScript SDK so Rust applications can talk to the BosBase Go backend with the same API surface.

## Status

- HTTP client with `before_send`/`after_send` hooks, auth store, filter builder, multipart uploads.
- Services: collections, records (auth helpers, OTP/MFA/OAuth2, impersonation), files, backups, batch writes, cache, cron, logs, settings, GraphQL, vectors, LLM documents, LangChaingo, realtime (SSE), and pubsub (websocket).
- Realtime subscriptions and pubsub reconnect automatically and reuse the same auth token when available.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
bosbase = { path = "rust-sdk" }
```

Basic example:

```rust
use bosbase::{BosBase};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pb = BosBase::new("http://127.0.0.1:8090");

    // authenticate against an auth collection
    pb.collection("users")
        .auth_with_password("test@example.com", "123456", None, None, json!({}), Default::default(), Default::default())?;

    // list records
    let posts = pb
        .collection("posts")
        .get_list(1, 10, false, Default::default(), Default::default(), None, None, None, None)?;
    println!("posts: {:?}", posts);

    // create a record
    let created = pb
        .collection("posts")
        .create(json!({"title": "Hello"}), Default::default(), vec![], Default::default(), None, None)?;
    println!("created: {:?}", created);

    Ok(())
}
```

Realtime subscription:

```rust
let unsubscribe = pb
    .collection("posts")
    .subscribe("*", |evt| println!("change: {evt:?}"), Default::default(), Default::default())?;
// ...
unsubscribe();
```

The `docs/` folder from the JS SDK applies to this crate—the endpoint names, query parameters, and helper methods are the same, only the Rust calling conventions differ.

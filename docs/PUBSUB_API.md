# Pub/Sub API - Rust SDK Documentation

## Overview

BosBase exposes a lightweight WebSocket-based publish/subscribe channel so SDK users can push and receive custom messages. The Go backend uses the `ws` transport and persists each published payload in the `_pubsub_messages` table so every node in a cluster can replay and fan-out messages to its local subscribers.

- Endpoint: `/api/pubsub` (WebSocket)
- Auth: the SDK automatically forwards `authStore.token` as a `token` query parameter; cookie-based auth also works. Anonymous clients may subscribe, but publishing requires an authenticated token.
- Reliability: automatic reconnect with topic re-subscription; messages are stored in the database and broadcasted to all connected nodes.

## Quick Start

```rust
use bosbase::BosBase;
use serde_json::Value;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Subscribe to a topic
let unsubscribe = pb.pubsub.subscribe(
    "chat/general",
    |msg: Value| {
        println!("message topic: {}, data: {:?}", msg["topic"], msg["data"]);
    },
    HashMap::new(),
    HashMap::new()
)?;

// Publish to a topic (resolves when the server stores and accepts it)
let ack = pb.pubsub.publish(
    "chat/general",
    json!({ "text": "Hello team!" }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("published at: {}", ack["created"]);

// Later, stop listening
unsubscribe();
```

## API Surface

- `pb.pubsub.publish(topic, data)` → `Result<Value>` - Returns `{ id, topic, created }`
- `pb.pubsub.subscribe(topic, handler)` → `Result<impl FnOnce()>` - Returns unsubscribe function
- `pb.pubsub.unsubscribe(topic?)` → `Result<()>` - Omit `topic` to drop all topics
- `pb.pubsub.disconnect()` - Explicitly close the socket and clear pending requests
- `pb.pubsub.is_connected()` - Check current WebSocket state

## Basic Usage

### Subscribe to a Topic

```rust
let unsubscribe = pb.pubsub.subscribe(
    "notifications",
    |msg: Value| {
        println!("Received: {:?}", msg["data"]);
    },
    HashMap::new(),
    HashMap::new()
)?;

// Later, unsubscribe
unsubscribe();
```

### Publish to a Topic

```rust
// Publish requires authentication
pb.collection("users").auth_with_password(
    "user@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

let ack = pb.pubsub.publish(
    "notifications",
    json!({
        "type": "alert",
        "message": "New update available"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Message published: {:?}", ack);
```

### Multiple Subscriptions

```rust
// Subscribe to multiple topics
let unsubscribe1 = pb.pubsub.subscribe(
    "chat/general",
    |msg: Value| {
        println!("General chat: {:?}", msg);
    },
    HashMap::new(),
    HashMap::new()
)?;

let unsubscribe2 = pb.pubsub.subscribe(
    "chat/private",
    |msg: Value| {
        println!("Private chat: {:?}", msg);
    },
    HashMap::new(),
    HashMap::new()
)?;

// Unsubscribe from specific topic
pb.pubsub.unsubscribe(Some("chat/general".to_string()))?;

// Or unsubscribe from all
pb.pubsub.unsubscribe(None)?;
```

## Complete Examples

### Example 1: Chat Application

```rust
struct ChatClient {
    pb: BosBase,
}

impl ChatClient {
    async fn send_message(
        &self,
        room: &str,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.pubsub.publish(
            &format!("chat/{}", room),
            json!({
                "text": text,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            HashMap::new(),
            HashMap::new()
        ).await?;
        Ok(())
    }

    fn join_room(
        &self,
        room: &str,
    ) -> Result<impl FnOnce(), Box<dyn std::error::Error>> {
        Ok(self.pb.pubsub.subscribe(
            &format!("chat/{}", room),
            |msg: Value| {
                if let Some(data) = msg.get("data") {
                    println!("[{}] {}", data["user"], data["text"]);
                }
            },
            HashMap::new(),
            HashMap::new()
        )?)
    }
}
```

### Example 2: Real-time Notifications

```rust
async fn setup_notifications(
    pb: &BosBase,
    user_id: &str,
) -> Result<impl FnOnce(), Box<dyn std::error::Error>> {
    let topic = format!("notifications/{}", user_id);
    
    Ok(pb.pubsub.subscribe(
        &topic,
        |msg: Value| {
            if let Some(data) = msg.get("data") {
                println!("Notification: {}", data["message"]);
                // Show notification in UI
            }
        },
        HashMap::new(),
        HashMap::new()
    )?)
}

// Send notification
async fn send_notification(
    pb: &BosBase,
    user_id: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    pb.pubsub.publish(
        &format!("notifications/{}", user_id),
        json!({
            "message": message,
            "type": "info"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;
    Ok(())
}
```

## Notes for Clusters

- Messages are written to `_pubsub_messages` with a timestamp; every running node polls the table and pushes new rows to its connected WebSocket clients.
- Old pub/sub rows are cleaned up automatically after a day to keep the table small.
- If a node restarts, it resumes from the latest message and replays new rows as they are inserted, so connected clients on other nodes stay in sync.

## Best Practices

1. **Authentication**: Authenticate before publishing messages
2. **Error Handling**: Handle connection errors and reconnection
3. **Unsubscribe**: Always unsubscribe when done to prevent memory leaks
4. **Topic Naming**: Use hierarchical topic names (e.g., `chat/general`, `notifications/user123`)
5. **Message Size**: Keep messages reasonably sized for performance

## Related Documentation

- [Realtime API](./REALTIME.md) - Real-time record updates
- [Authentication](./AUTHENTICATION.md) - User authentication


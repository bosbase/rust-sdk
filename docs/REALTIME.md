# Realtime API - Rust SDK Documentation

## Overview

The Realtime API enables real-time updates for collection records using **Server-Sent Events (SSE)**. It allows you to subscribe to changes in collections or specific records and receive instant notifications when records are created, updated, or deleted.

**Key Features:**
- Real-time notifications for record changes
- Collection-level and record-level subscriptions
- Automatic connection management and reconnection
- Authorization support
- Subscription options (expand, custom headers, query params)
- Event-driven architecture

**Backend Endpoints:**
- `GET /api/realtime` - Establish SSE connection
- `POST /api/realtime` - Set subscriptions

## How It Works

1. **Connection**: The SDK establishes an SSE connection to `/api/realtime`
2. **Client ID**: Server sends `PB_CONNECT` event with a unique `clientId`
3. **Subscriptions**: Client submits subscription topics via POST request
4. **Events**: Server sends events when matching records change
5. **Reconnection**: SDK automatically reconnects on connection loss

## Basic Usage

### Subscribe to Collection Changes

Subscribe to all changes in a collection:

```rust
use bosbase::BosBase;
use serde_json::Value;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Subscribe to all changes in the 'posts' collection
let unsubscribe = pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Action: {}", e["action"]);
        println!("Record: {:?}", e["record"]);
    },
    HashMap::new(),
    HashMap::new()
)?;

// Later, unsubscribe
unsubscribe();
```

### Subscribe to Specific Record

Subscribe to changes for a single record:

```rust
// Subscribe to changes for a specific post
pb.collection("posts").subscribe(
    "RECORD_ID",
    |e: Value| {
        println!("Record changed: {:?}", e["record"]);
        println!("Action: {}", e["action"]);
    },
    HashMap::new(),
    HashMap::new()
)?;
```

### Multiple Subscriptions

You can subscribe multiple times to the same or different topics:

```rust
fn handle_change(e: Value) {
    println!("Change event: {:?}", e);
}

fn handle_all_changes(e: Value) {
    println!("Collection-wide change: {:?}", e);
}

// Subscribe to multiple records
let unsubscribe1 = pb.collection("posts").subscribe(
    "RECORD_ID_1",
    handle_change,
    HashMap::new(),
    HashMap::new()
)?;

let unsubscribe2 = pb.collection("posts").subscribe(
    "RECORD_ID_2",
    handle_change,
    HashMap::new(),
    HashMap::new()
)?;

let unsubscribe3 = pb.collection("posts").subscribe(
    "*",
    handle_all_changes,
    HashMap::new(),
    HashMap::new()
)?;

// Unsubscribe individually
unsubscribe1();
unsubscribe2();
unsubscribe3();
```

## Event Structure

Each event received contains:

```rust
// Event structure:
// {
//   "action": "create" | "update" | "delete",  // Action type
//   "record": {                                 // Record data
//     "id": "RECORD_ID",
//     "collectionId": "COLLECTION_ID",
//     "collectionName": "collection_name",
//     "created": "2023-01-01 00:00:00.000Z",
//     "updated": "2023-01-01 00:00:00.000Z",
//     // ... other fields
//   }
// }
```

### PB_CONNECT Event

When the connection is established, you receive a `PB_CONNECT` event:

```rust
use serde_json::json;

pb.realtime.subscribe(
    "PB_CONNECT",
    |e: Value| {
        println!("Connected! Client ID: {}", e["clientId"]);
        // e["clientId"] - unique client identifier
    },
    HashMap::new(),
    HashMap::new()
)?;
```

## Subscription Topics

### Collection-Level Subscription

Subscribe to all changes in a collection:

```rust
// Wildcard subscription - all records in collection
pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
```

**Access Control**: Uses the collection's `ListRule` to determine if the subscriber has access to receive events.

### Record-Level Subscription

Subscribe to changes for a specific record:

```rust
// Specific record subscription
pb.collection("posts").subscribe(
    "RECORD_ID",
    |e: Value| {
        println!("Record changed: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
```

**Access Control**: Uses the collection's `ViewRule` to determine if the subscriber has access to receive events.

## Subscription Options

You can pass additional options when subscribing:

```rust
use serde_json::json;

let mut query = HashMap::new();
query.insert("filter".to_string(), json!(r#"status = "published""#));
query.insert("expand".to_string(), json!("author"));

let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    query,
    headers
)?;
```

### Expand Relations

Expand relations in the event data:

```rust
let mut query = HashMap::new();
query.insert("expand".to_string(), json!("author,categories"));

pb.collection("posts").subscribe(
    "RECORD_ID",
    |e: Value| {
        if let Some(record) = e.get("record") {
            if let Some(expand) = record.get("expand") {
                if let Some(author) = expand.get("author") {
                    println!("Author: {}", author["name"]);
                }
            }
        }
    },
    query,
    HashMap::new()
)?;
```

### Filter with Query Parameters

Use query parameters for API rule filtering:

```rust
let mut query = HashMap::new();
query.insert("filter".to_string(), json!(r#"status = "published""#));

pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Published post changed: {:?}", e);
    },
    query,
    HashMap::new()
)?;
```

## Unsubscribing

### Unsubscribe from Specific Topic

```rust
// Remove all subscriptions for a specific record
pb.collection("posts").unsubscribe(Some("RECORD_ID".to_string()));

// Remove all wildcard subscriptions for the collection
pb.collection("posts").unsubscribe(Some("*".to_string()));
```

### Unsubscribe from All

```rust
// Unsubscribe from all subscriptions in the collection
pb.collection("posts").unsubscribe(None);

// Or unsubscribe from everything
pb.realtime.unsubscribe();
```

### Unsubscribe Using Returned Function

```rust
let unsubscribe = pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;

// Later...
unsubscribe();  // Removes this specific subscription
```

## Connection Management

### Disconnect Handler

Handle disconnection events:

```rust
// Note: The Rust SDK handles reconnection automatically
// You can check connection status through the realtime service
```

### Automatic Reconnection

The SDK automatically:
- Reconnects when the connection is lost
- Resubmits all active subscriptions
- Handles network interruptions gracefully
- Closes connection after 5 minutes of inactivity (server-side timeout)

## Authorization

### Authenticated Subscriptions

Subscriptions respect authentication. If you're authenticated, events are filtered based on your permissions:

```rust
// Authenticate first
pb.collection("users").auth_with_password(
    "user@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Now subscribe - events will respect your permissions
pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
```

### Authorization Rules

- **Collection-level (`*`)**: Uses `ListRule` to determine access
- **Record-level**: Uses `ViewRule` to determine access
- **Superusers**: Can receive all events (if rules allow)
- **Guests**: Only receive events they have permission to see

### Auth State Changes

When authentication state changes, you may need to resubscribe:

```rust
// After login/logout, resubscribe to update permissions
pb.collection("users").auth_with_password(
    "user@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Re-subscribe to update auth state in realtime connection
pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
```

## Advanced Examples

### Example 1: Real-time Chat

```rust
async fn setup_chat_room(
    pb: &BosBase,
    room_id: &str,
) -> Result<impl FnOnce(), Box<dyn std::error::Error>> {
    let mut query = HashMap::new();
    query.insert("filter".to_string(), json!(format!(r#"roomId = "{}""#, room_id)));

    let unsubscribe = pb.collection("messages").subscribe(
        "*",
        move |e: Value| {
            // Filter for this room only
            if let Some(record) = e.get("record") {
                if record.get("roomId").and_then(|r| r.as_str()) == Some(room_id) {
                    match e["action"].as_str() {
                        Some("create") => {
                            println!("New message: {:?}", record);
                        }
                        Some("delete") => {
                            println!("Message deleted: {}", record["id"]);
                        }
                        _ => {}
                    }
                }
            }
        },
        query,
        HashMap::new()
    )?;
    
    Ok(unsubscribe)
}

// Usage
let unsubscribe = setup_chat_room(&pb, "ROOM_ID").await?;
// ... later
unsubscribe();
```

### Example 2: Real-time Dashboard

```rust
async fn setup_dashboard(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    // Posts updates
    let mut posts_query = HashMap::new();
    posts_query.insert("filter".to_string(), json!(r#"status = "published""#));
    posts_query.insert("expand".to_string(), json!("author"));

    pb.collection("posts").subscribe(
        "*",
        |e: Value| {
            match e["action"].as_str() {
                Some("create") => {
                    println!("New post: {:?}", e["record"]);
                }
                Some("update") => {
                    println!("Post updated: {:?}", e["record"]);
                }
                _ => {}
            }
        },
        posts_query,
        HashMap::new()
    )?;

    // Comments updates
    let mut comments_query = HashMap::new();
    comments_query.insert("expand".to_string(), json!("user"));

    pb.collection("comments").subscribe(
        "*",
        |e: Value| {
            if let Some(record) = e.get("record") {
                if let Some(post_id) = record.get("postId") {
                    println!("Comment on post {}: {:?}", post_id, record);
                }
            }
        },
        comments_query,
        HashMap::new()
    )?;

    Ok(())
}
```

### Example 3: User Activity Tracking

```rust
async fn track_user_activity(
    pb: &BosBase,
    user_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut query = HashMap::new();
    query.insert("filter".to_string(), json!(format!(r#"author = "{}""#, user_id)));

    pb.collection("posts").subscribe(
        "*",
        move |e: Value| {
            // Only track changes to user's own posts
            if let Some(record) = e.get("record") {
                if record.get("author").and_then(|a| a.as_str()) == Some(user_id) {
                    println!("Your post {}: {:?}", e["action"], record["title"]);
                    
                    if e["action"].as_str() == Some("update") {
                        println!("Post updated notification");
                    }
                }
            }
        },
        query,
        HashMap::new()
    )?;

    Ok(())
}
```

### Example 4: Real-time Collaboration

```rust
async fn track_document_edits(
    pb: &BosBase,
    document_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut query = HashMap::new();
    query.insert("expand".to_string(), json!("lastEditor"));

    pb.collection("documents").subscribe(
        document_id,
        |e: Value| {
            if e["action"].as_str() == Some("update") {
                if let Some(record) = e.get("record") {
                    let last_editor = record.get("lastEditor");
                    let updated_at = record.get("updated");
                    
                    // Show who last edited the document
                    println!("Document edited by: {:?}, at: {:?}", last_editor, updated_at);
                }
            }
        },
        query,
        HashMap::new()
    )?;

    Ok(())
}
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
) {
    Ok(unsubscribe) => {
        println!("Subscribed successfully");
        // Store unsubscribe function for later use
    }
    Err(err) => {
        match err.status() {
            403 => {
                eprintln!("Permission denied");
            }
            404 => {
                eprintln!("Collection not found");
            }
            _ => {
                eprintln!("Subscription error: {:?}", err);
            }
        }
    }
}
```

## Best Practices

1. **Unsubscribe When Done**: Always unsubscribe when components unmount or subscriptions are no longer needed
2. **Handle Disconnections**: The SDK handles reconnection automatically, but monitor for issues
3. **Filter Server-Side**: Use query parameters to filter events server-side when possible
4. **Limit Subscriptions**: Don't subscribe to more collections than necessary
5. **Use Record-Level When Possible**: Prefer record-level subscriptions over collection-level when you only need specific records
6. **Monitor Connection**: Track connection state for debugging and user feedback
7. **Handle Errors**: Wrap subscriptions in error handling
8. **Respect Permissions**: Understand that events respect API rules and permissions

## Limitations

- **Maximum Subscriptions**: Up to 1000 subscriptions per client
- **Topic Length**: Maximum 2500 characters per topic
- **Idle Timeout**: Connection closes after 5 minutes of inactivity
- **Network Dependency**: Requires stable network connection
- **SSE Support**: SSE requires modern HTTP clients (not available in all environments)

## Troubleshooting

### Connection Not Establishing

```rust
// Manually trigger connection by subscribing
let unsubscribe = pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Connected and receiving events");
    },
    HashMap::new(),
    HashMap::new()
)?;
```

### Events Not Received

1. Check API rules - you may not have permission
2. Verify subscription is active
3. Check network connectivity
4. Review server logs for errors

### Memory Leaks

Always unsubscribe:

```rust
// Good
let unsubscribe = pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
// ... later
unsubscribe();

// Bad - no cleanup
pb.collection("posts").subscribe(
    "*",
    |e: Value| {
        println!("Event: {:?}", e);
    },
    HashMap::new(),
    HashMap::new()
)?;
// Never unsubscribed - potential memory leak!
```

## Related Documentation

- [API Records](./API_RECORDS.md) - CRUD operations
- [Collections](./COLLECTIONS.md) - Collection configuration
- [API Rules and Filters](./API_RULES_AND_FILTERS.md) - Understanding API rules


# AI Development Guide - Rust SDK Documentation

This guide provides a comprehensive, fast reference for AI systems to quickly develop applications using the BosBase Rust SDK. All examples are production-ready and follow best practices.

## Table of Contents

1. [Authentication](#authentication)
2. [Initialize Collections](#initialize-collections)
3. [Define Collection Fields](#define-collection-fields)
4. [Add Data to Collections](#add-data-to-collections)
5. [Modify Collection Data](#modify-collection-data)
6. [Delete Data from Collections](#delete-data-from-collections)
7. [Query Collection Contents](#query-collection-contents)
8. [Upload Files](#upload-files)

## Authentication

### Initialize Client

```rust
use bosbase::BosBase;

let pb = BosBase::new("http://localhost:8090");
```

### Password Authentication

```rust
use std::collections::HashMap;

// Authenticate with email/username and password
let auth_data = pb.collection("users").auth_with_password(
    "user@example.com",
    "password123",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Auth data is automatically stored
println!("Token: {}", auth_data["token"]);
println!("Record: {:?}", auth_data["record"]);
```

### Check Authentication Status

```rust
if pb.auth_store().is_valid() {
    println!("Authenticated");
} else {
    println!("Not authenticated");
}
```

### Logout

```rust
pb.auth_store().clear();
```

## Initialize Collections

### Create Base Collection

```rust
use serde_json::json;

let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "articles",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true,
                "min": 6,
                "max": 100
            },
            {
                "name": "description",
                "type": "text"
            }
        ],
        "listRule": r#"@request.auth.id != "" || status = "public""#,
        "viewRule": r#"@request.auth.id != "" || status = "public""#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Add Data to Collections

```rust
let record = pb.collection("articles").create(
    json!({
        "title": "My Article",
        "description": "Article description"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Modify Collection Data

```rust
pb.collection("articles").update(
    "RECORD_ID",
    json!({
        "title": "Updated Title"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Delete Data from Collections

```rust
pb.collection("articles").delete(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Query Collection Contents

```rust
// Get list with pagination
let result = pb.collection("articles").get_list(
    1,
    20,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;

// Get single record
let record = pb.collection("articles").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Upload Files

```rust
use bosbase::FileAttachment;

let mut files = Vec::new();
files.push(FileAttachment {
    field: "image".to_string(),
    filename: "photo.jpg".to_string(),
    content_type: "image/jpeg".to_string(),
    data: image_bytes,
});

let record = pb.collection("articles").create(
    json!({
        "title": "Article with Image"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;
```

## Related Documentation

- [Collections](./COLLECTIONS.md) - Complete collection guide
- [API Records](./API_RECORDS.md) - Record operations
- [Authentication](./AUTHENTICATION.md) - Authentication guide


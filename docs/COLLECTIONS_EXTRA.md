# Collections Extra - Rust SDK Documentation

## Overview

This document provides additional information about working with Collections and Fields in the BosBase Rust SDK, complementing the main Collections documentation.

## Collection Types

Currently there are 3 collection types: **Base**, **View** and **Auth**.

### Base Collection

**Base collection** is the default collection type and it could be used to store any application data (articles, products, posts, etc.).

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Create a base collection
let collection = pb.collections.create_base(
    "articles",
    json!({
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
    HashMap::new()
).await?;
```

### View Collection

**View collection** is a read-only collection type where the data is populated from a plain SQL `SELECT` statement, allowing users to perform aggregations or any other custom queries.

```rust
// Create a view collection
let view_collection = pb.collections.create_view(
    "post_stats",
    r#"SELECT posts.id, posts.name, count(comments.id) as totalComments 
       FROM posts 
       LEFT JOIN comments on comments.postId = posts.id 
       GROUP BY posts.id"#,
    HashMap::new(),
    HashMap::new()
).await?;
```

**Note**: View collections don't receive realtime events because they don't have create/update/delete operations.

### Auth Collection

**Auth collection** has everything from the **Base collection** but with some additional special fields to help you manage your app users and also provide various authentication options.

Each Auth collection has the following special system fields: `email`, `emailVisibility`, `verified`, `password` and `tokenKey`. They cannot be renamed or deleted but can be configured using their specific field options.

```rust
// Create an auth collection
let users_collection = pb.collections.create_auth(
    "users",
    json!({
        "fields": [
            {
                "name": "name",
                "type": "text",
                "required": true
            },
            {
                "name": "role",
                "type": "select",
                "options": {
                    "values": ["employee", "staff", "admin"]
                }
            }
        ]
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## Field Types

BosBase supports various field types. See the main [Collections](./COLLECTIONS.md) documentation for a complete list.

## Related Documentation

- [Collections](./COLLECTIONS.md) - Main collections documentation
- [Collection API](./COLLECTION_API.md) - Collection management API


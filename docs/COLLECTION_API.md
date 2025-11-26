# Collection API - Rust SDK Documentation

## Overview

The Collection API provides endpoints for managing collections (Base, Auth, and View types). All operations require superuser authentication and allow you to create, read, update, and delete collections along with their schemas and configurations.

**Key Features:**
- List and search collections
- View collection details
- Create collections (base, auth, view)
- Update collection schemas and rules
- Delete collections
- Truncate collections (delete all records)
- Import collections in bulk
- Get collection scaffolds (templates)

**Backend Endpoints:**
- `GET /api/collections` - List collections
- `GET /api/collections/{collection}` - View collection
- `POST /api/collections` - Create collection
- `PATCH /api/collections/{collection}` - Update collection
- `DELETE /api/collections/{collection}` - Delete collection
- `DELETE /api/collections/{collection}/truncate` - Truncate collection
- `PUT /api/collections/import` - Import collections
- `GET /api/collections/meta/scaffolds` - Get scaffolds

**Note**: All Collection API operations require superuser authentication.

## Authentication

All Collection API operations require superuser authentication:

```rust
use bosbase::BosBase;

let pb = BosBase::new("http://127.0.0.1:8090");

// Authenticate as superuser
pb.admins().auth_with_password("admin@example.com", "password").await?;
// OR
pb.collection("_superusers").auth_with_password(
    "admin@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;
```

## List Collections

Returns a paginated list of collections with support for filtering and sorting.

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Basic list
let result = pb.collections.get_list(
    1,
    30,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;

println!("Page: {}", result["page"]);
println!("Per Page: {}", result["perPage"]);
println!("Total Items: {}", result["totalItems"]);
println!("Items: {:?}", result["items"]);
```

### Advanced Filtering and Sorting

```rust
// Filter by type
let auth_collections = pb.collections.get_list(
    1,
    100,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"type = "auth""#.to_string()),
    None,
    None,
    None
).await?;

// Filter by name pattern
let matching_collections = pb.collections.get_list(
    1,
    100,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"name ~ "user""#.to_string()),
    None,
    None,
    None
).await?;

// Sort by creation date
let sorted_collections = pb.collections.get_list(
    1,
    100,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-created".to_string()),
    None,
    None
).await?;

// Complex filter
let filtered = pb.collections.get_list(
    1,
    100,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"type = "base" && system = false && created >= "2023-01-01""#.to_string()),
    Some("name".to_string()),
    None,
    None
).await?;
```

### Get Full List

```rust
// Get all collections at once
let all_collections = pb.collections.get_full_list(
    200,
    HashMap::new(),
    HashMap::new(),
    Some("system = false".to_string()),
    Some("name".to_string()),
    None,
    None
).await?;
```

### Get First Matching Collection

```rust
// Get first auth collection
let auth_collection = pb.collections.get_first_list_item(
    r#"type = "auth""#.to_string(),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## View Collection

Retrieve a single collection by ID or name:

```rust
// By name
let collection = pb.collections.get_one(
    "posts",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// By ID
let collection = pb.collections.get_one(
    "_pbc_2287844090",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// With field selection
let collection = pb.collections.get_one(
    "posts",
    HashMap::new(),
    HashMap::new(),
    None,
    Some("id,name,type,fields.name,fields.type".to_string())
).await?;
```

## Create Collection

Create a new collection with schema fields and configuration.

### Create Base Collection

```rust
use serde_json::json;

let base_collection = pb.collections.create(
    json!({
        "name": "posts",
        "type": "base",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true,
                "min": 10,
                "max": 255
            },
            {
                "name": "content",
                "type": "editor",
                "required": false
            },
            {
                "name": "published",
                "type": "bool",
                "required": false
            },
            {
                "name": "author",
                "type": "relation",
                "required": true,
                "options": {
                    "collectionId": "_pbc_users_auth_"
                },
                "maxSelect": 1
            }
        ],
        "listRule": r#"@request.auth.id != """#,
        "viewRule": r#"@request.auth.id != "" || published = true"#,
        "createRule": r#"@request.auth.id != """#,
        "updateRule": "author = @request.auth.id",
        "deleteRule": "author = @request.auth.id"
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Create Auth Collection

```rust
let auth_collection = pb.collections.create(
    json!({
        "name": "users",
        "type": "auth",
        "fields": [
            {
                "name": "name",
                "type": "text",
                "required": false
            },
            {
                "name": "avatar",
                "type": "file",
                "required": false,
                "maxSelect": 1,
                "maxSize": 2097152,  // 2MB
                "mimeTypes": ["image/jpeg", "image/png"]
            }
        ],
        "listRule": json!(null),
        "viewRule": r#"@request.auth.id = id"#,
        "createRule": json!(null),
        "updateRule": r#"@request.auth.id = id"#,
        "deleteRule": r#"@request.auth.id = id"#,
        "manageRule": json!(null),
        "authRule": "verified = true",
        "passwordAuth": {
            "enabled": true,
            "identityFields": ["email", "username"]
        }
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Create View Collection

```rust
let view_collection = pb.collections.create_view(
    "post_stats",
    "SELECT posts.id, posts.name, count(comments.id) as totalComments 
     FROM posts LEFT JOIN comments on comments.postId = posts.id 
     GROUP BY posts.id",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Update Collection

Update collection schema, rules, or configuration:

```rust
// Update collection rules
pb.collections.update(
    "articles",
    json!({
        "listRule": r#"@request.auth.id != "" && (status = "published" || status = "draft")"#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Update collection name
pb.collections.update(
    "articles",
    json!({
        "name": "posts"
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Update fields
let mut collection = pb.collections.get_one(
    "articles",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    fields.push(json!({
        "name": "views",
        "type": "number",
        "min": 0
    }));
}

pb.collections.update(
    "articles",
    json!({
        "fields": collection["fields"]
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Delete Collection

Delete a collection and all its records:

```rust
pb.collections.delete_collection(
    "articles",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Truncate Collection

Delete all records in a collection without deleting the collection itself:

```rust
pb.collections.truncate(
    "articles",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Import Collections

Import multiple collections at once:

```rust
let collections_data = json!([
    {
        "name": "posts",
        "type": "base",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            }
        ]
    },
    {
        "name": "comments",
        "type": "base",
        "fields": [
            {
                "name": "message",
                "type": "text",
                "required": true
            }
        ]
    }
]);

pb.collections.import(
    collections_data,
    HashMap::new(),
    HashMap::new()
).await?;
```

## Get Scaffolds

Get collection templates/scaffolds:

```rust
let scaffolds = pb.collections.get_scaffolds(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Available scaffolds: {:?}", scaffolds);
```

## Complete Examples

### Example 1: Create Blog Collections

```rust
async fn create_blog_collections(
    pb: &BosBase,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create users collection
    let users = pb.collections.create_auth(
        "users",
        json!({
            "fields": [
                {
                    "name": "name",
                    "type": "text",
                    "required": true
                }
            ]
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    // Create posts collection
    let posts = pb.collections.create_base(
        "posts",
        json!({
            "fields": [
                {
                    "name": "title",
                    "type": "text",
                    "required": true
                },
                {
                    "name": "content",
                    "type": "editor"
                },
                {
                    "name": "author",
                    "type": "relation",
                    "options": {
                        "collectionId": users["id"]
                    },
                    "maxSelect": 1
                }
            ],
            "listRule": r#"status = "published" || @request.auth.id != """#,
            "viewRule": r#"status = "published" || @request.auth.id != """#,
            "createRule": r#"@request.auth.id != """#,
            "updateRule": "author = @request.auth.id",
            "deleteRule": "author = @request.auth.id"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    // Create comments collection
    pb.collections.create_base(
        "comments",
        json!({
            "fields": [
                {
                    "name": "message",
                    "type": "text",
                    "required": true
                },
                {
                    "name": "post",
                    "type": "relation",
                    "options": {
                        "collectionId": posts["id"]
                    },
                    "maxSelect": 1
                },
                {
                    "name": "user",
                    "type": "relation",
                    "options": {
                        "collectionId": users["id"]
                    },
                    "maxSelect": 1
                }
            ],
            "listRule": r#"@request.auth.id != """#,
            "createRule": r#"@request.auth.id != """#,
            "updateRule": "user = @request.auth.id",
            "deleteRule": "user = @request.auth.id"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    Ok(())
}
```

## Best Practices

1. **Superuser Authentication**: Always authenticate as superuser before using Collection API
2. **Backup Before Changes**: Create backups before modifying collection schemas
3. **Test in Development**: Test collection changes in development first
4. **Document Collections**: Document your collection schemas and rules
5. **Version Control**: Keep collection schemas in version control
6. **Incremental Changes**: Make incremental changes rather than large schema updates

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection types and field management
- [API Rules and Filters](./API_RULES_AND_FILTERS.md) - Setting up API rules


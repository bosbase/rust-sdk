# Collections - Rust SDK Documentation

## Overview

**Collections** represent your application data. Under the hood they are backed by plain SQLite tables that are generated automatically with the collection **name** and **fields** (columns).

A single entry of a collection is called a **record** (a single row in the SQL table).

## Collection Types

### Base Collection

Default collection type for storing any application data.

```rust
use bosbase::BosBase;
use serde_json::json;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "articles",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            },
            {
                "name": "description",
                "type": "text"
            }
        ]
    }),
    Default::default(),
    Default::default(),
    None,
    None
).await?;
```

### View Collection

Read-only collection populated from a SQL SELECT statement.

```rust
let view = pb.collections.create_view(
    "post_stats",
    "SELECT posts.id, posts.name, count(comments.id) as totalComments 
     FROM posts LEFT JOIN comments on comments.postId = posts.id 
     GROUP BY posts.id",
    Default::default(),
    Default::default()
).await?;
```

### Auth Collection

Base collection with authentication fields (email, password, etc.).

```rust
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
    Default::default(),
    Default::default()
).await?;
```

## Collections API

### List Collections

```rust
use std::collections::HashMap;
use serde_json::Value;

// Get paginated list
let result = pb.collections.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;

// Get full list
let all = pb.collections.get_full_list(
    200,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;
```

### Get Collection

```rust
let collection = pb.collections.get_one(
    "articles",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Create Collection

```rust
// Manual creation
let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "articles",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            },
            {
                "name": "created",
                "type": "autodate",
                "required": false,
                "onCreate": true,
                "onUpdate": false
            },
            {
                "name": "updated",
                "type": "autodate",
                "required": false,
                "onCreate": true,
                "onUpdate": true
            }
        ]
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Update Collection

```rust
// Update collection rules
pb.collections.update(
    "articles",
    json!({
        "listRule": "published = true"
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
```

### Add Fields to Collection

To add a new field to an existing collection, fetch the collection, add the field to the fields array, and update:

```rust
// Get existing collection
let mut collection = pb.collections.get_one(
    "articles",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Add new field to existing fields
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    fields.push(json!({
        "name": "views",
        "type": "number",
        "min": 0,
        "onlyInt": true
    }));
}

// Update collection with new field
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

### Delete Fields from Collection

To delete a field, fetch the collection, remove the field from the fields array, and update:

```rust
// Get existing collection
let mut collection = pb.collections.get_one(
    "articles",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Remove field by filtering it out
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    fields.retain(|field| {
        field.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n != "oldFieldName")
            .unwrap_or(true)
    });
}

// Update collection without the deleted field
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

### Modify Fields in Collection

To modify an existing field (e.g., change its type, add options, etc.), fetch the collection, update the field object, and save:

```rust
// Get existing collection
let mut collection = pb.collections.get_one(
    "articles",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Find and modify a field
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    if let Some(title_field) = fields.iter_mut().find(|f| {
        f.get("name").and_then(|n| n.as_str()) == Some("title")
    }) {
        title_field["max"] = json!(200);
        title_field["required"] = json!(true);
    }
}

// Save changes
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

### Delete Collection

```rust
pb.collections.delete(
    "articles",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Records API

### List Records

```rust
use std::collections::HashMap;

let result = pb.collection("articles").get_list(
    1,
    20,
    false,
    HashMap::new(),
    HashMap::new(),
    Some("published = true".to_string()),
    Some("-created".to_string()),
    Some("author".to_string()),
    None
).await?;
```

### Get Record

```rust
let record = pb.collection("articles").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    Some("author,category".to_string()),
    None
).await?;
```

### Create Record

```rust
let record = pb.collection("articles").create(
    json!({
        "title": "My Article",
        "views+": 1  // Field modifier
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Update Record

```rust
pb.collection("articles").update(
    "RECORD_ID",
    json!({
        "title": "Updated",
        "views+": 1,
        "tags+": "new-tag"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Delete Record

```rust
pb.collection("articles").delete(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Field Types

### BoolField

```rust
// Field definition
{
    "name": "published",
    "type": "bool",
    "required": true
}

// Usage
pb.collection("articles").create(
    json!({
        "published": true
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### NumberField

```rust
// Field definition
{
    "name": "views",
    "type": "number",
    "min": 0
}

// Usage with modifier
pb.collection("articles").update(
    "ID",
    json!({
        "views+": 1
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### TextField

```rust
// Field definition
{
    "name": "title",
    "type": "text",
    "required": true,
    "min": 6,
    "max": 100
}

// Usage with autogenerate
pb.collection("articles").create(
    json!({
        "slug:autogenerate": "article-"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### EmailField

```rust
// Field definition
{
    "name": "email",
    "type": "email",
    "required": true
}
```

### URLField

```rust
// Field definition
{
    "name": "website",
    "type": "url"
}
```

### EditorField

```rust
// Field definition
{
    "name": "content",
    "type": "editor",
    "required": true
}

// Usage
pb.collection("articles").create(
    json!({
        "content": "<p>HTML content</p>"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### DateField

```rust
// Field definition
{
    "name": "published_at",
    "type": "date"
}

// Usage
pb.collection("articles").create(
    json!({
        "published_at": "2024-11-10 18:45:27.123Z"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### AutodateField

**Important Note:** Bosbase does not initialize `created` and `updated` fields by default. To use these fields, you must explicitly add them when initializing the collection. For autodate fields, `onCreate` and `onUpdate` must be direct properties of the field object, not nested in an `options` object:

```rust
// Create field with proper structure
{
    "name": "created",
    "type": "autodate",
    "required": false,
    "onCreate": true,  // Set on record creation (direct property)
    "onUpdate": false  // Don't update on record update (direct property)
}

// For updated field
{
    "name": "updated",
    "type": "autodate",
    "required": false,
    "onCreate": true,  // Set on record creation (direct property)
    "onUpdate": true   // Update on record update (direct property)
}

// The value is automatically set by the backend based on onCreate and onUpdate properties
```

### SelectField

```rust
// Single select
{
    "name": "status",
    "type": "select",
    "options": {
        "values": ["draft", "published"]
    },
    "maxSelect": 1
}

// Usage
pb.collection("articles").create(
    json!({
        "status": "published"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Multiple select
{
    "name": "tags",
    "type": "select",
    "options": {
        "values": ["tech", "design"]
    },
    "maxSelect": 5
}

// Usage with modifier
pb.collection("articles").update(
    "ID",
    json!({
        "tags+": "marketing"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### FileField

```rust
// Single file
{
    "name": "cover",
    "type": "file",
    "maxSelect": 1,
    "mimeTypes": ["image/jpeg"]
}

// Usage with file upload
use bosbase::FileAttachment;

let mut files = Vec::new();
files.push(FileAttachment {
    field: "cover".to_string(),
    filename: "image.jpg".to_string(),
    content_type: "image/jpeg".to_string(),
    data: image_bytes,
});

pb.collection("articles").create(
    json!({
        "title": "My Article"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;
```

### RelationField

```rust
// Field definition
{
    "name": "author",
    "type": "relation",
    "options": {
        "collectionId": "users"
    },
    "maxSelect": 1
}

// Usage
pb.collection("articles").create(
    json!({
        "author": "USER_ID"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// With expand
let record = pb.collection("articles").get_one(
    "ID",
    HashMap::new(),
    HashMap::new(),
    Some("author".to_string()),
    None
).await?;
```

### JSONField

```rust
// Field definition
{
    "name": "metadata",
    "type": "json"
}

// Usage
pb.collection("articles").create(
    json!({
        "metadata": {
            "seo": {
                "title": "SEO Title"
            }
        }
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### GeoPointField

```rust
// Field definition
{
    "name": "location",
    "type": "geoPoint"
}

// Usage
pb.collection("places").create(
    json!({
        "location": {
            "lon": 139.6917,
            "lat": 35.6586
        }
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Complete Example

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pb = BosBase::new("http://localhost:8090");
    pb.admins().auth_with_password("admin@example.com", "password").await?;

    // Create collections
    let users = pb.collections.create_auth(
        "users",
        json!({}),
        HashMap::new(),
        HashMap::new()
    ).await?;

    let articles = pb.collections.create_base(
        "articles",
        json!({
            "fields": [
                {
                    "name": "title",
                    "type": "text",
                    "required": true
                },
                {
                    "name": "author",
                    "type": "relation",
                    "options": {
                        "collectionId": users["id"]
                    },
                    "maxSelect": 1
                }
            ]
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    // Create and authenticate user
    let user = pb.collection("users").create(
        json!({
            "email": "user@example.com",
            "password": "password123",
            "passwordConfirm": "password123"
        }),
        HashMap::new(),
        Vec::new(),
        HashMap::new(),
        None,
        None
    ).await?;

    pb.collection("users").auth_with_password(
        "user@example.com",
        "password123",
        HashMap::new(),
        HashMap::new(),
        None
    ).await?;

    // Create article
    let article = pb.collection("articles").create(
        json!({
            "title": "My Article",
            "author": user["id"]
        }),
        HashMap::new(),
        Vec::new(),
        HashMap::new(),
        None,
        None
    ).await?;

    println!("Created article: {:?}", article);
    Ok(())
}
```


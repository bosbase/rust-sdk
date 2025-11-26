# API Records - Rust SDK Documentation

## Overview

The Records API provides comprehensive CRUD (Create, Read, Update, Delete) operations for collection records, along with powerful search, filtering, and authentication capabilities.

**Key Features:**
- Paginated list and search with filtering and sorting
- Single record retrieval with expand support
- Create, update, and delete operations
- Batch operations for multiple records
- Authentication methods (password, OAuth2, OTP)
- Email verification and password reset
- Relation expansion up to 6 levels deep
- Field selection and excerpt modifiers

**Backend Endpoints:**
- `GET /api/collections/{collection}/records` - List records
- `GET /api/collections/{collection}/records/{id}` - View record
- `POST /api/collections/{collection}/records` - Create record
- `PATCH /api/collections/{collection}/records/{id}` - Update record
- `DELETE /api/collections/{collection}/records/{id}` - Delete record
- `POST /api/batch` - Batch operations

## CRUD Operations

### List/Search Records

Returns a paginated records list with support for sorting, filtering, and expansion.

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Basic list with pagination
let result = pb.collection("posts").get_list(
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

println!("Page: {}", result["page"]);
println!("Per Page: {}", result["perPage"]);
println!("Total Items: {}", result["totalItems"]);
println!("Total Pages: {}", result["totalPages"]);
```

#### Advanced List with Filtering and Sorting

```rust
// Filter and sort
let result = pb.collection("posts").get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"created >= "2022-01-01 00:00:00" && status = "published""#.to_string()),
    Some("-created,title".to_string()),  // DESC by created, ASC by title
    Some("author,categories".to_string()),
    None
).await?;

// Filter with operators
let result2 = pb.collection("posts").get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"title ~ "javascript" && views > 100"#.to_string()),
    Some("-views".to_string()),
    None,
    None
).await?;
```

#### Get Full List

Fetch all records at once (useful for small collections):

```rust
// Get all records
let all_posts = pb.collection("posts").get_full_list(
    200,
    HashMap::new(),
    HashMap::new(),
    Some(r#"status = "published""#.to_string()),
    Some("-created".to_string()),
    None,
    None
).await?;
```

#### Get First Matching Record

Get only the first record that matches a filter:

```rust
let post = pb.collection("posts").get_first_list_item(
    r#"slug = "my-post-slug""#.to_string(),
    HashMap::new(),
    HashMap::new(),
    Some("author,categories.tags".to_string()),
    None
).await?;
```

### View Record

Retrieve a single record by ID:

```rust
// Basic retrieval
let record = pb.collection("posts").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// With expanded relations
let record = pb.collection("posts").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    Some("author,categories,tags".to_string()),
    None
).await?;

// Nested expand
let record = pb.collection("comments").get_one(
    "COMMENT_ID",
    HashMap::new(),
    HashMap::new(),
    Some("post.author,user".to_string()),
    None
).await?;

// Field selection
let record = pb.collection("posts").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    Some("id,title,content,author.name".to_string())
).await?;
```

### Create Record

Create a new record:

```rust
use bosbase::FileAttachment;

// Simple create
let record = pb.collection("posts").create(
    json!({
        "title": "My First Post",
        "content": "Lorem ipsum...",
        "status": "draft"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Create with relations
let record = pb.collection("posts").create(
    json!({
        "title": "My Post",
        "author": "AUTHOR_ID",           // Single relation
        "categories": ["cat1", "cat2"]   // Multiple relation
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Create with file upload
let mut files = Vec::new();
files.push(FileAttachment {
    field: "image".to_string(),
    filename: "photo.jpg".to_string(),
    content_type: "image/jpeg".to_string(),
    data: image_bytes,
});

let record = pb.collection("posts").create(
    json!({
        "title": "My Post"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;

// Create with expand to get related data immediately
let record = pb.collection("posts").create(
    json!({
        "title": "My Post",
        "author": "AUTHOR_ID"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    Some("author".to_string()),
    None
).await?;
```

### Update Record

Update an existing record:

```rust
// Simple update
let record = pb.collection("posts").update(
    "RECORD_ID",
    json!({
        "title": "Updated Title",
        "status": "published"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Update with relations
pb.collection("posts").update(
    "RECORD_ID",
    json!({
        "categories+": "NEW_CATEGORY_ID",  // Append
        "tags-": "OLD_TAG_ID"              // Remove
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Update with file upload
let mut files = Vec::new();
files.push(FileAttachment {
    field: "image".to_string(),
    filename: "new_photo.jpg".to_string(),
    content_type: "image/jpeg".to_string(),
    data: new_image_bytes,
});

let record = pb.collection("posts").update(
    "RECORD_ID",
    json!({
        "title": "Updated Title"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;

// Update with expand
let record = pb.collection("posts").update(
    "RECORD_ID",
    json!({
        "title": "Updated"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    Some("author,categories".to_string()),
    None
).await?;
```

### Delete Record

Delete a record:

```rust
// Simple delete
pb.collection("posts").delete(
    "RECORD_ID",
    json!({}),
    HashMap::new(),
    HashMap::new()
).await?;

// Note: Returns 204 No Content on success
// Returns error if record doesn't exist or permission denied
```

## Filter Syntax

The filter parameter supports a powerful query syntax:

### Comparison Operators

```rust
// Equal
filter: Some(r#"status = "published""#.to_string())

// Not equal
filter: Some(r#"status != "draft""#.to_string())

// Greater than / Less than
filter: Some("views > 100".to_string())
filter: Some(r#"created < "2023-01-01""#.to_string())

// Greater/Less than or equal
filter: Some("age >= 18".to_string())
filter: Some("price <= 99.99".to_string())
```

### String Operators

```rust
// Contains (like)
filter: Some(r#"title ~ "javascript""#.to_string())
// Equivalent to: title LIKE "%javascript%"

// Not contains
filter: Some(r#"title !~ "deprecated""#.to_string())

// Exact match (case-sensitive)
filter: Some(r#"email = "user@example.com""#.to_string())
```

### Array Operators (for multiple relations/files)

```rust
// Any of / At least one
filter: Some(r#"tags.id ?= "TAG_ID""#.to_string())         // Any tag matches
filter: Some(r#"tags.name ?~ "important""#.to_string())    // Any tag name contains "important"

// All must match
filter: Some(r#"tags.id = "TAG_ID" && tags.id = "TAG_ID2""#.to_string())
```

### Logical Operators

```rust
// AND
filter: Some(r#"status = "published" && views > 100"#.to_string())

// OR
filter: Some(r#"status = "published" || status = "featured""#.to_string())

// Parentheses for grouping
filter: Some(r#"(status = "published" || featured = true) && views > 50"#.to_string())
```

### Special Identifiers

```rust
// Request context (only in API rules, not client filters)
// @request.auth.id, @request.query.*, etc.

// Collection joins
filter: Some(r#"@collection.users.email = "test@example.com""#.to_string())

// Record fields
filter: Some(r#"author.id = @request.auth.id"#.to_string())
```

### Comments

```rust
// Single-line comments are supported
filter: Some(r#"status = "published" // Only published posts"#.to_string())
```

## Sorting

Sort records using the `sort` parameter:

```rust
// Single field (ASC)
sort: Some("created".to_string())

// Single field (DESC)
sort: Some("-created".to_string())

// Multiple fields
sort: Some("-created,title".to_string())  // DESC by created, then ASC by title

// Supported fields
sort: Some("@random".to_string())         // Random order
sort: Some("@rowid".to_string())          // Internal row ID
sort: Some("id".to_string())              // Record ID
sort: Some("fieldName".to_string())       // Any collection field

// Relation field sorting
sort: Some("author.name".to_string())     // Sort by related author's name
```

## Field Selection

Control which fields are returned:

```rust
// Specific fields
fields: Some("id,title,content".to_string())

// All fields at level
fields: Some("*".to_string())

// Nested field selection
fields: Some("*,author.name,author.email".to_string())

// Excerpt modifier for text fields
fields: Some("*,content:excerpt(200,true)".to_string())
// Returns first 200 characters with ellipsis if truncated

// Combined
fields: Some("*,content:excerpt(200),author.name,author.email".to_string())
```

## Expanding Relations

Expand related records without additional API calls:

```rust
// Single relation
expand: Some("author".to_string())

// Multiple relations
expand: Some("author,categories,tags".to_string())

// Nested relations (up to 6 levels)
expand: Some("author.profile,categories.tags".to_string())

// Back-relations
expand: Some("comments_via_post.user".to_string())
```

See [Relations Documentation](./RELATIONS.md) for detailed information.

## Pagination Options

```rust
// Skip total count (faster queries)
let result = pb.collection("posts").get_list(
    1,
    50,
    true,  // skip_total: true - totalItems and totalPages will be -1
    HashMap::new(),
    HashMap::new(),
    Some(r#"status = "published""#.to_string()),
    None,
    None,
    None
).await?;

// Get Full List with batch processing
let all_posts = pb.collection("posts").get_full_list(
    200,  // batch size
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-created".to_string()),
    None,
    None
).await?;
// Processes in batches of 200 to avoid memory issues
```

## Batch Operations

Execute multiple operations in a single transaction:

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

// Create a batch
let batch = pb.create_batch();

// Add operations
batch.collection("posts").create(
    json!({
        "title": "Post 1",
        "author": "AUTHOR_ID"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
);

batch.collection("posts").create(
    json!({
        "title": "Post 2",
        "author": "AUTHOR_ID"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
);

batch.collection("tags").update(
    "TAG_ID",
    json!({
        "name": "Updated Tag"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
);

batch.collection("categories").delete(
    "CAT_ID",
    json!({}),
    HashMap::new(),
    HashMap::new()
);

// Send batch request
let results = batch.send().await?;

// Results is an array matching the order of operations
for (index, result) in results.iter().enumerate() {
    if let Some(status) = result.get("status").and_then(|s| s.as_u64()) {
        if status >= 400 {
            eprintln!("Operation {} failed: {:?}", index, result.get("body"));
        } else {
            println!("Operation {} succeeded: {:?}", index, result.get("body"));
        }
    }
}
```

**Note**: Batch operations must be enabled in Dashboard > Settings > Application.

## Authentication Actions

### List Auth Methods

Get available authentication methods for a collection:

```rust
let methods = pb.collection("users").list_auth_methods(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Password enabled: {}", methods["password"]["enabled"]);
println!("OAuth2 enabled: {}", methods["oauth2"]["enabled"]);
println!("OTP enabled: {}", methods["otp"]["enabled"]);
println!("MFA enabled: {}", methods["mfa"]["enabled"]);
```

### Auth with Password

```rust
let auth_data = pb.collection("users").auth_with_password(
    "user@example.com",  // username or email
    "password123",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Auth data is automatically stored in pb.auth_store()
println!("Is valid: {}", pb.auth_store().is_valid());
println!("Token: {}", pb.auth_store().token());
println!("User ID: {}", pb.auth_store().record()["id"]);

// Access the returned data
println!("Token: {}", auth_data["token"]);
println!("Record: {:?}", auth_data["record"]);

// With expand
let auth_data = pb.collection("users").auth_with_password(
    "user@example.com",
    "password123",
    HashMap::new(),
    HashMap::new(),
    Some("profile".to_string())
).await?;
```

### Auth with OAuth2

```rust
// Step 1: Get OAuth2 URL (usually done in UI)
let methods = pb.collection("users").list_auth_methods(
    HashMap::new(),
    HashMap::new()
).await?;

// Find provider
let providers = methods["oauth2"]["providers"].as_array().unwrap();
let provider = providers.iter().find(|p| p["name"] == "google");

if let Some(provider) = provider {
    // Redirect user to provider.authURL
    let auth_url = provider["authURL"].as_str().unwrap();
    
    // Step 2: After redirect, exchange code for token
    let auth_data = pb.collection("users").auth_with_oauth2_code(
        "google",                    // Provider name
        "AUTHORIZATION_CODE",        // From redirect URL
        provider["codeVerifier"].as_str().unwrap(),
        "https://yourapp.com/callback", // Redirect URL
        json!({                      // Optional data for new accounts
            "name": "John Doe"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;
}
```

### Auth with OTP (One-Time Password)

```rust
// Step 1: Request OTP
let otp_request = pb.collection("users").request_otp(
    "user@example.com",
    HashMap::new(),
    HashMap::new()
).await?;
// Returns: { "otpId": "..." }

// Step 2: User enters OTP from email
// Step 3: Authenticate with OTP
let auth_data = pb.collection("users").auth_with_otp(
    otp_request["otpId"].as_str().unwrap(),
    "123456",  // OTP code from email
    None,
    HashMap::new(),
    HashMap::new()
).await?;
```

### Auth Refresh

Refresh the current auth token and get updated user data:

```rust
// Refresh auth (useful on page reload)
let auth_data = pb.collection("users").auth_refresh(
    HashMap::new(),
    HashMap::new()
).await?;

// Check if still valid
if pb.auth_store().is_valid() {
    println!("User is authenticated");
} else {
    println!("Token expired or invalid");
}
```

### Email Verification

```rust
// Request verification email
pb.collection("users").request_verification(
    "user@example.com",
    HashMap::new(),
    HashMap::new()
).await?;

// Confirm verification (on verification page)
pb.collection("users").confirm_verification(
    "VERIFICATION_TOKEN",
    HashMap::new(),
    HashMap::new()
).await?;
```

### Password Reset

```rust
// Request password reset email
pb.collection("users").request_password_reset(
    "user@example.com",
    HashMap::new(),
    HashMap::new()
).await?;

// Confirm password reset (on reset page)
// Note: This invalidates all previous auth tokens
pb.collection("users").confirm_password_reset(
    "RESET_TOKEN",
    "newpassword123",
    "newpassword123",  // Confirm
    HashMap::new(),
    HashMap::new()
).await?;
```

### Email Change

```rust
// Must be authenticated first
pb.collection("users").auth_with_password(
    "user@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Request email change
pb.collection("users").request_email_change(
    "newemail@example.com",
    HashMap::new(),
    HashMap::new()
).await?;

// Confirm email change (on confirmation page)
// Note: This invalidates all previous auth tokens
pb.collection("users").confirm_email_change(
    "EMAIL_CHANGE_TOKEN",
    "currentpassword",
    HashMap::new(),
    HashMap::new()
).await?;
```

### Impersonate (Superuser Only)

Generate a token to authenticate as another user:

```rust
// Must be authenticated as superuser
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Impersonate a user
let impersonate_client = pb.collection("users").impersonate(
    "USER_ID",
    3600,  // Optional: token duration in seconds
    HashMap::new(),
    HashMap::new()
).await?;

// Use the impersonated client
let posts = impersonate_client.collection("posts").get_full_list(
    200,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;

// Access the token
println!("Token: {}", impersonate_client.auth_store().token());
println!("Record: {:?}", impersonate_client.auth_store().record());
```

## Complete Examples

### Example 1: Blog Post Search with Filters

```rust
async fn search_posts(
    pb: &BosBase,
    query: &str,
    category_id: Option<&str>,
    min_views: Option<i32>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let mut filter = format!(r#"title ~ "{}" || content ~ "{}""#, query, query);
    
    if let Some(cat_id) = category_id {
        filter.push_str(&format!(r#" && categories.id ?= "{}""#, cat_id));
    }
    
    if let Some(views) = min_views {
        filter.push_str(&format!(" && views >= {}", views));
    }
    
    let result = pb.collection("posts").get_list(
        1,
        20,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(filter),
        Some("-created".to_string()),
        Some("author,categories".to_string()),
        None
    ).await?;
    
    Ok(result["items"].as_array().unwrap().clone())
}
```

### Example 2: User Dashboard with Related Content

```rust
async fn get_user_dashboard(
    pb: &BosBase,
    user_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Get user's posts
    let posts = pb.collection("posts").get_list(
        1,
        10,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(format!(r#"author = "{}""#, user_id)),
        Some("-created".to_string()),
        Some("categories".to_string()),
        None
    ).await?;
    
    // Get user's comments
    let comments = pb.collection("comments").get_list(
        1,
        10,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(format!(r#"user = "{}""#, user_id)),
        Some("-created".to_string()),
        Some("post".to_string()),
        None
    ).await?;
    
    Ok(json!({
        "posts": posts["items"],
        "comments": comments["items"]
    }))
}
```

### Example 3: Advanced Filtering

```rust
// Complex filter example
let result = pb.collection("posts").get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"
        (status = "published" || featured = true) &&
        created >= "2023-01-01" &&
        (tags.id ?= "important" || categories.id = "news") &&
        views > 100 &&
        author.email != ""
    "#.to_string()),
    Some("-views,created".to_string()),
    Some("author.profile,tags,categories".to_string()),
    Some("*,content:excerpt(300),author.name,author.email".to_string())
).await?;
```

### Example 4: Batch Create Posts

```rust
async fn create_multiple_posts(
    pb: &BosBase,
    posts_data: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let batch = pb.create_batch();
    
    for post_data in posts_data {
        batch.collection("posts").create(
            post_data,
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
            None,
            None
        );
    }
    
    let results = batch.send().await?;
    
    // Check for failures
    let failures: Vec<_> = results.iter()
        .enumerate()
        .filter_map(|(index, result)| {
            if let Some(status) = result.get("status").and_then(|s| s.as_u64()) {
                if status >= 400 {
                    Some((index, result))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    if !failures.is_empty() {
        eprintln!("Some posts failed to create: {:?}", failures);
    }
    
    Ok(results.iter()
        .filter_map(|r| r.get("body").cloned())
        .collect())
}
```

### Example 5: Pagination Helper

```rust
async fn get_all_records_paginated(
    pb: &BosBase,
    collection_name: &str,
    filter: Option<String>,
    sort: Option<String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let mut all_records = Vec::new();
    let mut page = 1;
    let mut has_more = true;
    
    while has_more {
        let result = pb.collection(collection_name).get_list(
            page,
            500,
            true,  // skip_total for performance
            HashMap::new(),
            HashMap::new(),
            filter.clone(),
            sort.clone(),
            None,
            None
        ).await?;
        
        let items = result["items"].as_array().unwrap();
        all_records.extend_from_slice(items);
        
        has_more = items.len() == 500;
        page += 1;
    }
    
    Ok(all_records)
}
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.collection("posts").create(
    json!({
        "title": "My Post"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await {
    Ok(record) => {
        println!("Created record: {:?}", record);
    }
    Err(err) => {
        match err.status() {
            400 => {
                eprintln!("Validation errors: {:?}", err.data());
            }
            403 => {
                eprintln!("Access denied");
            }
            404 => {
                eprintln!("Collection or record not found");
            }
            _ => {
                eprintln!("Unexpected error: {:?}", err);
            }
        }
    }
}
```

## Best Practices

1. **Use Pagination**: Always use pagination for large datasets
2. **Skip Total When Possible**: Use `skip_total: true` for better performance when you don't need counts
3. **Batch Operations**: Use batch for multiple operations to reduce round trips
4. **Field Selection**: Only request fields you need to reduce payload size
5. **Expand Wisely**: Only expand relations you actually use
6. **Filter Before Sort**: Apply filters before sorting for better performance
7. **Cache Auth Tokens**: Auth tokens are automatically stored in `auth_store`, no need to manually cache
8. **Handle Errors**: Always handle authentication and permission errors gracefully

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection configuration
- [Relations](./RELATIONS.md) - Working with relations
- [API Rules and Filters](./API_RULES_AND_FILTERS.md) - Filter syntax details
- [Authentication](./AUTHENTICATION.md) - Detailed authentication guide
- [Files](./FILES.md) - File uploads and handling


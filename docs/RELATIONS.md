# Working with Relations - Rust SDK Documentation

## Overview

Relations allow you to link records between collections. BosBase supports both single and multiple relations, and provides powerful features for expanding related records and working with back-relations.

**Key Features:**
- Single and multiple relations
- Expand related records without additional requests
- Nested relation expansion (up to 6 levels)
- Back-relations for reverse lookups
- Field modifiers for append/prepend/remove operations

**Relation Field Types:**
- **Single Relation**: Links to one record (MaxSelect <= 1)
- **Multiple Relation**: Links to multiple records (MaxSelect > 1)

**Backend Behavior:**
- Relations are stored as record IDs or arrays of IDs
- Expand only includes relations the client can view (satisfies View API Rule)
- Back-relations use format: `collectionName_via_fieldName`
- Back-relation expand limited to 1000 records per field

## Setting Up Relations

### Creating a Relation Field

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let mut collection = pb.collections.get_one(
    "posts",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Add single relation field
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    fields.push(json!({
        "name": "user",
        "type": "relation",
        "options": {
            "collectionId": "users"
        },
        "maxSelect": 1,           // Single relation
        "required": true
    }));

    // Multiple relation field
    fields.push(json!({
        "name": "tags",
        "type": "relation",
        "options": {
            "collectionId": "tags"
        },
        "maxSelect": 10,          // Multiple relation (max 10)
        "minSelect": 1,           // Minimum 1 required
        "cascadeDelete": false    // Don't delete post when tags deleted
    }));
}

pb.collections.update(
    "posts",
    json!({
        "fields": collection["fields"]
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Creating Records with Relations

### Single Relation

```rust
// Create a post with a single user relation
let post = pb.collection("posts").create(
    json!({
        "title": "My Post",
        "user": "USER_ID"  // Single relation ID
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Multiple Relations

```rust
// Create a post with multiple tags
let post = pb.collection("posts").create(
    json!({
        "title": "My Post",
        "tags": ["TAG_ID1", "TAG_ID2", "TAG_ID3"]  // Array of IDs
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Mixed Relations

```rust
// Create a comment with both single and multiple relations
let comment = pb.collection("comments").create(
    json!({
        "message": "Great post!",
        "post": "POST_ID",        // Single relation
        "user": "USER_ID",        // Single relation
        "tags": ["TAG1", "TAG2"]  // Multiple relation
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Updating Relations

### Replace All Relations

```rust
// Replace all tags
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags": ["NEW_TAG1", "NEW_TAG2"]
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Append Relations (Using + Modifier)

```rust
// Append tags to existing ones
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags+": "NEW_TAG_ID"  // Append single tag
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Append multiple tags
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags+": ["TAG_ID1", "TAG_ID2"]  // Append multiple tags
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Prepend Relations (Using + Prefix)

```rust
// Prepend tags (tags will appear first)
pb.collection("posts").update(
    "POST_ID",
    json!({
        "+tags": "PRIORITY_TAG"  // Prepend single tag
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Prepend multiple tags
pb.collection("posts").update(
    "POST_ID",
    json!({
        "+tags": ["TAG1", "TAG2"]  // Prepend multiple tags
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Remove Relations (Using - Modifier)

```rust
// Remove single tag
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags-": "TAG_ID_TO_REMOVE"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Remove multiple tags
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags-": ["TAG1", "TAG2"]
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Complete Example

```rust
// Get existing post
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

println!("Current tags: {:?}", post["tags"]);

// Remove one tag, add two new ones
pb.collection("posts").update(
    "POST_ID",
    json!({
        "tags-": "tag1",           // Remove
        "tags+": ["tag3", "tag4"]  // Append
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

let updated = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
println!("Updated tags: {:?}", updated["tags"]);
```

## Expanding Relations

The `expand` parameter allows you to fetch related records in a single request, eliminating the need for multiple API calls.

### Basic Expand

```rust
// Get comment with expanded user
let comment = pb.collection("comments").get_one(
    "COMMENT_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user".to_string()),
    None
).await?;

// Access expanded relation
if let Some(expand) = comment.get("expand") {
    if let Some(user) = expand.get("user") {
        println!("User name: {}", user["name"]);
    }
}
// comment["user"] still contains the ID: "USER_ID"
```

### Expand Multiple Relations

```rust
// Expand multiple relations (comma-separated)
let comment = pb.collection("comments").get_one(
    "COMMENT_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user,post".to_string()),
    None
).await?;

if let Some(expand) = comment.get("expand") {
    if let Some(user) = expand.get("user") {
        println!("User name: {}", user["name"]);
    }
    if let Some(post) = expand.get("post") {
        println!("Post title: {}", post["title"]);
    }
}
```

### Nested Expand (Dot Notation)

You can expand nested relations up to 6 levels deep using dot notation:

```rust
// Expand post and its tags, and user
let comment = pb.collection("comments").get_one(
    "COMMENT_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user,post.tags".to_string()),
    None
).await?;

// Access nested expands
if let Some(expand) = comment.get("expand") {
    if let Some(post) = expand.get("post") {
        if let Some(post_expand) = post.get("expand") {
            if let Some(tags) = post_expand.get("tags").and_then(|t| t.as_array()) {
                println!("Post tags: {:?}", tags);
            }
        }
    }
}

// Expand even deeper
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user,comments_via_post.user".to_string()),
    None
).await?;
```

### Expand with List Requests

```rust
// List comments with expanded users
let comments = pb.collection("comments").get_list(
    1,
    20,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    Some("user".to_string()),
    None
).await?;

if let Some(items) = comments.get("items").and_then(|i| i.as_array()) {
    for comment in items {
        println!("Message: {}", comment["message"]);
        if let Some(expand) = comment.get("expand") {
            if let Some(user) = expand.get("user") {
                println!("User: {}", user["name"]);
            }
        }
    }
}
```

### Expand Single vs Multiple Relations

```rust
// Single relation - expand.user is an object
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user".to_string()),
    None
).await?;

if let Some(expand) = post.get("expand") {
    if let Some(user) = expand.get("user") {
        // user is an object
        println!("User: {:?}", user);
    }
}

// Multiple relation - expand.tags is an array
let post_with_tags = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("tags".to_string()),
    None
).await?;

if let Some(expand) = post_with_tags.get("expand") {
    if let Some(tags) = expand.get("tags").and_then(|t| t.as_array()) {
        // tags is an array
        println!("Tags: {:?}", tags);
    }
}
```

### Expand Permissions

**Important**: Only relations that satisfy the related collection's `viewRule` will be expanded. If you don't have permission to view a related record, it won't appear in the expand.

```rust
// If you don't have view permission for user, expand.user will be undefined
let comment = pb.collection("comments").get_one(
    "COMMENT_ID",
    HashMap::new(),
    HashMap::new(),
    Some("user".to_string()),
    None
).await?;

if let Some(expand) = comment.get("expand") {
    if let Some(user) = expand.get("user") {
        println!("User name: {}", user["name"]);
    } else {
        println!("User not accessible or not found");
    }
}
```

## Back-Relations

Back-relations allow you to query and expand records that reference the current record through a relation field.

### Back-Relation Syntax

The format is: `collectionName_via_fieldName`

- `collectionName`: The collection that contains the relation field
- `fieldName`: The name of the relation field that points to your record

### Example: Posts with Comments

```rust
// Get a post and expand all comments that reference it
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("comments_via_post".to_string()),
    None
).await?;

// comments_via_post is always an array (even if original field is single)
if let Some(expand) = post.get("expand") {
    if let Some(comments) = expand.get("comments_via_post").and_then(|c| c.as_array()) {
        println!("Comments: {:?}", comments);
    }
}
```

### Back-Relation with Nested Expand

```rust
// Get post with comments, and expand each comment's user
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("comments_via_post.user".to_string()),
    None
).await?;

// Access nested expands
if let Some(expand) = post.get("expand") {
    if let Some(comments) = expand.get("comments_via_post").and_then(|c| c.as_array()) {
        for comment in comments {
            println!("Message: {}", comment["message"]);
            if let Some(comment_expand) = comment.get("expand") {
                if let Some(user) = comment_expand.get("user") {
                    println!("User: {}", user["name"]);
                }
            }
        }
    }
}
```

### Filtering with Back-Relations

```rust
// List posts that have at least one comment containing "hello"
let posts = pb.collection("posts").get_list(
    1,
    20,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"comments_via_post.message ?~ "hello""#.to_string()),
    None,
    Some("comments_via_post.user".to_string()),
    None
).await?;

if let Some(items) = posts.get("items").and_then(|i| i.as_array()) {
    for post in items {
        println!("Post: {}", post["title"]);
        if let Some(expand) = post.get("expand") {
            if let Some(comments) = expand.get("comments_via_post").and_then(|c| c.as_array()) {
                for comment in comments {
                    println!("  - {}", comment["message"]);
                    if let Some(comment_expand) = comment.get("expand") {
                        if let Some(user) = comment_expand.get("user") {
                            println!("    by {}", user["name"]);
                        }
                    }
                }
            }
        }
    }
}
```

### Back-Relation Caveats

1. **Always Multiple**: Back-relations are always treated as arrays, even if the original relation field is single.

2. **1000 Record Limit**: Back-relation expand is limited to 1000 records per field. For larger datasets, use separate paginated requests:

```rust
// Instead of expanding all comments (if > 1000)
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Fetch comments separately with pagination
let comments = pb.collection("comments").get_list(
    1,
    100,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(format!(r#"post = "{}""#, post["id"])),
    Some("-created".to_string()),
    Some("user".to_string()),
    None
).await?;
```

## Complete Examples

### Example 1: Blog Post with Author and Tags

```rust
// Create a blog post with relations
let post = pb.collection("posts").create(
    json!({
        "title": "Getting Started with BosBase",
        "content": "Lorem ipsum...",
        "author": "AUTHOR_ID",           // Single relation
        "tags": ["tag1", "tag2", "tag3"] // Multiple relation
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Retrieve with all relations expanded
let full_post = pb.collection("posts").get_one(
    post["id"].as_str().unwrap(),
    HashMap::new(),
    HashMap::new(),
    Some("author,tags".to_string()),
    None
).await?;

println!("Title: {}", full_post["title"]);
if let Some(expand) = full_post.get("expand") {
    if let Some(author) = expand.get("author") {
        println!("Author: {}", author["name"]);
    }
    println!("Tags:");
    if let Some(tags) = expand.get("tags").and_then(|t| t.as_array()) {
        for tag in tags {
            println!("  - {}", tag["name"]);
        }
    }
}
```

### Example 2: Comment System with Nested Relations

```rust
// Create a comment on a post
let comment = pb.collection("comments").create(
    json!({
        "message": "Great article!",
        "post": "POST_ID",
        "user": "USER_ID"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

// Get post with all comments and their authors
let post = pb.collection("posts").get_one(
    "POST_ID",
    HashMap::new(),
    HashMap::new(),
    Some("author,comments_via_post.user".to_string()),
    None
).await?;

println!("Post: {}", post["title"]);
if let Some(expand) = post.get("expand") {
    if let Some(author) = expand.get("author") {
        println!("Author: {}", author["name"]);
    }
    if let Some(comments) = expand.get("comments_via_post").and_then(|c| c.as_array()) {
        println!("Comments ({}):", comments.len());
        for comment in comments {
            println!("  {}", comment["message"]);
            if let Some(comment_expand) = comment.get("expand") {
                if let Some(user) = comment_expand.get("user") {
                    println!("    by {}", user["name"]);
                }
            }
        }
    }
}
```

### Example 3: Dynamic Tag Management

```rust
struct PostManager {
    pb: BosBase,
}

impl PostManager {
    async fn add_tag(&self, post_id: &str, tag_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.collection("posts").update(
            post_id,
            json!({
                "tags+": tag_id
            }),
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
            None,
            None
        ).await?;
        Ok(())
    }

    async fn remove_tag(&self, post_id: &str, tag_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.collection("posts").update(
            post_id,
            json!({
                "tags-": tag_id
            }),
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
            None,
            None
        ).await?;
        Ok(())
    }

    async fn get_post_with_tags(&self, post_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(self.pb.collection("posts").get_one(
            post_id,
            HashMap::new(),
            HashMap::new(),
            Some("tags".to_string()),
            None
        ).await?)
    }
}

// Usage
let manager = PostManager { pb: pb.clone() };
manager.add_tag("POST_ID", "NEW_TAG_ID").await?;
let post = manager.get_post_with_tags("POST_ID").await?;
```

### Example 4: Filtering Posts by Tag

```rust
// Get all posts with a specific tag
let posts = pb.collection("posts").get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"tags.id ?= "TAG_ID""#.to_string()),
    Some("-created".to_string()),
    Some("author,tags".to_string()),
    None
).await?;

if let Some(items) = posts.get("items").and_then(|i| i.as_array()) {
    for post in items {
        println!("{}", post["title"]);
        if let Some(expand) = post.get("expand") {
            if let Some(author) = expand.get("author") {
                println!("  by {}", author["name"]);
            }
        }
    }
}
```

## Best Practices

1. **Use Expand Wisely**: Only expand relations you actually need to reduce response size and improve performance.

2. **Handle Missing Expands**: Always check if expand data exists before accessing:

```rust
if let Some(expand) = record.get("expand") {
    if let Some(user) = expand.get("user") {
        println!("User: {}", user["name"]);
    }
}
```

3. **Pagination for Large Back-Relations**: If you expect more than 1000 related records, fetch them separately with pagination.

4. **Cache Expansion**: Consider caching expanded data on the client side to reduce API calls.

5. **Error Handling**: Handle cases where related records might not be accessible due to API rules.

6. **Nested Limit**: Remember that nested expands are limited to 6 levels deep.

## Performance Considerations

- **Expand Cost**: Expanding relations doesn't require additional round trips, but increases response payload size
- **Back-Relation Limit**: The 1000 record limit for back-relations prevents extremely large responses
- **Permission Checks**: Each expanded relation is checked against the collection's `viewRule`
- **Nested Depth**: Limit nested expands to avoid performance issues (max 6 levels supported)

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection and field configuration
- [API Rules and Filters](./API_RULES_AND_FILTERS.md) - Filtering and querying related records


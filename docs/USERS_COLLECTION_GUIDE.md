# Built-in Users Collection Guide - Rust SDK Documentation

## Overview

This guide explains how to use the built-in `users` collection for authentication, registration, and API rules. **The `users` collection is automatically created when BosBase is initialized and does not need to be created manually.**

The `users` collection is a **built-in auth collection** that is automatically created when BosBase starts. It has:

- **Collection ID**: `_pb_users_auth_`
- **Collection Name**: `users`
- **Type**: `auth` (authentication collection)
- **Purpose**: User accounts, authentication, and authorization

**Important**: 
- ✅ **DO NOT** create a new `users` collection manually
- ✅ **DO** use the existing built-in `users` collection
- ✅ The collection already has proper API rules configured
- ✅ It supports password, OAuth2, and OTP authentication

## Getting Users Collection Information

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Get the users collection details
let users_collection = pb.collections.get_one(
    "users",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

println!("Collection ID: {}", users_collection["id"]);
println!("Collection Name: {}", users_collection["name"]);
println!("Collection Type: {}", users_collection["type"]);
println!("Fields: {:?}", users_collection["fields"]);
```

## User Registration

### Basic Registration

Users can register by creating a record in the `users` collection. The `createRule` is set to `""` (empty string), meaning **anyone can register**.

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Register a new user
let new_user = pb.collection("users").create(
    json!({
        "email": "user@example.com",
        "password": "securepassword123",
        "passwordConfirm": "securepassword123",
        "name": "John Doe"
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;

println!("User registered: {}", new_user["id"]);
println!("Email: {}", new_user["email"]);
```

## User Login/Authentication

### Password Authentication

```rust
// Login with email and password
let auth = pb.collection("users").auth_with_password(
    "user@example.com",
    "securepassword123",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

println!("Token: {}", auth["token"]);
println!("Record: {:?}", auth["record"]);
```

## API Rules and Filters with Users

The `users` collection comes with these default API rules:

```rust
// Default rules:
// {
//   "listRule": "id = @request.auth.id",    // Users can only list themselves
//   "viewRule": "id = @request.auth.id",   // Users can only view themselves
//   "createRule": "",                       // Anyone can register (public)
//   "updateRule": "id = @request.auth.id", // Users can only update themselves
//   "deleteRule": "id = @request.auth.id"  // Users can only delete themselves
// }
```

## Using Users with Other Collections

### Owner-Based Access

```rust
// Create a post with author relation
let post = pb.collection("posts").create(
    json!({
        "title": "My Post",
        "content": "Post content",
        "author": auth["record"]["id"]  // Link to authenticated user
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Filtering by User

```rust
// Get posts by current user
let my_posts = pb.collection("posts").get_list(
    1,
    20,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(format!(r#"author = "{}""#, auth["record"]["id"])),
    None,
    None,
    None
).await?;
```

## Complete Examples

### Example 1: User Registration and Login Flow

```rust
async fn register_and_login(
    pb: &BosBase,
    email: &str,
    password: &str,
    name: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Register
    let user = pb.collection("users").create(
        json!({
            "email": email,
            "password": password,
            "passwordConfirm": password,
            "name": name
        }),
        HashMap::new(),
        Vec::new(),
        HashMap::new(),
        None,
        None
    ).await?;

    // Login
    let auth = pb.collection("users").auth_with_password(
        email,
        password,
        HashMap::new(),
        HashMap::new(),
        None
    ).await?;

    Ok(auth)
}
```

## Related Documentation

- [Authentication](./AUTHENTICATION.md) - Complete authentication guide
- [API Rules and Filters](./API_RULES_AND_FILTERS.md) - Understanding API rules
- [Relations](./RELATIONS.md) - Linking users to other collections


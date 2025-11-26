# Schema Query API - Rust SDK Documentation

## Overview

The Schema Query API provides lightweight interfaces to retrieve collection field information without fetching full collection definitions. This is particularly useful for AI systems that need to understand the structure of collections and the overall system architecture.

**Key Features:**
- Get schema for a single collection by name or ID
- Get schemas for all collections in the system
- Lightweight response with only essential field information
- Support for all collection types (base, auth, view)
- Fast and efficient queries

**Backend Endpoints:**
- `GET /api/collections/{collection}/schema` - Get single collection schema
- `GET /api/collections/schemas` - Get all collection schemas

**Note**: All Schema Query API operations require superuser authentication.

## Authentication

All Schema Query API operations require superuser authentication:

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

## Get Single Collection Schema

Retrieves the schema (fields and types) for a single collection by name or ID.

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Get schema for a collection by name
let schema = pb.collections.get_schema(
    "demo1",
    HashMap::new(),
    HashMap::new()
).await?;

println!("Name: {}", schema["name"]);
println!("Type: {}", schema["type"]);
println!("Fields: {:?}", schema["fields"]);

// Iterate through fields
if let Some(fields) = schema.get("fields").and_then(|f| f.as_array()) {
    for field in fields {
        println!("{}: {}", field["name"], field["type"]);
    }
}
```

### Using Collection ID

```rust
// Get schema for a collection by ID
let schema = pb.collections.get_schema(
    "_pbc_base_123",
    HashMap::new(),
    HashMap::new()
).await?;

println!("Name: {}", schema["name"]);
```

### Handling Different Collection Types

```rust
// Base collection
let base_schema = pb.collections.get_schema(
    "demo1",
    HashMap::new(),
    HashMap::new()
).await?;
println!("Type: {}", base_schema["type"]);  // "base"

// Auth collection
let auth_schema = pb.collections.get_schema(
    "users",
    HashMap::new(),
    HashMap::new()
).await?;
println!("Type: {}", auth_schema["type"]);  // "auth"

// View collection
let view_schema = pb.collections.get_schema(
    "view1",
    HashMap::new(),
    HashMap::new()
).await?;
println!("Type: {}", view_schema["type"]);  // "view"
```

## Get All Collection Schemas

Retrieves the schema (fields and types) for all collections in the system.

### Basic Usage

```rust
// Get schemas for all collections
let result = pb.collections.get_all_schemas(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Collections: {:?}", result["collections"]);

// Iterate through all collections
if let Some(collections) = result.get("collections").and_then(|c| c.as_array()) {
    for collection in collections {
        println!("Collection: {} ({})", collection["name"], collection["type"]);
        if let Some(fields) = collection.get("fields").and_then(|f| f.as_array()) {
            println!("Fields: {}", fields.len());
        }
    }
}
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.collections.get_schema(
    "nonexistent",
    HashMap::new(),
    HashMap::new()
).await {
    Ok(schema) => {
        println!("Schema: {:?}", schema);
    }
    Err(err) => {
        match err.status() {
            404 => {
                println!("Collection not found");
            }
            _ => {
                eprintln!("Error: {:?}", err);
            }
        }
    }
}
```

## Related Documentation

- [Collection API](./COLLECTION_API.md) - Full collection management
- [Collections](./COLLECTIONS.md) - Collection types and fields


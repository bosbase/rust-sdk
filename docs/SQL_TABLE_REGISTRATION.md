# Register Existing SQL Tables - Rust SDK Documentation

## Overview

Use the SQL table helpers to expose existing tables (or run SQL to create them) and automatically generate REST collections. Both calls are **superuser-only**.

- `register_sql_tables(tables: Vec<String>)` – map existing tables to collections without running SQL.
- `import_sql_tables(tables: Vec<SqlTableDefinition>)` – optionally run SQL to create tables first, then register them. Returns `{ created, skipped }`.

## Requirements

- Authenticate with a `_superusers` token.
- Each table must contain a `TEXT` primary key column named `id`.
- Missing audit columns (`created`, `updated`, `createdBy`, `updatedBy`) are automatically added so the default API rules can be applied.
- Non-system columns are mapped by best effort (text, number, bool, date/time, JSON).

## Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let collections = pb.collections.register_sql_tables(
    vec!["projects", "accounts"],
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(collections_array) = collections.as_array() {
    for collection in collections_array {
        println!("Collection: {}", collection["name"]);
    }
}
```

## With Request Options

You can pass standard request options (headers, query params, etc.):

```rust
let mut headers = HashMap::new();
headers.insert("x-trace-id".to_string(), "reg-123".to_string());

let collections = pb.collections.register_sql_tables(
    vec!["legacy_orders"],
    HashMap::new(),
    headers
).await?;
```

## Create-or-register flow

`import_sql_tables()` accepts `SqlTableDefinition { name: string, sql?: string }` items, runs the SQL (if provided), and registers collections. Existing collection names are reported under `skipped`.

```rust
use serde_json::json;

let result = pb.collections.import_sql_tables(
    vec![
        json!({
            "name": "legacy_orders",
            "sql": r#"
                CREATE TABLE IF NOT EXISTS legacy_orders (
                    id TEXT PRIMARY KEY,
                    customer_email TEXT NOT NULL
                );
            "#
        }),
        json!({
            "name": "reporting_view"  // assumes table already exists
        })
    ],
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(created) = result.get("created").and_then(|c| c.as_array()) {
    for collection in created {
        println!("Created: {}", collection["name"]);
    }
}

if let Some(skipped) = result.get("skipped").and_then(|s| s.as_array()) {
    for name in skipped {
        println!("Skipped: {}", name);
    }
}
```

## What It Does

- Creates BosBase collection metadata for the provided tables.
- Generates REST endpoints for CRUD against those tables.
- Applies the standard default API rules (authenticated create; update/delete scoped to the creator).
- Ensures audit columns exist (`created`, `updated`, `createdBy`, `updatedBy`) and leaves all other existing SQL schema and data untouched; no further field mutations or table syncs are performed.
- Marks created collections with `externalTable: true` so you can distinguish them from regular BosBase-managed tables.

## Troubleshooting

- 400 error: ensure `id` exists as `TEXT PRIMARY KEY` and the table name is not system-reserved (no leading `_`).
- 401/403: confirm you are authenticated as a superuser.
- Default audit fields (`created`, `updated`, `createdBy`, `updatedBy`) are auto-added if they're missing so the default owner rules validate successfully.

## Related Documentation

- [Collection API](./COLLECTION_API.md) - Collection management
- [Collections](./COLLECTIONS.md) - Collection types and fields


# GraphQL - Rust SDK Documentation

## Overview

The GraphQL API allows you to query and mutate data using GraphQL syntax. All GraphQL operations require superuser authentication.

**Key Features:**
- Single-table queries
- Multi-table joins via expands
- Conditional queries with variables
- Create, update, and delete mutations
- Type-safe queries

**Backend Endpoints:**
- `POST /api/graphql` - Execute GraphQL queries and mutations

**Note**: The GraphQL endpoint is **superuser-only**. Authenticate as a superuser before calling GraphQL.

## Authentication

The GraphQL endpoint requires superuser authentication:

```rust
use bosbase::BosBase;

let pb = BosBase::new("http://127.0.0.1:8090");

// Authenticate as superuser
pb.collection("_superusers").auth_with_password(
    "admin@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;
```

## Single-table query

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let query = r#"
  query ActiveUsers($limit: Int!) {
    records(collection: "users", perPage: $limit, filter: "status = true") {
      items { id data }
    }
  }
"#;

let variables = json!({
    "limit": 5
});

let result = pb.graphql.query(
    query,
    variables,
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(data) = result.get("data") {
    if let Some(records) = data.get("records") {
        if let Some(items) = records.get("items").and_then(|i| i.as_array()) {
            for item in items {
                println!("User: {:?}", item);
            }
        }
    }
}
```

## Multi-table join via expands

```rust
let query = r#"
  query PostsWithAuthors {
    records(
      collection: "posts",
      expand: ["author", "author.profile"],
      sort: "-created"
    ) {
      items {
        id
        data  // expanded relations live under data.expand
      }
    }
  }
"#;

let result = pb.graphql.query(
    query,
    json!({}),
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(data) = result.get("data") {
    if let Some(records) = data.get("records") {
        if let Some(items) = records.get("items").and_then(|i| i.as_array()) {
            for post in items {
                println!("Post: {:?}", post);
                if let Some(post_data) = post.get("data") {
                    if let Some(expand) = post_data.get("expand") {
                        if let Some(author) = expand.get("author") {
                            println!("Author: {:?}", author);
                        }
                    }
                }
            }
        }
    }
}
```

## Conditional query with variables

```rust
let query = r#"
  query FilteredOrders($minTotal: Float!, $state: String!) {
    records(
      collection: "orders",
      filter: "total >= $minTotal && status = $state",
      sort: "created"
    ) {
      items { id data }
    }
  }
"#;

let variables = json!({
    "minTotal": 100.0,
    "state": "paid"
});

let result = pb.graphql.query(
    query,
    variables,
    HashMap::new(),
    HashMap::new()
).await?;
```

Use the `filter`, `sort`, `page`, `perPage`, and `expand` arguments to mirror REST list behavior while keeping query logic in GraphQL.

## Create a record

```rust
let mutation = r#"
  mutation CreatePost($data: JSON!) {
    createRecord(collection: "posts", data: $data, expand: ["author"]) {
      id
      data
    }
  }
"#;

let variables = json!({
    "data": {
        "title": "Hello",
        "author": "USER_ID"
    }
});

let result = pb.graphql.query(
    mutation,
    variables,
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(data) = result.get("data") {
    if let Some(create_record) = data.get("createRecord") {
        println!("Created record: {:?}", create_record);
    }
}
```

## Update a record

```rust
let mutation = r#"
  mutation UpdatePost($id: ID!, $data: JSON!) {
    updateRecord(collection: "posts", id: $id, data: $data) {
      id
      data
    }
  }
"#;

let variables = json!({
    "id": "POST_ID",
    "data": {
        "title": "Updated title"
    }
});

let result = pb.graphql.query(
    mutation,
    variables,
    HashMap::new(),
    HashMap::new()
).await?;
```

## Delete a record

```rust
let mutation = r#"
  mutation DeletePost($id: ID!) {
    deleteRecord(collection: "posts", id: $id)
  }
"#;

let variables = json!({
    "id": "POST_ID"
});

let result = pb.graphql.query(
    mutation,
    variables,
    HashMap::new(),
    HashMap::new()
).await?;
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.graphql.query(
    query,
    variables,
    HashMap::new(),
    HashMap::new()
).await {
    Ok(result) => {
        if let Some(errors) = result.get("errors") {
            eprintln!("GraphQL errors: {:?}", errors);
        } else {
            println!("Query successful: {:?}", result.get("data"));
        }
    }
    Err(err) => {
        eprintln!("GraphQL query failed: {:?}", err);
    }
}
```

## Best Practices

1. **Superuser Authentication**: Always authenticate as superuser before GraphQL queries
2. **Error Handling**: Check for both network errors and GraphQL errors in the response
3. **Variables**: Use variables for dynamic values instead of string interpolation
4. **Type Safety**: Use strongly-typed variables when possible
5. **Query Optimization**: Only request fields you need

## Related Documentation

- [API Records](./API_RECORDS.md) - REST API equivalent operations
- [Collections](./COLLECTIONS.md) - Collection configuration


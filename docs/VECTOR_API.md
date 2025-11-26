# Vector Database API - Rust SDK Documentation

## Overview

Vector database operations for semantic search, RAG (Retrieval-Augmented Generation), and AI applications.

> **Note**: Vector operations are currently implemented using sqlite-vec but are designed with abstraction in mind to support future vector database providers.

The Vector API provides a unified interface for working with vector embeddings, enabling you to:
- Store and search vector embeddings
- Perform similarity search
- Build RAG applications
- Create recommendation systems
- Enable semantic search capabilities

## Getting Started

```rust
use bosbase::BosBase;

let pb = BosBase::new("http://localhost:8090");

// Authenticate as superuser (vectors require superuser auth)
pb.admins().auth_with_password("admin@example.com", "password").await?;
```

## Collection Management

### Create Collection

Create a new vector collection with specified dimension and distance metric.

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Create collection with custom config
pb.vectors.create_collection(
    "documents",
    json!({
        "dimension": 384,      // Vector dimension (default: 384)
        "distance": "cosine"   // Distance metric: "cosine" (default), "l2", "dot"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// Minimal example (uses defaults)
pb.vectors.create_collection(
    "documents",
    json!({}),
    HashMap::new(),
    HashMap::new()
).await?;
```

**Parameters:**
- `name` (string): Collection name
- `config` (object, optional):
  - `dimension` (number, optional): Vector dimension. Default: 384
  - `distance` (string, optional): Distance metric. Default: "cosine"
  - Options: "cosine", "l2", "dot"

### List Collections

Get all available vector collections.

```rust
let collections = pb.vectors.list_collections(
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(collections_array) = collections.as_array() {
    for collection in collections_array {
        println!("{}: {} vectors", 
            collection["name"], 
            collection.get("count").and_then(|c| c.as_u64()).unwrap_or(0));
    }
}
```

### Update Collection

Update a vector collection configuration (distance metric and options).
Note: Collection name and dimension cannot be changed after creation.

```rust
// Change from cosine to L2
pb.vectors.update_collection(
    "documents",
    json!({
        "distance": "l2"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// Update with options
pb.vectors.update_collection(
    "documents",
    json!({
        "distance": "inner_product",
        "options": {
            "customOption": "value"
        }
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

### Delete Collection

Delete a vector collection and all its data.

```rust
pb.vectors.delete_collection(
    "documents",
    HashMap::new(),
    HashMap::new()
).await?;
```

**⚠️ Warning**: This permanently deletes the collection and all vectors in it!

## Document Operations

### Insert Document

Insert a single vector document.

```rust
// With custom ID
let result = pb.vectors.insert(
    json!({
        "id": "doc_001",
        "vector": [0.1, 0.2, 0.3, 0.4],
        "metadata": {
            "category": "tech",
            "tags": ["AI", "ML"]
        },
        "content": "Document about machine learning"
    }),
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Inserted: {}", result["id"]);

// Without ID (auto-generated)
let result2 = pb.vectors.insert(
    json!({
        "vector": [0.5, 0.6, 0.7, 0.8],
        "content": "Another document"
    }),
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

### Batch Insert

Insert multiple vector documents efficiently.

```rust
let result = pb.vectors.batch_insert(
    json!({
        "documents": [
            {
                "vector": [0.1, 0.2, 0.3],
                "metadata": { "cat": "A" },
                "content": "Doc A"
            },
            {
                "vector": [0.4, 0.5, 0.6],
                "metadata": { "cat": "B" },
                "content": "Doc B"
            },
            {
                "vector": [0.7, 0.8, 0.9],
                "metadata": { "cat": "A" },
                "content": "Doc C"
            }
        ],
        "skipDuplicates": true  // Skip documents with duplicate IDs
    }),
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Inserted: {}", result["insertedCount"]);
println!("Failed: {}", result["failedCount"]);
println!("IDs: {:?}", result["ids"]);
```

### Search Documents

Perform similarity search on vector documents.

```rust
let results = pb.vectors.search(
    json!({
        "queryVector": [0.1, 0.2, 0.3, 0.4],
        "limit": 10,
        "minScore": 0.7,
        "includeDistance": true,
        "includeContent": true
    }),
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(results_array) = results.as_array() {
    for result in results_array {
        println!("Score: {}, ID: {}", 
            result["score"], 
            result["document"]["id"]);
    }
}
```

### Get Document

Retrieve a single document by ID.

```rust
let document = pb.vectors.get(
    "doc_001",
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Document: {:?}", document);
```

### Update Document

Update an existing document.

```rust
pb.vectors.update(
    "doc_001",
    json!({
        "vector": [0.9, 0.8, 0.7, 0.6],
        "metadata": {
            "category": "updated"
        }
    }),
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

### Delete Document

Delete a document by ID.

```rust
pb.vectors.delete(
    "doc_001",
    json!({
        "collection": "documents"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## Complete Examples

### Example 1: RAG Application

```rust
async fn setup_rag_collection(
    pb: &BosBase,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create collection
    pb.vectors.create_collection(
        "knowledge-base",
        json!({
            "dimension": 384,
            "distance": "cosine"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    // Insert documents
    pb.vectors.batch_insert(
        json!({
            "documents": [
                {
                    "vector": generate_embedding("What is Rust?"),
                    "content": "Rust is a systems programming language...",
                    "metadata": {
                        "topic": "programming",
                        "language": "rust"
                    }
                },
                {
                    "vector": generate_embedding("What is Python?"),
                    "content": "Python is a high-level programming language...",
                    "metadata": {
                        "topic": "programming",
                        "language": "python"
                    }
                }
            ]
        }),
        json!({
            "collection": "knowledge-base"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    Ok(())
}

async fn search_knowledge_base(
    pb: &BosBase,
    query: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query_vector = generate_embedding(query);
    
    let results = pb.vectors.search(
        json!({
            "queryVector": query_vector,
            "limit": 5,
            "minScore": 0.7,
            "includeContent": true
        }),
        json!({
            "collection": "knowledge-base"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    Ok(results.as_array().unwrap().clone())
}
```

## Best Practices

1. **Dimension Consistency**: Use the same dimension for all vectors in a collection
2. **Distance Metric**: Choose the appropriate distance metric for your use case
3. **Batch Operations**: Use batch insert for better performance
4. **Metadata**: Use metadata for filtering and additional context
5. **Score Thresholds**: Set appropriate minScore thresholds for search results

## Related Documentation

- [LLM Documents](./LLM_DOCUMENTS.md) - Document management for LLM workflows
- [LangChaingo API](./LANGCHAINGO_API.md) - RAG workflows with LangChain


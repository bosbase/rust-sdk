# LLM Document API - Rust SDK Documentation

## Overview

The `LLMDocumentService` wraps the `/api/llm-documents` endpoints that are backed by the embedded chromem-go vector store (persisted in rqlite). Each document contains text content, optional metadata and an embedding vector that can be queried with semantic search.

## Getting Started

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Create a logical namespace for your documents
pb.llm_documents.create_collection(
    "knowledge-base",
    json!({
        "domain": "internal"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## Insert Documents

```rust
let doc = pb.llm_documents.insert(
    json!({
        "content": "Leaves are green because chlorophyll absorbs red and blue light.",
        "metadata": {
            "topic": "biology"
        }
    }),
    json!({
        "collection": "knowledge-base"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

pb.llm_documents.insert(
    json!({
        "id": "sky",
        "content": "The sky is blue because of Rayleigh scattering.",
        "metadata": {
            "topic": "physics"
        }
    }),
    json!({
        "collection": "knowledge-base"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## Query Documents

```rust
let result = pb.llm_documents.query(
    json!({
        "queryText": "Why is the sky blue?",
        "limit": 3,
        "where": {
            "topic": "physics"
        }
    }),
    json!({
        "collection": "knowledge-base"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
    for match_result in results {
        println!("ID: {}, Similarity: {}", 
            match_result["id"], 
            match_result["similarity"]);
    }
}
```

## Manage Documents

```rust
// Update a document
pb.llm_documents.update(
    "sky",
    json!({
        "metadata": {
            "topic": "physics",
            "reviewed": "true"
        }
    }),
    json!({
        "collection": "knowledge-base"
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// List documents with pagination
let page = pb.llm_documents.list(
    json!({
        "collection": "knowledge-base",
        "page": 1,
        "perPage": 25
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// Delete unwanted entries
pb.llm_documents.delete(
    "sky",
    json!({
        "collection": "knowledge-base"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## HTTP Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET /api/llm-documents/collections` | List collections |
| `POST /api/llm-documents/collections/{name}` | Create collection |
| `DELETE /api/llm-documents/collections/{name}` | Delete collection |
| `GET /api/llm-documents/{collection}` | List documents |
| `POST /api/llm-documents/{collection}` | Insert document |
| `GET /api/llm-documents/{collection}/{id}` | Fetch document |
| `PATCH /api/llm-documents/{collection}/{id}` | Update document |
| `DELETE /api/llm-documents/{collection}/{id}` | Delete document |
| `POST /api/llm-documents/{collection}/documents/query` | Query by semantic similarity |

## Complete Examples

### Example 1: Knowledge Base Setup

```rust
async fn setup_knowledge_base(
    pb: &BosBase,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create collection
    pb.llm_documents.create_collection(
        "knowledge-base",
        json!({
            "domain": "internal"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    // Insert documents
    pb.llm_documents.insert(
        json!({
            "content": "Rust is a systems programming language focused on safety and performance.",
            "metadata": {
                "topic": "programming",
                "language": "rust"
            }
        }),
        json!({
            "collection": "knowledge-base"
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;

    Ok(())
}
```

## Related Documentation

- [Vector API](./VECTOR_API.md) - Vector database operations
- [LangChaingo API](./LANGCHAINGO_API.md) - RAG workflows with LangChain


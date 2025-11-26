# LangChaingo API - Rust SDK Documentation

## Overview

BosBase exposes the `/api/langchaingo` endpoints so you can run LangChainGo powered workflows without leaving the platform. The Rust SDK wraps these endpoints with the `pb.langchaingo` service.

The service exposes four high-level methods:

| Method | HTTP Endpoint | Description |
| --- | --- | --- |
| `pb.langchaingo.completions()` | `POST /api/langchaingo/completions` | Runs a chat/completion call using the configured LLM provider. |
| `pb.langchaingo.rag()` | `POST /api/langchaingo/rag` | Runs a retrieval-augmented generation pass over an `llmDocuments` collection. |
| `pb.langchaingo.query_documents()` | `POST /api/langchaingo/documents/query` | Asks an OpenAI-backed chain to answer questions over `llmDocuments` and optionally return matched sources. |
| `pb.langchaingo.sql()` | `POST /api/langchaingo/sql` | Lets OpenAI draft and execute SQL against your BosBase database, then returns the results. |

Each method accepts an optional `model` block:

```rust
// Model configuration structure
{
    "provider": "openai" | "ollama" | string,
    "model": string,
    "apiKey": string,  // optional
    "baseUrl": string  // optional
}
```

If you omit the `model` section, BosBase defaults to `provider: "openai"` and `model: "gpt-4o-mini"` with credentials read from the server environment. Passing an `apiKey` lets you override server defaults on a per-request basis.

## Text + Chat Completions

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

let completion = pb.langchaingo.completions(
    json!({
        "model": {
            "provider": "openai",
            "model": "gpt-4o-mini"
        },
        "messages": [
            {
                "role": "system",
                "content": "Answer in one sentence."
            },
            {
                "role": "user",
                "content": "Explain Rayleigh scattering."
            }
        ],
        "temperature": 0.2
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Content: {}", completion["content"]);
```

The completion response mirrors the LangChainGo `ContentResponse` shape, so you can inspect the `functionCall`, `toolCalls`, or `generationInfo` fields when you need more than plain text.

## Retrieval-Augmented Generation (RAG)

Pair the LangChaingo endpoints with the `/api/llm-documents` store to build RAG workflows. The backend automatically uses the chromem-go collection configured for the target LLM collection.

```rust
let answer = pb.langchaingo.rag(
    json!({
        "collection": "knowledge-base",
        "question": "Why is the sky blue?",
        "topK": 4,
        "returnSources": true,
        "filters": {
            "where": {
                "topic": "physics"
            }
        }
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Answer: {}", answer["answer"]);
if let Some(sources) = answer.get("sources").and_then(|s| s.as_array()) {
    for source in sources {
        println!("Score: {}, Title: {:?}", 
            source.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
            source.get("metadata").and_then(|m| m.get("title")));
    }
}
```

Set `promptTemplate` when you want to control how the retrieved context is stuffed into the answer prompt:

```rust
pb.langchaingo.rag(
    json!({
        "collection": "knowledge-base",
        "question": "Summarize the explanation below in 2 sentences.",
        "promptTemplate": "Context:\n{{.context}}\n\nQuestion: {{.question}}\nSummary:"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## LLM Document Queries

> **Note**: This interface is only available to superusers.

When you want to pose a question to a specific `llmDocuments` collection and have LangChaingo+OpenAI synthesize an answer, use `query_documents`. It mirrors the RAG arguments but takes a `query` field:

```rust
let response = pb.langchaingo.query_documents(
    json!({
        "collection": "knowledge-base",
        "query": "List three bullet points about Rayleigh scattering.",
        "topK": 3,
        "returnSources": true
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("Answer: {}", response["answer"]);
println!("Sources: {:?}", response["sources"]);
```

## SQL Generation + Execution

> **Important Notes**:
> - This interface is only available to superusers. Requests authenticated with regular `users` tokens return a `401 Unauthorized`.
> - It is recommended to execute query statements (SELECT) only.
> - **Do not use this interface for adding or modifying table structures.** Collection interfaces should be used instead for managing database schema.
> - Directly using this interface for initializing table structures and adding or modifying database tables will cause errors that prevent the automatic generation of APIs.

Superuser tokens (`_superusers` records) can ask LangChaingo to have OpenAI propose a SQL statement, execute it, and return both the generated SQL and execution output.

```rust
let result = pb.langchaingo.sql(
    json!({
        "query": "Add a demo project row if it doesn't exist, then list the 5 most recent projects.",
        "tables": ["projects"],  // optional hint to limit which tables the model sees
        "topK": 5
    }),
    HashMap::new(),
    HashMap::new()
).await?;

println!("SQL: {}", result["sql"]);
println!("Answer: {}", result["answer"]);
println!("Columns: {:?}", result["columns"]);
println!("Rows: {:?}", result["rows"]);
```

Use `tables` to restrict which table definitions and sample rows are passed to the model, and `topK` to control how many rows the model should target when building queries. You can also pass the optional `model` block described above to override the default OpenAI model or key for this call.

## Related Documentation

- [LLM Documents](./LLM_DOCUMENTS.md) - Document management for LLM workflows
- [Vector API](./VECTOR_API.md) - Vector database operations


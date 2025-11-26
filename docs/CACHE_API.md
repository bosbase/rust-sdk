# Cache API - Rust SDK Documentation

## Overview

BosBase caches combine in-memory [FreeCache](https://github.com/coocood/freecache) storage with persistent database copies. Each cache instance is safe to use in single-node or multi-node (cluster) mode: nodes read from FreeCache first, fall back to the database if an item is missing or expired, and then reload FreeCache automatically.

The Rust SDK exposes the cache endpoints through `pb.caches`. Typical use cases include:

- Caching AI prompts/responses that must survive restarts.
- Quickly sharing feature flags and configuration between workers.
- Preloading expensive vector search results for short periods.

> **Timeouts & TTLs:** Each cache defines a default TTL (in seconds). Individual entries may provide their own `ttlSeconds`. A value of `0` keeps the entry until it is manually deleted.

## List available caches

The `list()` function allows you to query and retrieve all currently available caches, including their names and capacities. This is particularly useful for AI systems to discover existing caches before creating new ones, avoiding duplicate cache creation.

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("root@example.com", "hunter2").await?;

// Query all available caches
let caches = pb.caches.list(
    HashMap::new(),
    HashMap::new()
).await?;

// Each cache object contains:
// - name: string - The cache identifier
// - sizeBytes: number - The cache capacity in bytes
// - defaultTTLSeconds: number - Default expiration time
// - readTimeoutMs: number - Read timeout in milliseconds
// - created: string - Creation timestamp (RFC3339)
// - updated: string - Last update timestamp (RFC3339)

// Example: Find a cache by name and check its capacity
if let Some(cache_array) = caches.as_array() {
    if let Some(target_cache) = cache_array.iter().find(|c| c["name"] == "ai-session") {
        println!("Cache \"{}\" has capacity of {} bytes", 
            target_cache["name"], 
            target_cache["sizeBytes"]);
    } else {
        println!("Cache not found, create a new one if needed");
    }
}
```

## Manage cache configurations

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("root@example.com", "hunter2").await?;

// List all available caches
let caches = pb.caches.list(
    HashMap::new(),
    HashMap::new()
).await?;

// Find an existing cache by name
let existing_cache = caches.as_array()
    .and_then(|arr| arr.iter().find(|c| c["name"] == "ai-session"));

if let Some(cache) = existing_cache {
    println!("Found cache \"{}\" with capacity {} bytes", 
        cache["name"], 
        cache["sizeBytes"]);
} else {
    // Create a new cache only if it doesn't exist
    pb.caches.create(
        json!({
            "name": "ai-session",
            "sizeBytes": 64 * 1024 * 1024,
            "defaultTTLSeconds": 300,
            "readTimeoutMs": 25
        }),
        HashMap::new(),
        HashMap::new()
    ).await?;
}

// Update limits later (eg. shrink TTL to 2 minutes)
pb.caches.update(
    "ai-session",
    json!({
        "defaultTTLSeconds": 120
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// Delete the cache (DB rows + FreeCache)
pb.caches.delete(
    "ai-session",
    HashMap::new(),
    HashMap::new()
).await?;
```

Field reference:

| Field | Description |
|-------|-------------|
| `sizeBytes` | Approximate FreeCache size. Values too small (<512KB) or too large (>512MB) are clamped. |
| `defaultTTLSeconds` | Default expiration for entries. `0` means no expiration. |
| `readTimeoutMs` | Optional lock timeout while reading FreeCache. When exceeded, the value is fetched from the database instead. |

## Work with cache entries

```rust
use serde_json::json;

// Store an object in cache. The same payload is serialized into the DB.
pb.caches.set_entry(
    "ai-session",
    "dialog:42",
    json!({
        "prompt": "describe Saturn",
        "embedding": [/* vector */]
    }),
    Some(90),  // per-entry TTL in seconds
    HashMap::new(),
    HashMap::new()
).await?;

// Read from cache. `source` indicates where the hit came from.
let entry = pb.caches.get_entry(
    "ai-session",
    "dialog:42",
    HashMap::new(),
    HashMap::new()
).await?;

println!("Source: {}", entry["source"]);   // "cache" or "database"
if let Some(expires_at) = entry.get("expiresAt") {
    println!("Expires at: {}", expires_at);
}

// Renew an entry's TTL without changing its value.
// This extends the expiration time by the specified TTL (or uses the cache's default TTL if omitted).
let renewed = pb.caches.renew_entry(
    "ai-session",
    "dialog:42",
    Some(120),  // extend by 120 seconds
    HashMap::new(),
    HashMap::new()
).await?;
println!("New expiration: {:?}", renewed.get("expiresAt"));

// Delete an entry
pb.caches.delete_entry(
    "ai-session",
    "dialog:42",
    HashMap::new(),
    HashMap::new()
).await?;
```

### Cluster-aware behaviour

1. **Write-through persistence** – every `setEntry` writes to FreeCache and the `_cache_entries` table so other nodes (or a restarted node) can immediately reload values.
2. **Read path** – FreeCache is consulted first. If a lock cannot be acquired within `readTimeoutMs` or if the entry is missing/expired, BosBase queries the database copy and repopulates FreeCache in the background.
3. **Automatic cleanup** – expired entries are ignored and removed from the database when fetched, preventing stale data across nodes.

Use caches whenever you need fast, transient data that must still be recoverable or shareable across BosBase nodes.

## Complete Examples

### Example 1: AI Session Cache

```rust
struct AISessionCache {
    pb: BosBase,
    cache_name: String,
}

impl AISessionCache {
    async fn store_session(
        &self,
        session_id: &str,
        prompt: &str,
        response: &str,
        ttl_seconds: Option<u64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.caches.set_entry(
            &self.cache_name,
            &format!("session:{}", session_id),
            json!({
                "prompt": prompt,
                "response": response,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            ttl_seconds,
            HashMap::new(),
            HashMap::new()
        ).await?;
        Ok(())
    }

    async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let entry = self.pb.caches.get_entry(
            &self.cache_name,
            &format!("session:{}", session_id),
            HashMap::new(),
            HashMap::new()
        ).await?;
        
        if let Some(value) = entry.get("value") {
            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    async fn extend_session(
        &self,
        session_id: &str,
        additional_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.caches.renew_entry(
            &self.cache_name,
            &format!("session:{}", session_id),
            Some(additional_seconds),
            HashMap::new(),
            HashMap::new()
        ).await?;
        Ok(())
    }
}
```

### Example 2: Feature Flag Cache

```rust
async fn get_feature_flag(
    pb: &BosBase,
    flag_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let entry = pb.caches.get_entry(
        "feature-flags",
        flag_name,
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    if let Some(value) = entry.get("value") {
        if let Some(enabled) = value.get("enabled").and_then(|v| v.as_bool()) {
            return Ok(enabled);
        }
    }
    
    // Default to false if not found
    Ok(false)
}

async fn set_feature_flag(
    pb: &BosBase,
    flag_name: &str,
    enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    pb.caches.set_entry(
        "feature-flags",
        flag_name,
        json!({
            "enabled": enabled,
            "updated": chrono::Utc::now().to_rfc3339()
        }),
        Some(0),  // No expiration
        HashMap::new(),
        HashMap::new()
    ).await?;
    Ok(())
}
```

## Best Practices

1. **Cache Discovery**: Always list existing caches before creating new ones
2. **TTL Management**: Set appropriate TTLs based on data freshness requirements
3. **Error Handling**: Handle cache misses gracefully
4. **Cluster Awareness**: Remember that caches are shared across nodes
5. **Size Limits**: Be mindful of cache size limits when storing large values

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection management
- [API Records](./API_RECORDS.md) - Record operations


# Health API - Rust SDK Documentation

## Overview

The Health API provides a simple endpoint to check the health status of the server. It returns basic health information and, when authenticated as a superuser, provides additional diagnostic information about the server state.

**Key Features:**
- No authentication required for basic health check
- Superuser authentication provides additional diagnostic data
- Lightweight endpoint for monitoring and health checks
- Supports both GET and HEAD methods

**Backend Endpoints:**
- `GET /api/health` - Check health status
- `HEAD /api/health` - Check health status (HEAD method)

**Note**: The health endpoint is publicly accessible, but superuser authentication provides additional information.

## Authentication

Basic health checks do not require authentication:

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Basic health check (no auth required)
let health = pb.health.check(
    HashMap::new(),
    HashMap::new()
).await?;
```

For additional diagnostic information, authenticate as a superuser:

```rust
// Authenticate as superuser for extended health data
pb.collection("_superusers").auth_with_password(
    "admin@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

let health = pb.health.check(
    HashMap::new(),
    HashMap::new()
).await?;
```

## Health Check Response Structure

### Basic Response (Guest/Regular User)

```rust
// Response structure:
// {
//   "code": 200,
//   "message": "API is healthy.",
//   "data": {}
// }
```

### Superuser Response

```rust
// Response structure:
// {
//   "code": 200,
//   "message": "API is healthy.",
//   "data": {
//     "canBackup": boolean,           // Whether backup operations are allowed
//     "realIP": string,               // Real IP address of the client
//     "requireS3": boolean,           // Whether S3 storage is required
//     "possibleProxyHeader": string   // Detected proxy header (if behind reverse proxy)
//   }
// }
```

## Check Health Status

Returns the health status of the API server.

### Basic Usage

```rust
// Simple health check
let health = pb.health.check(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Message: {}", health["message"]);
println!("Code: {}", health["code"]);
```

### With Superuser Authentication

```rust
// Authenticate as superuser first
pb.collection("_superusers").auth_with_password(
    "admin@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Get extended health information
let health = pb.health.check(
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(data) = health.get("data").and_then(|d| d.as_object()) {
    println!("Can backup: {:?}", data.get("canBackup"));
    println!("Real IP: {:?}", data.get("realIP"));
    println!("Require S3: {:?}", data.get("requireS3"));
    println!("Proxy header: {:?}", data.get("possibleProxyHeader"));
}
```

## Response Fields

### Common Fields (All Users)

| Field | Type | Description |
|-------|------|-------------|
| `code` | number | HTTP status code (always 200 for healthy server) |
| `message` | string | Health status message ("API is healthy.") |
| `data` | object | Health data (empty for non-superusers, populated for superusers) |

### Superuser-Only Fields (in `data`)

| Field | Type | Description |
|-------|------|-------------|
| `canBackup` | boolean | `true` if backup/restore operations can be performed, `false` if a backup/restore is currently in progress |
| `realIP` | string | The real IP address of the client (useful when behind proxies) |
| `requireS3` | boolean | `true` if S3 storage is required (local fallback disabled), `false` otherwise |
| `possibleProxyHeader` | string | Detected proxy header name (e.g., "X-Forwarded-For", "CF-Connecting-IP") if the server appears to be behind a reverse proxy, empty string otherwise |

## Use Cases

### 1. Basic Health Monitoring

```rust
async fn check_server_health(
    pb: &BosBase,
) -> Result<bool, Box<dyn std::error::Error>> {
    match pb.health.check(HashMap::new(), HashMap::new()).await {
        Ok(health) => {
            if health["code"].as_u64() == Some(200) 
                && health["message"].as_str() == Some("API is healthy.") {
                println!("✓ Server is healthy");
                Ok(true)
            } else {
                println!("✗ Server health check failed");
                Ok(false)
            }
        }
        Err(err) => {
            eprintln!("✗ Health check error: {:?}", err);
            Ok(false)
        }
    }
}
```

### 2. Backup Readiness Check

```rust
async fn can_perform_backup(
    pb: &BosBase,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Authenticate as superuser
    pb.collection("_superusers").auth_with_password(
        "admin@example.com",
        "password",
        HashMap::new(),
        HashMap::new(),
        None
    ).await?;
    
    let health = pb.health.check(
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    if let Some(data) = health.get("data") {
        if let Some(can_backup) = data.get("canBackup") {
            if can_backup.as_bool() == Some(false) {
                println!("⚠️ Backup operation is currently in progress");
                return Ok(false);
            }
        }
    }
    
    println!("✓ Backup operations are allowed");
    Ok(true)
}

// Use before creating backups
if can_perform_backup(&pb).await? {
    pb.backups.create("backup.zip", HashMap::new(), HashMap::new()).await?;
}
```

### 3. Monitoring Dashboard

```rust
struct HealthMonitor {
    pb: BosBase,
    is_superuser: bool,
}

impl HealthMonitor {
    async fn authenticate_as_superuser(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match self.pb.collection("_superusers").auth_with_password(
            email,
            password,
            HashMap::new(),
            HashMap::new(),
            None
        ).await {
            Ok(_) => {
                self.is_superuser = true;
                Ok(true)
            }
            Err(err) => {
                eprintln!("Superuser authentication failed: {:?}", err);
                Ok(false)
            }
        }
    }

    async fn get_health_status(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let health = self.pb.health.check(
            HashMap::new(),
            HashMap::new()
        ).await?;
        
        let mut status = json!({
            "healthy": health["code"].as_u64() == Some(200),
            "message": health["message"],
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        if self.is_superuser {
            if let Some(data) = health.get("data") {
                status["diagnostics"] = json!({
                    "canBackup": data.get("canBackup"),
                    "realIP": data.get("realIP"),
                    "requireS3": data.get("requireS3"),
                    "behindProxy": data.get("possibleProxyHeader").is_some(),
                    "proxyHeader": data.get("possibleProxyHeader")
                });
            }
        }
        
        Ok(status)
    }
}
```

## Best Practices

1. **Monitoring**: Use health checks for regular monitoring (e.g., every 30-60 seconds)
2. **Load Balancers**: Configure load balancers to use the health endpoint for health checks
3. **Pre-flight Checks**: Check `canBackup` before initiating backup operations
4. **Error Handling**: Always handle errors gracefully as the server may be down
5. **Rate Limiting**: Don't poll the health endpoint too frequently (avoid spamming)
6. **Caching**: Consider caching health check results for a few seconds to reduce load
7. **Logging**: Log health check results for troubleshooting and monitoring
8. **Alerting**: Set up alerts for consecutive health check failures
9. **Superuser Auth**: Only authenticate as superuser when you need diagnostic information
10. **Proxy Configuration**: Use `possibleProxyHeader` to detect and configure reverse proxy settings

## Response Codes

| Code | Meaning |
|------|---------|
| 200 | Server is healthy |
| Network Error | Server is unreachable or down |

## Limitations

- **No Detailed Metrics**: The health endpoint does not provide detailed performance metrics
- **Basic Status Only**: Returns basic status, not detailed system information
- **Superuser Required**: Extended diagnostics require superuser authentication
- **No Historical Data**: Only returns current status, no historical health data

## Related Documentation

- [Backups API](./BACKUPS_API.md) - Using `canBackup` to check backup readiness
- [Authentication](./AUTHENTICATION.md) - Superuser authentication


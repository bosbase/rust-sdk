# Logs API - Rust SDK Documentation

## Overview

The Logs API provides endpoints for viewing and analyzing application logs. All operations require superuser authentication and allow you to query request logs, filter by various criteria, and get aggregated statistics.

**Key Features:**
- List and paginate logs
- View individual log entries
- Filter logs by status, URL, method, IP, etc.
- Sort logs by various fields
- Get hourly aggregated statistics
- Filter statistics by criteria

**Backend Endpoints:**
- `GET /api/logs` - List logs
- `GET /api/logs/{id}` - View log
- `GET /api/logs/stats` - Get statistics

**Note**: All Logs API operations require superuser authentication.

## Authentication

All Logs API operations require superuser authentication:

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

## List Logs

Returns a paginated list of logs with support for filtering and sorting.

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Basic list
let result = pb.logs.get_list(
    1,
    30,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;

println!("Page: {}", result["page"]);
println!("Per Page: {}", result["perPage"]);
println!("Total Items: {}", result["totalItems"]);
println!("Items: {:?}", result["items"]);
```

### Log Entry Structure

Each log entry contains:

```rust
// {
//   "id": "ai5z3aoed6809au",
//   "created": "2024-10-27 09:28:19.524Z",
//   "level": 0,
//   "message": "GET /api/collections/posts/records",
//   "data": {
//     "auth": "_superusers",
//     "execTime": 2.392327,
//     "method": "GET",
//     "referer": "http://localhost:8090/_/",
//     "remoteIP": "127.0.0.1",
//     "status": 200,
//     "type": "request",
//     "url": "/api/collections/posts/records?page=1",
//     "userAgent": "Mozilla/5.0...",
//     "userIP": "127.0.0.1"
//   }
// }
```

### Filtering Logs

```rust
// Filter by HTTP status code
let error_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some("data.status >= 400".to_string()),
    None,
    None,
    None
).await?;

// Filter by method
let get_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.method = "GET""#.to_string()),
    None,
    None,
    None
).await?;

// Filter by URL pattern
let api_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.url ~ "/api/""#.to_string()),
    None,
    None,
    None
).await?;

// Filter by IP address
let ip_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.remoteIP = "127.0.0.1""#.to_string()),
    None,
    None,
    None
).await?;

// Filter by execution time (slow requests)
let slow_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some("data.execTime > 1.0".to_string()),
    None,
    None,
    None
).await?;

// Filter by log level
let error_level_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some("level > 0".to_string()),
    None,
    None,
    None
).await?;

// Filter by date range
let recent_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"created >= "2024-10-27 00:00:00""#.to_string()),
    None,
    None,
    None
).await?;
```

### Complex Filters

```rust
// Multiple conditions
let complex_filter = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.status >= 400 && data.method = "POST" && data.execTime > 0.5"#.to_string()),
    None,
    None,
    None
).await?;

// Exclude superuser requests
let user_logs = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.auth != "_superusers""#.to_string()),
    None,
    None,
    None
).await?;

// Specific endpoint errors
let endpoint_errors = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.url ~ "/api/collections/posts/records" && data.status >= 400"#.to_string()),
    None,
    None,
    None
).await?;

// Errors or slow requests
let problems = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    Some("data.status >= 400 || data.execTime > 2.0".to_string()),
    None,
    None,
    None
).await?;
```

### Sorting Logs

```rust
// Sort by creation date (newest first)
let recent = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-created".to_string()),
    None,
    None
).await?;

// Sort by execution time (slowest first)
let slowest = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-data.execTime".to_string()),
    None,
    None
).await?;

// Sort by status code
let by_status = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("data.status".to_string()),
    None,
    None
).await?;

// Sort by rowid (most efficient)
let by_row_id = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-rowid".to_string()),
    None,
    None
).await?;

// Multiple sort fields
let multi_sort = pb.logs.get_list(
    1,
    50,
    false,
    HashMap::new(),
    HashMap::new(),
    None,
    Some("-created,level".to_string()),
    None,
    None
).await?;
```

## View Log

Retrieve a single log entry by ID:

```rust
// Get specific log
let log = pb.logs.get_one(
    "ai5z3aoed6809au",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

println!("Message: {}", log["message"]);
println!("Status: {}", log["data"]["status"]);
println!("Execution Time: {}", log["data"]["execTime"]);
```

### Log Details

```rust
async fn analyze_log(
    pb: &BosBase,
    log_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let log = pb.logs.get_one(
        log_id,
        HashMap::new(),
        HashMap::new(),
        None,
        None
    ).await?;
    
    println!("Log ID: {}", log["id"]);
    println!("Created: {}", log["created"]);
    println!("Level: {}", log["level"]);
    println!("Message: {}", log["message"]);
    
    if let Some(data) = log.get("data") {
        if data.get("type").and_then(|t| t.as_str()) == Some("request") {
            println!("Method: {}", data["method"]);
            println!("URL: {}", data["url"]);
            println!("Status: {}", data["status"]);
            println!("Execution Time: {} ms", data["execTime"]);
            println!("Remote IP: {}", data["remoteIP"]);
            println!("User Agent: {}", data["userAgent"]);
            println!("Auth Collection: {}", data["auth"]);
        }
    }
    
    Ok(())
}
```

## Logs Statistics

Get hourly aggregated statistics for logs:

### Basic Usage

```rust
// Get all statistics
let stats = pb.logs.get_stats(
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Each stat entry contains:
// { "total": 4, "date": "2022-06-01 19:00:00.000" }
```

### Filtered Statistics

```rust
// Statistics for errors only
let error_stats = pb.logs.get_stats(
    HashMap::new(),
    HashMap::new(),
    Some("data.status >= 400".to_string())
).await?;

// Statistics for specific endpoint
let endpoint_stats = pb.logs.get_stats(
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.url ~ "/api/collections/posts/records""#.to_string())
).await?;

// Statistics for slow requests
let slow_stats = pb.logs.get_stats(
    HashMap::new(),
    HashMap::new(),
    Some("data.execTime > 1.0".to_string())
).await?;

// Statistics excluding superuser requests
let user_stats = pb.logs.get_stats(
    HashMap::new(),
    HashMap::new(),
    Some(r#"data.auth != "_superusers""#.to_string())
).await?;
```

## Complete Examples

### Example 1: Error Monitoring Dashboard

```rust
async fn get_error_metrics(
    pb: &BosBase,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Get error logs from last 24 hours
    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    let date_filter = format!(r#"created >= "{} 00:00:00""#, yesterday.format("%Y-%m-%d"));
    
    // 4xx errors
    let client_errors = pb.logs.get_list(
        1,
        100,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(format!(r#"{} && data.status >= 400 && data.status < 500"#, date_filter)),
        Some("-created".to_string()),
        None,
        None
    ).await?;
    
    // 5xx errors
    let server_errors = pb.logs.get_list(
        1,
        100,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(format!(r#"{} && data.status >= 500"#, date_filter)),
        Some("-created".to_string()),
        None,
        None
    ).await?;
    
    // Get hourly statistics
    let error_stats = pb.logs.get_stats(
        HashMap::new(),
        HashMap::new(),
        Some(format!(r#"{} && data.status >= 400"#, date_filter))
    ).await?;
    
    Ok(json!({
        "clientErrors": client_errors["items"],
        "serverErrors": server_errors["items"],
        "stats": error_stats
    }))
}
```

### Example 2: Performance Analysis

```rust
async fn analyze_performance(
    pb: &BosBase,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Get slow requests
    let slow_requests = pb.logs.get_list(
        1,
        50,
        false,
        HashMap::new(),
        HashMap::new(),
        Some("data.execTime > 1.0".to_string()),
        Some("-data.execTime".to_string()),
        None,
        None
    ).await?;
    
    // Analyze by endpoint
    let mut endpoint_stats: std::collections::HashMap<String, serde_json::Value> = HashMap::new();
    
    if let Some(items) = slow_requests.get("items").and_then(|i| i.as_array()) {
        for log in items {
            if let Some(url) = log["data"]["url"].as_str() {
                let url_path = url.split('?').next().unwrap_or(url);
                let entry = endpoint_stats.entry(url_path.to_string())
                    .or_insert_with(|| json!({
                        "count": 0,
                        "totalTime": 0.0,
                        "maxTime": 0.0
                    }));
                
                entry["count"] = json!(entry["count"].as_u64().unwrap() + 1);
                let exec_time = log["data"]["execTime"].as_f64().unwrap();
                entry["totalTime"] = json!(entry["totalTime"].as_f64().unwrap() + exec_time);
                entry["maxTime"] = json!(entry["maxTime"].as_f64().unwrap().max(exec_time));
            }
        }
    }
    
    // Calculate averages
    for (_, stats) in endpoint_stats.iter_mut() {
        let count = stats["count"].as_u64().unwrap() as f64;
        let total = stats["totalTime"].as_f64().unwrap();
        stats["avgTime"] = json!(total / count);
    }
    
    Ok(serde_json::to_value(endpoint_stats)?)
}
```

## Best Practices

1. **Use Filters**: Always use filters to narrow down results, especially for large log datasets
2. **Paginate**: Use pagination instead of fetching all logs at once
3. **Efficient Sorting**: Use `-rowid` for default sorting (most efficient)
4. **Filter Statistics**: Always filter statistics for meaningful insights
5. **Monitor Errors**: Regularly check for 4xx/5xx errors
6. **Performance Tracking**: Monitor execution times for slow endpoints
7. **Security Auditing**: Track authentication failures and suspicious activity
8. **Archive Old Logs**: Consider deleting or archiving old logs to maintain performance

## Limitations

- **Superuser Only**: All operations require superuser authentication
- **Data Fields**: Only fields in the `data` object are filterable
- **Statistics**: Statistics are aggregated hourly
- **Performance**: Large log datasets may be slow to query
- **Storage**: Logs accumulate over time and may need periodic cleanup

## Log Levels

- **0**: Info (normal requests)
- **> 0**: Warnings/Errors (non-200 status codes, exceptions, etc.)

Higher values typically indicate more severe issues.

## Related Documentation

- [Authentication](./AUTHENTICATION.md) - User authentication
- [API Records](./API_RECORDS.md) - Record operations
- [Collection API](./COLLECTION_API.md) - Collection management


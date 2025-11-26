# Crons API - Rust SDK Documentation

## Overview

The Crons API provides endpoints for viewing and manually triggering scheduled cron jobs. All operations require superuser authentication and allow you to list registered cron jobs and execute them on-demand.

**Key Features:**
- List all registered cron jobs
- View cron job schedules (cron expressions)
- Manually trigger cron jobs
- Built-in system jobs for maintenance tasks

**Backend Endpoints:**
- `GET /api/crons` - List cron jobs
- `POST /api/crons/{jobId}` - Run cron job

**Note**: All Crons API operations require superuser authentication.

## Authentication

All Crons API operations require superuser authentication:

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

## List Cron Jobs

Returns a list of all registered cron jobs with their IDs and schedule expressions.

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Get all cron jobs
let jobs = pb.crons.get_full_list(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Cron jobs: {:?}", jobs);
// [
//   { "id": "__pbLogsCleanup__", "expression": "0 */6 * * *" },
//   { "id": "__pbDBOptimize__", "expression": "0 0 * * *" },
//   { "id": "__pbMFACleanup__", "expression": "0 * * * *" },
//   { "id": "__pbOTPCleanup__", "expression": "0 * * * *" }
// ]
```

### Cron Job Structure

Each cron job contains:

```rust
// {
//   "id": string,        // Unique identifier for the job
//   "expression": string // Cron expression defining the schedule
// }
```

### Built-in System Jobs

The following cron jobs are typically registered by default:

| Job ID | Expression | Description | Schedule |
|--------|-----------|-------------|----------|
| `__pbLogsCleanup__` | `0 */6 * * *` | Cleans up old log entries | Every 6 hours |
| `__pbDBOptimize__` | `0 0 * * *` | Optimizes database | Daily at midnight |
| `__pbMFACleanup__` | `0 * * * *` | Cleans up expired MFA records | Every hour |
| `__pbOTPCleanup__` | `0 * * * *` | Cleans up expired OTP codes | Every hour |

### Working with Cron Jobs

```rust
// List all cron jobs
let jobs = pb.crons.get_full_list(
    HashMap::new(),
    HashMap::new()
).await?;

// Find a specific job
if let Some(jobs_array) = jobs.as_array() {
    if let Some(logs_cleanup) = jobs_array.iter().find(|job| job["id"] == "__pbLogsCleanup__") {
        println!("Logs cleanup runs: {}", logs_cleanup["expression"]);
    }
    
    // Filter system jobs
    let system_jobs: Vec<_> = jobs_array.iter()
        .filter(|job| {
            job["id"].as_str()
                .map(|id| id.starts_with("__pb"))
                .unwrap_or(false)
        })
        .collect();
    
    // Filter custom jobs
    let custom_jobs: Vec<_> = jobs_array.iter()
        .filter(|job| {
            job["id"].as_str()
                .map(|id| !id.starts_with("__pb"))
                .unwrap_or(false)
        })
        .collect();
}
```

## Run Cron Job

Manually trigger a cron job to execute immediately.

### Basic Usage

```rust
// Run a specific cron job
pb.crons.run(
    "__pbLogsCleanup__",
    HashMap::new(),
    HashMap::new()
).await?;
```

### Use Cases

```rust
// Trigger logs cleanup manually
async fn cleanup_logs_now(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    pb.crons.run(
        "__pbLogsCleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    println!("Logs cleanup triggered");
    Ok(())
}

// Trigger database optimization
async fn optimize_database(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    pb.crons.run(
        "__pbDBOptimize__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    println!("Database optimization triggered");
    Ok(())
}

// Trigger MFA cleanup
async fn cleanup_mfa(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    pb.crons.run(
        "__pbMFACleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    println!("MFA cleanup triggered");
    Ok(())
}

// Trigger OTP cleanup
async fn cleanup_otp(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    pb.crons.run(
        "__pbOTPCleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    println!("OTP cleanup triggered");
    Ok(())
}
```

## Cron Expression Format

Cron expressions use the standard 5-field format:

```
* * * * *
│ │ │ │ │
│ │ │ │ └─── Day of week (0-7, 0 or 7 is Sunday)
│ │ │ └───── Month (1-12)
│ │ └─────── Day of month (1-31)
│ └───────── Hour (0-23)
└─────────── Minute (0-59)
```

### Common Patterns

| Expression | Description |
|------------|-------------|
| `0 * * * *` | Every hour at minute 0 |
| `0 */6 * * *` | Every 6 hours |
| `0 0 * * *` | Daily at midnight |
| `0 0 * * 0` | Weekly on Sunday at midnight |
| `0 0 1 * *` | Monthly on the 1st at midnight |
| `*/30 * * * *` | Every 30 minutes |
| `0 9 * * 1-5` | Weekdays at 9 AM |

### Supported Macros

| Macro | Equivalent Expression | Description |
|-------|----------------------|-------------|
| `@yearly` or `@annually` | `0 0 1 1 *` | Once a year |
| `@monthly` | `0 0 1 * *` | Once a month |
| `@weekly` | `0 0 * * 0` | Once a week |
| `@daily` or `@midnight` | `0 0 * * *` | Once a day |
| `@hourly` | `0 * * * *` | Once an hour |

## Complete Examples

### Example 1: Cron Job Monitor

```rust
struct CronMonitor {
    pb: BosBase,
}

impl CronMonitor {
    async fn list_all_jobs(&self) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let jobs = self.pb.crons.get_full_list(
            HashMap::new(),
            HashMap::new()
        ).await?;
        
        if let Some(jobs_array) = jobs.as_array() {
            println!("Found {} cron jobs:", jobs_array.len());
            for job in jobs_array {
                println!("  - {}: {}", job["id"], job["expression"]);
            }
            Ok(jobs_array.clone())
        } else {
            Ok(vec![])
        }
    }

    async fn run_job(&self, job_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match self.pb.crons.run(
            job_id,
            HashMap::new(),
            HashMap::new()
        ).await {
            Ok(_) => {
                println!("Successfully triggered: {}", job_id);
                Ok(true)
            }
            Err(err) => {
                eprintln!("Failed to run {}: {:?}", job_id, err);
                Ok(false)
            }
        }
    }

    async fn run_maintenance_jobs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let maintenance_jobs = vec![
            "__pbLogsCleanup__",
            "__pbDBOptimize__",
            "__pbMFACleanup__",
            "__pbOTPCleanup__",
        ];

        for job_id in maintenance_jobs {
            println!("Running {}...", job_id);
            self.run_job(job_id).await?;
            // Wait a bit between jobs
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
```

### Example 2: Manual Maintenance Script

```rust
async fn perform_maintenance(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting maintenance tasks...");
    
    // Cleanup old logs
    println!("1. Cleaning up old logs...");
    pb.crons.run(
        "__pbLogsCleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    // Cleanup expired MFA records
    println!("2. Cleaning up expired MFA records...");
    pb.crons.run(
        "__pbMFACleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    // Cleanup expired OTP codes
    println!("3. Cleaning up expired OTP codes...");
    pb.crons.run(
        "__pbOTPCleanup__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    // Optimize database (run last as it may take longer)
    println!("4. Optimizing database...");
    pb.crons.run(
        "__pbDBOptimize__",
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    println!("Maintenance tasks completed");
    Ok(())
}
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.crons.get_full_list(HashMap::new(), HashMap::new()).await {
    Ok(jobs) => {
        println!("Jobs: {:?}", jobs);
    }
    Err(err) => {
        match err.status() {
            401 => {
                eprintln!("Not authenticated");
            }
            403 => {
                eprintln!("Not a superuser");
            }
            _ => {
                eprintln!("Unexpected error: {:?}", err);
            }
        }
    }
}

match pb.crons.run("__pbLogsCleanup__", HashMap::new(), HashMap::new()).await {
    Ok(_) => {
        println!("Job triggered successfully");
    }
    Err(err) => {
        match err.status() {
            401 => {
                eprintln!("Not authenticated");
            }
            403 => {
                eprintln!("Not a superuser");
            }
            404 => {
                eprintln!("Cron job not found");
            }
            _ => {
                eprintln!("Unexpected error: {:?}", err);
            }
        }
    }
}
```

## Best Practices

1. **Check Job Existence**: Verify a cron job exists before trying to run it
2. **Error Handling**: Always handle errors when running cron jobs
3. **Rate Limiting**: Don't trigger cron jobs too frequently manually
4. **Monitoring**: Regularly check that expected cron jobs are registered
5. **Logging**: Log when cron jobs are manually triggered for auditing
6. **Testing**: Test cron jobs in development before running in production
7. **Documentation**: Document custom cron jobs and their purposes
8. **Scheduling**: Let the cron scheduler handle regular execution; use manual triggers sparingly

## Limitations

- **Superuser Only**: All operations require superuser authentication
- **Read-Only API**: The SDK API only allows listing and running jobs; adding/removing jobs must be done via backend hooks
- **Asynchronous Execution**: Running a cron job triggers it asynchronously; the API returns immediately
- **No Status**: The API doesn't provide execution status or history
- **System Jobs**: Built-in system jobs (prefixed with `__pb`) cannot be removed via the API

## Related Documentation

- [Collection API](./COLLECTION_API.md) - Collection management
- [Logs API](./LOGS_API.md) - Log viewing and analysis
- [Backups API](./BACKUPS_API.md) - Backup management


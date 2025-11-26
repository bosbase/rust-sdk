# Backups API - Rust SDK Documentation

## Overview

The Backups API provides endpoints for managing application data backups. You can create backups, upload existing backup files, download backups, delete backups, and restore the application from a backup.

**Key Features:**
- List all available backup files
- Create new backups with custom names or auto-generated names
- Upload existing backup ZIP files
- Download backup files (requires file token)
- Delete backup files
- Restore the application from a backup (restarts the app)

**Backend Endpoints:**
- `GET /api/backups` - List backups
- `POST /api/backups` - Create backup
- `POST /api/backups/upload` - Upload backup
- `GET /api/backups/{key}` - Download backup
- `DELETE /api/backups/{key}` - Delete backup
- `POST /api/backups/{key}/restore` - Restore backup

**Note**: All Backups API operations require superuser authentication (except download which requires a superuser file token).

## Authentication

All Backups API operations require superuser authentication:

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

**Downloading backups** requires a superuser file token (obtained via `pb.files.get_token()`), but does not require the Authorization header.

## Backup File Structure

Each backup file contains:
- `key`: The filename/key of the backup file (string)
- `size`: File size in bytes (number)
- `modified`: ISO 8601 timestamp of when the backup was last modified (string)

## List Backups

Returns a list of all available backup files with their metadata.

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Get all backups
let backups = pb.backups.get_full_list(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Backups: {:?}", backups);
// [
//   {
//     "key": "pb_backup_20230519162514.zip",
//     "modified": "2023-05-19T16:25:57.542Z",
//     "size": 251316185
//   },
//   ...
// ]
```

### Working with Backup Lists

```rust
// Sort backups by modification date (newest first)
let mut backups: Vec<serde_json::Value> = pb.backups.get_full_list(
    HashMap::new(),
    HashMap::new()
).await?.as_array().unwrap().clone();

backups.sort_by(|a, b| {
    let a_modified = a["modified"].as_str().unwrap();
    let b_modified = b["modified"].as_str().unwrap();
    b_modified.cmp(a_modified)
});

// Find the most recent backup
if let Some(most_recent) = backups.first() {
    println!("Most recent backup: {:?}", most_recent);
}

// Filter backups by size (larger than 100MB)
let large_backups: Vec<_> = backups.iter()
    .filter(|backup| {
        backup["size"].as_u64().unwrap() > 100 * 1024 * 1024
    })
    .collect();

// Get total storage used by backups
let total_size: u64 = backups.iter()
    .map(|b| b["size"].as_u64().unwrap())
    .sum();
println!("Total backup storage: {} MB", total_size / 1024 / 1024);
```

## Create Backup

Creates a new backup of the application data. The backup process is asynchronous and may take some time depending on the size of your data.

### Basic Usage

```rust
// Create backup with custom name
pb.backups.create(
    "my_backup_2024.zip",
    HashMap::new(),
    HashMap::new()
).await?;

// Create backup with auto-generated name (pass empty string)
pb.backups.create(
    "",
    HashMap::new(),
    HashMap::new()
).await?;
```

### Backup Name Format

Backup names must follow the format: `[a-z0-9_-].zip`
- Only lowercase letters, numbers, underscores, and hyphens
- Must end with `.zip`
- Maximum length: 150 characters
- Must be unique (no existing backup with the same name)

### Examples

```rust
async fn create_named_backup(
    pb: &BosBase,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match pb.backups.create(name, HashMap::new(), HashMap::new()).await {
        Ok(_) => {
            println!("Backup \"{}\" creation initiated", name);
            Ok(())
        }
        Err(err) => {
            if err.status() == 400 {
                eprintln!("Invalid backup name or backup already exists");
            } else {
                eprintln!("Failed to create backup: {:?}", err);
            }
            Err(err.into())
        }
    }
}

// Create backup with timestamp
fn create_timestamped_backup(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let name = format!("backup_{}.zip", timestamp);
    pb.backups.create(&name, HashMap::new(), HashMap::new()).await?;
    Ok(())
}
```

### Important Notes

- **Asynchronous Process**: Backup creation happens in the background. The API returns immediately (204 No Content).
- **Concurrent Operations**: Only one backup or restore operation can run at a time. If another operation is in progress, you'll receive a 400 error.
- **Storage**: Backups are stored in the configured backup filesystem (local or S3).
- **S3 Consistency**: For S3 storage, the backup file may not be immediately available after creation due to eventual consistency.

## Upload Backup

Uploads an existing backup ZIP file to the server. This is useful for restoring backups created elsewhere or for importing backups.

### Basic Usage

```rust
use bosbase::FileAttachment;

// Upload from file bytes
let mut files = Vec::new();
files.push(FileAttachment {
    field: "file".to_string(),
    filename: "backup.zip".to_string(),
    content_type: "application/zip".to_string(),
    data: backup_bytes,  // Your backup file bytes
});

pb.backups.upload(
    files,
    HashMap::new(),
    HashMap::new()
).await?;
```

### File Requirements

- **MIME Type**: Must be `application/zip`
- **Format**: Must be a valid ZIP archive
- **Name**: Must be unique (no existing backup with the same name)
- **Validation**: The file will be validated before upload

## Download Backup

Download a backup file. Requires a superuser file token.

### Basic Usage

```rust
// Get file token
let token_result = pb.files.get_token(
    HashMap::new(),
    HashMap::new()
).await?;
let token = token_result["token"].as_str().unwrap();

// Download backup
let backup_url = pb.backups.get_download_url(
    "pb_backup_20230519162514.zip",
    token,
    HashMap::new()
);

println!("Download URL: {}", backup_url);
```

## Delete Backup

Delete a backup file:

```rust
pb.backups.delete(
    "pb_backup_20230519162514.zip",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Restore Backup

Restore the application from a backup. This will restart the application.

```rust
// Restore from backup
pb.backups.restore(
    "pb_backup_20230519162514.zip",
    HashMap::new(),
    HashMap::new()
).await?;

// Note: This will restart the application
```

## Complete Examples

### Example 1: Automated Backup Scheduler

```rust
struct BackupScheduler {
    pb: BosBase,
}

impl BackupScheduler {
    async fn create_daily_backup(&self) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let name = format!("daily_backup_{}.zip", timestamp);
        
        self.pb.backups.create(&name, HashMap::new(), HashMap::new()).await?;
        
        Ok(name)
    }

    async fn cleanup_old_backups(&self, keep_days: i64) -> Result<(), Box<dyn std::error::Error>> {
        let backups = self.pb.backups.get_full_list(
            HashMap::new(),
            HashMap::new()
        ).await?;
        
        let cutoff = chrono::Utc::now() - chrono::Duration::days(keep_days);
        
        for backup in backups.as_array().unwrap() {
            if let Some(modified_str) = backup["modified"].as_str() {
                if let Ok(modified) = chrono::DateTime::parse_from_rfc3339(modified_str) {
                    if modified < cutoff {
                        self.pb.backups.delete(
                            backup["key"].as_str().unwrap(),
                            HashMap::new(),
                            HashMap::new()
                        ).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### Example 2: Backup Management

```rust
async fn manage_backups(pb: &BosBase) -> Result<(), Box<dyn std::error::Error>> {
    // List all backups
    let backups = pb.backups.get_full_list(
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    println!("Total backups: {}", backups.as_array().unwrap().len());
    
    // Create new backup
    let new_backup = pb.backups.create(
        "manual_backup.zip",
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    // Get token for download
    let token_result = pb.files.get_token(
        HashMap::new(),
        HashMap::new()
    ).await?;
    let token = token_result["token"].as_str().unwrap();
    
    // Get download URL
    let download_url = pb.backups.get_download_url(
        "manual_backup.zip",
        token,
        HashMap::new()
    );
    
    println!("Backup created. Download URL: {}", download_url);
    
    Ok(())
}
```

## Best Practices

1. **Regular Backups**: Create backups regularly (daily, weekly, etc.)
2. **Test Restores**: Periodically test restoring from backups
3. **Cleanup Old Backups**: Delete old backups to save storage space
4. **Monitor Storage**: Monitor backup storage usage
5. **Secure Storage**: Store backups in secure locations
6. **Documentation**: Document backup and restore procedures

## Related Documentation

- [Health API](./HEALTH_API.md) - Check backup readiness with `canBackup`
- [Files API](./FILE_API.md) - File token management for downloads


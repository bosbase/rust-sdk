# Files Upload and Handling - Rust SDK Documentation

## Overview

BosBase allows you to upload and manage files through file fields in your collections. Files are stored with sanitized names and a random suffix for security (e.g., `test_52iwbgds7l.png`).

**Key Features:**
- Upload multiple files per field
- Maximum file size: ~8GB (2^53-1 bytes)
- Automatic filename sanitization and random suffix
- Image thumbnails support
- Protected files with token-based access
- File modifiers for append/prepend/delete operations

**Backend Endpoints:**
- `POST /api/files/token` - Get file access token for protected files
- `GET /api/files/{collection}/{recordId}/{filename}` - Download file

## File Field Configuration

Before uploading files, you must add a file field to your collection:

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let mut collection = pb.collections.get_one(
    "example",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Add file field
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    fields.push(json!({
        "name": "documents",
        "type": "file",
        "maxSelect": 5,        // Maximum number of files (1 for single file)
        "maxSize": 5242880,    // 5MB in bytes (optional, default: 5MB)
        "mimeTypes": ["image/jpeg", "image/png", "application/pdf"],
        "thumbs": ["100x100", "300x300"],  // Thumbnail sizes for images
        "protected": false     // Require token for access
    }));
}

pb.collections.update(
    "example",
    json!({
        "fields": collection["fields"]
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Uploading Files

### Basic Upload with Create

When creating a new record, you can upload files directly:

```rust
use bosbase::{BosBase, FileAttachment};
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Prepare files
let mut files = Vec::new();
files.push(FileAttachment {
    field: "documents".to_string(),
    filename: "file1.txt".to_string(),
    content_type: "text/plain".to_string(),
    data: b"content 1...".to_vec(),
});

files.push(FileAttachment {
    field: "documents".to_string(),
    filename: "file2.txt".to_string(),
    content_type: "text/plain".to_string(),
    data: b"content 2...".to_vec(),
});

// Create record with files
let created_record = pb.collection("example").create(
    json!({
        "title": "Hello world!"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;
```

### Upload with Update

```rust
// Update record and upload new files
let mut files = Vec::new();
files.push(FileAttachment {
    field: "documents".to_string(),
    filename: "file3.txt".to_string(),
    content_type: "text/plain".to_string(),
    data: b"content 3...".to_vec(),
});

let updated_record = pb.collection("example").update(
    "RECORD_ID",
    json!({
        "title": "Updated title"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;
```

### Append Files (Using + Modifier)

For multiple file fields, use the `+` modifier to append files:

```rust
// Append files to existing ones
let mut files = Vec::new();
files.push(FileAttachment {
    field: "documents+".to_string(),  // Note: + modifier in field name
    filename: "file4.txt".to_string(),
    content_type: "text/plain".to_string(),
    data: b"content 4...".to_vec(),
});

pb.collection("example").update(
    "RECORD_ID",
    json!({}),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;
```

## Deleting Files

### Delete All Files

```rust
// Delete all files in a field (set to empty array)
pb.collection("example").update(
    "RECORD_ID",
    json!({
        "documents": []
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Delete Specific Files (Using - Modifier)

```rust
// Delete individual files by filename
pb.collection("example").update(
    "RECORD_ID",
    json!({
        "documents-": ["file1.pdf", "file2.txt"]
    }),
    HashMap::new(),
    Vec::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## File URLs

### Get File URL

Each uploaded file can be accessed via its URL:

```
http://localhost:8090/api/files/COLLECTION_ID_OR_NAME/RECORD_ID/FILENAME
```

**Using SDK:**

```rust
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Single file field (returns string)
if let Some(filename) = record.get("documents").and_then(|f| f.as_str()) {
    let url = pb.get_file_url(
        record.clone(),
        filename,
        None,      // thumb
        None,      // token
        None,      // download
        HashMap::new()
    );
    println!("File URL: {}", url);
}

// Multiple file field (returns array)
if let Some(files) = record.get("documents").and_then(|f| f.as_array()) {
    if let Some(first_file) = files.first().and_then(|f| f.as_str()) {
        let url = pb.get_file_url(
            record.clone(),
            first_file,
            None,
            None,
            None,
            HashMap::new()
        );
        println!("First file URL: {}", url);
    }
}
```

### Image Thumbnails

If your file field has thumbnail sizes configured, you can request thumbnails:

```rust
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("avatar").and_then(|f| f.as_str()) {
    // Get thumbnail with specific size
    let thumb_url = pb.get_file_url(
        record.clone(),
        filename,
        Some("100x300".to_string()),  // Width x Height
        None,
        None,
        HashMap::new()
    );
    println!("Thumbnail URL: {}", thumb_url);
}
```

**Thumbnail Formats:**

- `WxH` (e.g., `100x300`) - Crop to WxH viewbox from center
- `WxHt` (e.g., `100x300t`) - Crop to WxH viewbox from top
- `WxHb` (e.g., `100x300b`) - Crop to WxH viewbox from bottom
- `WxHf` (e.g., `100x300f`) - Fit inside WxH viewbox (no cropping)
- `0xH` (e.g., `0x300`) - Resize to H height, preserve aspect ratio
- `Wx0` (e.g., `100x0`) - Resize to W width, preserve aspect ratio

**Supported Image Formats:**
- JPEG (`.jpg`, `.jpeg`)
- PNG (`.png`)
- GIF (`.gif` - first frame only)
- WebP (`.webp` - stored as PNG)

**Example:**

```rust
let record = pb.collection("products").get_one(
    "PRODUCT_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(image) = record.get("image").and_then(|i| i.as_str()) {
    // Different thumbnail sizes
    let thumb_small = pb.get_file_url(
        record.clone(),
        image,
        Some("100x100".to_string()),
        None,
        None,
        HashMap::new()
    );
    
    let thumb_medium = pb.get_file_url(
        record.clone(),
        image,
        Some("300x300f".to_string()),
        None,
        None,
        HashMap::new()
    );
    
    let thumb_large = pb.get_file_url(
        record.clone(),
        image,
        Some("800x600".to_string()),
        None,
        None,
        HashMap::new()
    );
}
```

### Force Download

To force browser download instead of preview:

```rust
let url = pb.get_file_url(
    record.clone(),
    filename,
    None,
    None,
    Some(true),  // Force download
    HashMap::new()
);
```

## Protected Files

By default, all files are publicly accessible if you know the full URL. For sensitive files, you can mark the field as "Protected" in the collection settings.

### Setting Up Protected Files

```rust
let mut collection = pb.collections.get_one(
    "example",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Find and update file field
if let Some(fields) = collection.get_mut("fields").and_then(|f| f.as_array_mut()) {
    if let Some(file_field) = fields.iter_mut().find(|f| {
        f.get("name").and_then(|n| n.as_str()) == Some("documents")
    }) {
        file_field["protected"] = json!(true);
    }
}

pb.collections.update(
    "example",
    json!({
        "fields": collection["fields"]
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Accessing Protected Files

Protected files require authentication and a file token:

```rust
// Step 1: Authenticate
pb.collection("users").auth_with_password(
    "user@example.com",
    "password123",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Step 2: Get file token (valid for ~2 minutes)
let token_result = pb.files.get_token(
    HashMap::new(),
    HashMap::new()
).await?;
let file_token = token_result["token"].as_str().unwrap();

// Step 3: Get protected file URL with token
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("privateDocument").and_then(|f| f.as_str()) {
    let url = pb.get_file_url(
        record.clone(),
        filename,
        None,
        Some(file_token.to_string()),
        None,
        HashMap::new()
    );
    println!("Protected file URL: {}", url);
}
```

**Important:**
- File tokens are short-lived (~2 minutes)
- Only authenticated users satisfying the collection's `viewRule` can access protected files
- Tokens must be regenerated when they expire

### Complete Protected File Example

```rust
async fn load_protected_image(
    pb: &BosBase,
    record_id: &str,
    filename: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Check if authenticated
    if !pb.auth_store().is_valid() {
        return Err("Not authenticated".into());
    }

    // Get fresh token
    let token_result = pb.files.get_token(
        HashMap::new(),
        HashMap::new()
    ).await?;
    let token = token_result["token"].as_str().unwrap();

    // Get file URL
    let record = pb.collection("example").get_one(
        record_id,
        HashMap::new(),
        HashMap::new(),
        None,
        None
    ).await?;

    let url = pb.get_file_url(
        record,
        filename,
        None,
        Some(token.to_string()),
        None,
        HashMap::new()
    );

    Ok(url)
}
```

## Complete Examples

### Example 1: Image Upload with Thumbnails

```rust
use bosbase::{BosBase, FileAttachment};
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Create collection with image field and thumbnails
let collection = pb.collections.create_base(
    "products",
    json!({
        "fields": [
            {
                "name": "name",
                "type": "text",
                "required": true
            },
            {
                "name": "image",
                "type": "file",
                "maxSelect": 1,
                "mimeTypes": ["image/jpeg", "image/png"],
                "thumbs": ["100x100", "300x300", "800x600f"]  // Thumbnail sizes
            }
        ]
    }),
    HashMap::new(),
    HashMap::new()
).await?;

// Upload product with image
let mut files = Vec::new();
files.push(FileAttachment {
    field: "image".to_string(),
    filename: "product.jpg".to_string(),
    content_type: "image/jpeg".to_string(),
    data: image_bytes,  // Your image bytes
});

let product = pb.collection("products").create(
    json!({
        "name": "My Product"
    }),
    HashMap::new(),
    files,
    HashMap::new(),
    None,
    None
).await?;

// Display thumbnail
if let Some(image) = product.get("image").and_then(|i| i.as_str()) {
    let thumbnail_url = pb.get_file_url(
        product.clone(),
        image,
        Some("300x300".to_string()),
        None,
        None,
        HashMap::new()
    );
    println!("Thumbnail URL: {}", thumbnail_url);
}
```

### Example 2: File Management

```rust
struct FileManager {
    pb: BosBase,
    collection_id: String,
    record_id: String,
}

impl FileManager {
    async fn load(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(self.pb.collection(&self.collection_id).get_one(
            &self.record_id,
            HashMap::new(),
            HashMap::new(),
            None,
            None
        ).await?)
    }

    async fn delete_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.collection(&self.collection_id).update(
            &self.record_id,
            json!({
                "documents-": [filename]
            }),
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
            None,
            None
        ).await?;
        Ok(())
    }

    async fn add_files(&self, files: Vec<FileAttachment>) -> Result<(), Box<dyn std::error::Error>> {
        self.pb.collection(&self.collection_id).update(
            &self.record_id,
            json!({}),
            HashMap::new(),
            files,
            HashMap::new(),
            None,
            None
        ).await?;
        Ok(())
    }
}
```

## File Field Modifiers

### Summary

- **No modifier** - Replace all files: `documents: []`
- **`+` suffix** - Append files: Use `"documents+"` as field name in FileAttachment
- **`-` suffix** - Delete files: `"documents-": ["file1.pdf"]`

## Best Practices

1. **File Size Limits**: Always validate file sizes on the client before upload
2. **MIME Types**: Configure allowed MIME types in collection field settings
3. **Thumbnails**: Pre-generate common thumbnail sizes for better performance
4. **Protected Files**: Use protected files for sensitive documents (ID cards, contracts)
5. **Token Refresh**: Refresh file tokens before they expire for protected files
6. **Error Handling**: Handle 404 errors for missing files and 401 for protected file access
7. **Filename Sanitization**: Files are automatically sanitized, but validate on client side too

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

match pb.collection("example").create(
    json!({
        "title": "Test"
    }),
    HashMap::new(),
    vec![FileAttachment {
        field: "documents".to_string(),
        filename: "test.txt".to_string(),
        content_type: "text/plain".to_string(),
        data: b"content".to_vec(),
    }],
    HashMap::new(),
    None,
    None
).await {
    Ok(record) => {
        println!("Upload successful: {:?}", record);
    }
    Err(err) => {
        match err.status() {
            413 => {
                eprintln!("File too large");
            }
            400 => {
                eprintln!("Invalid file type or field validation failed");
            }
            403 => {
                eprintln!("Insufficient permissions");
            }
            _ => {
                eprintln!("Upload failed: {:?}", err);
            }
        }
    }
}
```

## Storage Options

By default, BosBase stores files in `pb_data/storage` on the local filesystem. For production, you can configure S3-compatible storage (AWS S3, MinIO, Wasabi, DigitalOcean Spaces, etc.) from:
**Dashboard > Settings > Files storage**

This is configured server-side and doesn't require SDK changes.

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection and field configuration
- [Authentication](./AUTHENTICATION.md) - Required for protected files
- [File API](./FILE_API.md) - File download and access


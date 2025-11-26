# File API - Rust SDK Documentation

## Overview

The File API provides endpoints for downloading and accessing files stored in collection records. It supports thumbnail generation for images, protected file access with tokens, and force download options.

**Key Features:**
- Download files from collection records
- Generate thumbnails for images (crop, fit, resize)
- Protected file access with short-lived tokens
- Force download option for any file type
- Automatic content-type detection
- Support for Range requests and caching

**Backend Endpoints:**
- `GET /api/files/{collection}/{recordId}/{filename}` - Download/fetch file
- `POST /api/files/token` - Generate protected file token

## Download / Fetch File

Downloads a single file resource from a record.

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Get a record with a file field
let record = pb.collection("posts").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Get the file URL
if let Some(filename) = record.get("image").and_then(|f| f.as_str()) {
    let file_url = pb.get_file_url(
        record.clone(),
        filename,
        None,      // thumb
        None,      // token
        None,      // download
        HashMap::new()
    );
    println!("File URL: {}", file_url);
}
```

### File URL Structure

The file URL follows this pattern:
```
/api/files/{collectionIdOrName}/{recordId}/{filename}
```

Example:
```
http://127.0.0.1:8090/api/files/posts/abc123/photo_xyz789.jpg
```

## Thumbnails

Generate thumbnails for image files on-the-fly.

### Thumbnail Formats

The following thumbnail formats are supported:

| Format | Example | Description |
|--------|---------|-------------|
| `WxH` | `100x300` | Crop to WxH viewbox (from center) |
| `WxHt` | `100x300t` | Crop to WxH viewbox (from top) |
| `WxHb` | `100x300b` | Crop to WxH viewbox (from bottom) |
| `WxHf` | `100x300f` | Fit inside WxH viewbox (without cropping) |
| `0xH` | `0x300` | Resize to H height preserving aspect ratio |
| `Wx0` | `100x0` | Resize to W width preserving aspect ratio |

### Using Thumbnails

```rust
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("image").and_then(|f| f.as_str()) {
    // Get thumbnail URL
    let thumb_url = pb.get_file_url(
        record.clone(),
        filename,
        Some("100x100".to_string()),
        None,
        None,
        HashMap::new()
    );

    // Different thumbnail sizes
    let small_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("50x50".to_string()),
        None,
        None,
        HashMap::new()
    );

    let medium_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("200x200".to_string()),
        None,
        None,
        HashMap::new()
    );

    let large_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("500x500".to_string()),
        None,
        None,
        HashMap::new()
    );

    // Fit thumbnail (no cropping)
    let fit_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("200x200f".to_string()),
        None,
        None,
        HashMap::new()
    );

    // Resize to specific width
    let width_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("300x0".to_string()),
        None,
        None,
        HashMap::new()
    );

    // Resize to specific height
    let height_thumb = pb.get_file_url(
        record.clone(),
        filename,
        Some("0x200".to_string()),
        None,
        None,
        HashMap::new()
    );
}
```

### Thumbnail Behavior

- **Image Files Only**: Thumbnails are only generated for image files (PNG, JPG, JPEG, GIF, WEBP)
- **Non-Image Files**: For non-image files, the thumb parameter is ignored and the original file is returned
- **Caching**: Thumbnails are cached and reused if already generated
- **Fallback**: If thumbnail generation fails, the original file is returned
- **Field Configuration**: Thumb sizes must be defined in the file field's `thumbs` option or use default `100x100`

## Protected Files

Protected files require a special token for access, even if you're authenticated.

### Getting a File Token

```rust
// Must be authenticated first
pb.collection("users").auth_with_password(
    "user@example.com",
    "password",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Get file token
let token_result = pb.files.get_token(
    HashMap::new(),
    HashMap::new()
).await?;
let token = token_result["token"].as_str().unwrap();
println!("File token: {}", token);
```

### Using Protected File Token

```rust
// Get protected file URL with token
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("document").and_then(|f| f.as_str()) {
    let protected_file_url = pb.get_file_url(
        record.clone(),
        filename,
        None,
        Some(token.to_string()),
        None,
        HashMap::new()
    );
    
    // Use the URL to fetch the file
    println!("Protected file URL: {}", protected_file_url);
}
```

### Protected File Example

```rust
async fn display_protected_image(
    pb: &BosBase,
    record_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Authenticate
    pb.collection("users").auth_with_password(
        "user@example.com",
        "password",
        HashMap::new(),
        HashMap::new(),
        None
    ).await?;
    
    // Get record
    let record = pb.collection("documents").get_one(
        record_id,
        HashMap::new(),
        HashMap::new(),
        None,
        None
    ).await?;
    
    // Get file token
    let token_result = pb.files.get_token(
        HashMap::new(),
        HashMap::new()
    ).await?;
    let token = token_result["token"].as_str().unwrap();
    
    // Get protected file URL
    if let Some(filename) = record.get("thumbnail").and_then(|f| f.as_str()) {
        let image_url = pb.get_file_url(
            record.clone(),
            filename,
            Some("300x300".to_string()),
            Some(token.to_string()),
            None,
            HashMap::new()
        );
        return Ok(image_url);
    }
    
    Err("No thumbnail found".into())
}
```

### Token Lifetime

- File tokens are short-lived (typically expires after a few minutes)
- Tokens are associated with the authenticated user/superuser
- Generate a new token if the previous one expires

## Force Download

Force files to download instead of being displayed in the browser.

```rust
let record = pb.collection("example").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("document").and_then(|f| f.as_str()) {
    // Force download
    let download_url = pb.get_file_url(
        record.clone(),
        filename,
        None,
        None,
        Some(true),  // Force download
        HashMap::new()
    );
    println!("Download URL: {}", download_url);
}
```

## Complete Examples

### Example 1: Image Gallery

```rust
async fn display_image_gallery(
    pb: &BosBase,
    record_id: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let record = pb.collection("posts").get_one(
        record_id,
        HashMap::new(),
        HashMap::new(),
        None,
        None
    ).await?;
    
    let mut urls = Vec::new();
    
    // Handle both single and multiple file fields
    let images: Vec<String> = if let Some(images_array) = record.get("images").and_then(|i| i.as_array()) {
        images_array.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else if let Some(image) = record.get("image").and_then(|i| i.as_str()) {
        vec![image.to_string()]
    } else {
        vec![]
    };
    
    for filename in images {
        // Thumbnail for gallery
        let thumb_url = pb.get_file_url(
            record.clone(),
            &filename,
            Some("200x200".to_string()),
            None,
            None,
            HashMap::new()
        );
        
        // Full image URL
        let full_url = pb.get_file_url(
            record.clone(),
            &filename,
            None,
            None,
            None,
            HashMap::new()
        );
        
        urls.push(full_url);
    }
    
    Ok(urls)
}
```

### Example 2: Protected File Viewer

```rust
async fn view_protected_file(
    pb: &BosBase,
    record_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Authenticate if needed
    if !pb.auth_store().is_valid() {
        pb.collection("users").auth_with_password(
            "user@example.com",
            "password",
            HashMap::new(),
            HashMap::new(),
            None
        ).await?;
    }
    
    // Get token
    let token_result = pb.files.get_token(
        HashMap::new(),
        HashMap::new()
    ).await?;
    let token = token_result["token"].as_str().unwrap();
    
    // Get record and file URL
    let record = pb.collection("private_docs").get_one(
        record_id,
        HashMap::new(),
        HashMap::new(),
        None,
        None
    ).await?;
    
    if let Some(filename) = record.get("file").and_then(|f| f.as_str()) {
        let file_url = pb.get_file_url(
            record.clone(),
            filename,
            None,
            Some(token.to_string()),
            None,
            HashMap::new()
        );
        return Ok(file_url);
    }
    
    Err("File not found".into())
}
```

## Error Handling

```rust
use bosbase::errors::ClientResponseError;

let record = pb.collection("posts").get_one(
    "RECORD_ID",
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

if let Some(filename) = record.get("image").and_then(|f| f.as_str()) {
    match pb.get_file_url(
        record.clone(),
        filename,
        None,
        None,
        None,
        HashMap::new()
    ).as_str() {
        url if !url.is_empty() => {
            println!("File URL: {}", url);
        }
        _ => {
            eprintln!("Invalid file URL");
        }
    }
}
```

### Protected File Token Error Handling

```rust
async fn get_protected_file_url(
    pb: &BosBase,
    record: serde_json::Value,
    filename: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match pb.files.get_token(
        HashMap::new(),
        HashMap::new()
    ).await {
        Ok(token_result) => {
            let token = token_result["token"].as_str().unwrap();
            
            // Get file URL
            Ok(pb.get_file_url(
                record,
                filename,
                None,
                Some(token.to_string()),
                None,
                HashMap::new()
            ))
        }
        Err(err) => {
            match err.status() {
                401 => {
                    eprintln!("Not authenticated");
                }
                403 => {
                    eprintln!("No permission to access file");
                }
                _ => {
                    eprintln!("Failed to get file token: {:?}", err);
                }
            }
            Err(err.into())
        }
    }
}
```

## Best Practices

1. **Use Thumbnails for Lists**: Use thumbnails when displaying images in lists/grids to reduce bandwidth
2. **Cache Tokens**: Store file tokens and reuse them until they expire
3. **Error Handling**: Always handle file loading errors gracefully
4. **Content-Type**: Let the server handle content-type detection automatically
5. **Range Requests**: The API supports Range requests for efficient video/audio streaming
6. **Caching**: Files are cached with a 30-day cache-control header
7. **Security**: Always use tokens for protected files, never expose them in client-side code

## Thumbnail Size Guidelines

| Use Case | Recommended Size |
|----------|-----------------|
| Profile picture | `100x100` or `150x150` |
| List thumbnails | `200x200` or `300x300` |
| Card images | `400x400` or `500x500` |
| Gallery previews | `300x300f` (fit) or `400x400f` |
| Hero images | Use original or `800x800f` |
| Avatar | `50x50` or `75x75` |

## Limitations

- **Thumbnails**: Only work for image files (PNG, JPG, JPEG, GIF, WEBP)
- **Protected Files**: Require authentication to get tokens
- **Token Expiry**: File tokens expire after a short period (typically minutes)
- **File Size**: Large files may take time to generate thumbnails on first request
- **Thumb Sizes**: Must match sizes defined in field configuration or use default `100x100`

## Related Documentation

- [Files Upload and Handling](./FILES.md) - Uploading and managing files
- [API Records](./API_RECORDS.md) - Working with records
- [Collections](./COLLECTIONS.md) - Collection configuration


# Authentication - Rust SDK Documentation

## Overview

Authentication in BosBase is stateless and token-based. A client is considered authenticated as long as it sends a valid `Authorization: YOUR_AUTH_TOKEN` header with requests.

**Key Points:**
- **No sessions**: BosBase APIs are fully stateless (tokens are not stored in the database)
- **No logout endpoint**: To "logout", simply clear the token from your local state (`pb.auth_store().clear()`)
- **Token generation**: Auth tokens are generated through auth collection Web APIs or programmatically
- **Admin users**: `_superusers` collection works like regular auth collections but with full access (API rules are ignored)
- **OAuth2 limitation**: OAuth2 is not supported for `_superusers` collection

## Authentication Methods

BosBase supports multiple authentication methods that can be configured individually for each auth collection:

1. **Password Authentication** - Email/username + password
2. **OTP Authentication** - One-time password via email
3. **OAuth2 Authentication** - Google, GitHub, Microsoft, etc.
4. **Multi-factor Authentication (MFA)** - Requires 2 different auth methods

## Authentication Store

The SDK maintains an `auth_store` that automatically manages the authentication state:

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Check authentication status
println!("Is valid: {}", pb.auth_store().is_valid());
println!("Token: {}", pb.auth_store().token());
println!("Record: {:?}", pb.auth_store().record());

// Clear authentication (logout)
pb.auth_store().clear();
```

## Password Authentication

Authenticate using email/username and password. The identity field can be configured in the collection options (default is email).

**Backend Endpoint:** `POST /api/collections/{collection}/auth-with-password`

### Basic Usage

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");

// Authenticate with email and password
let auth_data = pb.collection("users").auth_with_password(
    "test@example.com",
    "password123",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

// Auth data is automatically stored in pb.auth_store()
println!("Is valid: {}", pb.auth_store().is_valid());  // true
println!("Token: {}", pb.auth_store().token());        // JWT token
println!("User ID: {}", pb.auth_store().record()["id"]); // user record ID
```

### Response Format

```rust
// Response structure:
// {
//   "token": "eyJhbGciOiJIUzI1NiJ9...",
//   "record": {
//     "id": "record_id",
//     "email": "test@example.com",
//     // ... other user fields
//   }
// }
```

### Error Handling with MFA

```rust
use bosbase::errors::ClientResponseError;

match pb.collection("users").auth_with_password(
    "test@example.com",
    "pass123",
    HashMap::new(),
    HashMap::new(),
    None
).await {
    Ok(auth_data) => {
        println!("Authentication successful");
    }
    Err(err) => {
        // Check for MFA requirement
        if let Some(data) = err.data().as_object() {
            if let Some(mfa_id) = data.get("mfaId") {
                let mfa_id_str = mfa_id.as_str().unwrap();
                // Handle MFA flow (see Multi-factor Authentication section)
            } else {
                eprintln!("Authentication failed: {:?}", err);
            }
        }
    }
}
```

## OTP Authentication

One-time password authentication via email.

**Backend Endpoints:**
- `POST /api/collections/{collection}/request-otp` - Request OTP
- `POST /api/collections/{collection}/auth-with-otp` - Authenticate with OTP

### Request OTP

```rust
// Send OTP to user's email
let result = pb.collection("users").request_otp(
    "test@example.com",
    HashMap::new(),
    HashMap::new()
).await?;

println!("OTP ID: {}", result["otpId"]);  // OTP ID to use in auth_with_otp
```

### Authenticate with OTP

```rust
// Step 1: Request OTP
let result = pb.collection("users").request_otp(
    "test@example.com",
    HashMap::new(),
    HashMap::new()
).await?;

// Step 2: User enters OTP from email
// Step 3: Authenticate with OTP
let auth_data = pb.collection("users").auth_with_otp(
    result["otpId"].as_str().unwrap(),
    "123456",  // OTP code from email
    None,
    HashMap::new(),
    HashMap::new()
).await?;
```

## OAuth2 Authentication

**Backend Endpoint:** `POST /api/collections/{collection}/auth-with-oauth2`

### Manual Code Exchange

```rust
// Get auth methods
let auth_methods = pb.collection("users").list_auth_methods(
    HashMap::new(),
    HashMap::new()
).await?;

// Find provider
let providers = auth_methods["oauth2"]["providers"].as_array().unwrap();
let provider = providers.iter().find(|p| p["name"] == "google");

if let Some(provider) = provider {
    // Exchange code for token (after OAuth2 redirect)
    let auth_data = pb.collection("users").auth_with_oauth2_code(
        "google",                    // Provider name
        "AUTHORIZATION_CODE",        // From redirect URL
        provider["codeVerifier"].as_str().unwrap(),
        "https://yourapp.com/callback", // Redirect URL
        serde_json::json!({}),        // Optional data for new accounts
        HashMap::new(),
        HashMap::new()
    ).await?;
}
```

## Multi-Factor Authentication (MFA)

Requires 2 different auth methods.

```rust
let mut mfa_id: Option<String> = None;

match pb.collection("users").auth_with_password(
    "test@example.com",
    "pass123",
    HashMap::new(),
    HashMap::new(),
    None
).await {
    Ok(_) => {
        println!("Authentication successful");
    }
    Err(err) => {
        if let Some(data) = err.data().as_object() {
            if let Some(mfa_id_val) = data.get("mfaId") {
                mfa_id = mfa_id_val.as_str().map(|s| s.to_string());
                
                // Second auth method (OTP)
                let otp_result = pb.collection("users").request_otp(
                    "test@example.com",
                    HashMap::new(),
                    HashMap::new()
                ).await?;
                
                let auth_data = pb.collection("users").auth_with_otp(
                    otp_result["otpId"].as_str().unwrap(),
                    "123456",
                    mfa_id.clone(),
                    HashMap::new(),
                    HashMap::new()
                ).await?;
            }
        }
    }
}
```

## User Impersonation

Superusers can impersonate other users.

**Backend Endpoint:** `POST /api/collections/{collection}/impersonate/{id}`

```rust
// Authenticate as superuser
pb.admins().auth_with_password("admin@example.com", "adminpass").await?;

// Impersonate a user
let impersonate_client = pb.collection("users").impersonate(
    "USER_RECORD_ID",
    3600,  // Optional: token duration in seconds
    HashMap::new(),
    HashMap::new()
).await?;

// Use impersonate client
let data = impersonate_client.collection("posts").get_full_list(
    200,
    HashMap::new(),
    HashMap::new(),
    None,
    None,
    None,
    None
).await?;
```

## Auth Token Verification

Verify token by calling `auth_refresh()`.

**Backend Endpoint:** `POST /api/collections/{collection}/auth-refresh`

```rust
match pb.collection("users").auth_refresh(
    HashMap::new(),
    HashMap::new()
).await {
    Ok(_) => {
        println!("Token is valid");
    }
    Err(err) => {
        eprintln!("Token verification failed: {:?}", err);
        pb.auth_store().clear();
    }
}
```

## List Available Auth Methods

**Backend Endpoint:** `GET /api/collections/{collection}/auth-methods`

```rust
let auth_methods = pb.collection("users").list_auth_methods(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Password enabled: {}", auth_methods["password"]["enabled"]);
println!("OAuth2 providers: {:?}", auth_methods["oauth2"]["providers"]);
println!("MFA enabled: {}", auth_methods["mfa"]["enabled"]);
```

## Complete Examples

### Example 1: Complete Authentication Flow with Error Handling

```rust
use bosbase::BosBase;
use bosbase::errors::ClientResponseError;
use std::collections::HashMap;

async fn authenticate_user(
    pb: &BosBase,
    email: &str,
    password: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match pb.collection("users").auth_with_password(
        email,
        password,
        HashMap::new(),
        HashMap::new(),
        None
    ).await {
        Ok(auth_data) => {
            println!("Successfully authenticated: {}", auth_data["record"]["email"]);
            Ok(auth_data)
        }
        Err(err) => {
            // Check if MFA is required
            if err.status() == 401 {
                if let Some(data) = err.data().as_object() {
                    if let Some(mfa_id) = data.get("mfaId") {
                        println!("MFA required, proceeding with second factor...");
                        return handle_mfa(pb, email, mfa_id.as_str().unwrap()).await;
                    }
                }
            }
            
            // Handle other errors
            if err.status() == 400 {
                return Err("Invalid credentials".into());
            } else if err.status() == 403 {
                return Err("Password authentication is not enabled for this collection".into());
            } else {
                return Err(err.into());
            }
        }
    }
}

async fn handle_mfa(
    pb: &BosBase,
    email: &str,
    mfa_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Request OTP for second factor
    let otp_result = pb.collection("users").request_otp(
        email,
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    // In a real app, get OTP from user input
    let user_entered_otp = "123456"; // Get from user input
    
    match pb.collection("users").auth_with_otp(
        otp_result["otpId"].as_str().unwrap(),
        user_entered_otp,
        Some(mfa_id.to_string()),
        HashMap::new(),
        HashMap::new()
    ).await {
        Ok(auth_data) => {
            println!("MFA authentication successful");
            Ok(auth_data)
        }
        Err(err) => {
            if err.status() == 429 {
                return Err("Too many OTP attempts, please request a new OTP".into());
            }
            Err("Invalid OTP code".into())
        }
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pb = BosBase::new("http://localhost:8090");
    authenticate_user(&pb, "user@example.com", "password123").await?;
    println!("User is authenticated: {:?}", pb.auth_store().record());
    Ok(())
}
```

### Example 2: Token Management and Refresh

```rust
use bosbase::BosBase;
use std::collections::HashMap;

async fn check_auth(pb: &BosBase) -> Result<bool, Box<dyn std::error::Error>> {
    if pb.auth_store().is_valid() {
        println!("User is authenticated: {:?}", pb.auth_store().record()["email"]);
        
        // Verify token is still valid and refresh if needed
        match pb.collection("users").auth_refresh(
            HashMap::new(),
            HashMap::new()
        ).await {
            Ok(_) => {
                println!("Token refreshed successfully");
                Ok(true)
            }
            Err(_) => {
                println!("Token expired or invalid, clearing auth");
                pb.auth_store().clear();
                Ok(false)
            }
        }
    } else {
        Ok(false)
    }
}
```

### Example 3: Admin Impersonation for Support

```rust
use bosbase::BosBase;
use std::collections::HashMap;

async fn impersonate_user_for_support(
    pb: &BosBase,
    user_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Authenticate as admin
    pb.admins().auth_with_password("admin@example.com", "adminpassword").await?;
    
    // Impersonate the user (1 hour token)
    let user_client = pb.collection("users").impersonate(
        user_id,
        3600,
        HashMap::new(),
        HashMap::new()
    ).await?;
    
    println!("Impersonating user: {:?}", user_client.auth_store().record()["email"]);
    
    // Use the impersonated client to test user experience
    let user_records = user_client.collection("posts").get_full_list(
        200,
        HashMap::new(),
        HashMap::new(),
        None,
        None,
        None,
        None
    ).await?;
    
    println!("User can see {} posts", user_records.as_array().unwrap().len());
    
    // Check what the user sees
    let user_view = user_client.collection("posts").get_list(
        1,
        10,
        false,
        HashMap::new(),
        HashMap::new(),
        Some(r#"published = true"#.to_string()),
        None,
        None,
        None
    ).await?;
    
    Ok(json!({
        "canAccess": user_view["items"].as_array().unwrap().len(),
        "totalPosts": user_records.as_array().unwrap().len()
    }))
}
```

## Best Practices

1. **Secure Token Storage**: Never expose tokens in client-side code or logs
2. **Token Refresh**: Implement automatic token refresh before expiration
3. **Error Handling**: Always handle MFA requirements and token expiration
4. **OAuth2 Security**: Always validate the `state` parameter in OAuth2 callbacks
5. **API Keys**: Use impersonation tokens for server-to-server communication only
6. **Superuser Tokens**: Never expose superuser impersonation tokens in client code
7. **OTP Security**: Use OTP with MFA for security-critical applications
8. **Rate Limiting**: Be aware of rate limits on authentication endpoints

## Troubleshooting

### Token Expired
If you get 401 errors, check if the token has expired:

```rust
match pb.collection("users").auth_refresh(
    HashMap::new(),
    HashMap::new()
).await {
    Ok(_) => {
        println!("Token is still valid");
    }
    Err(_) => {
        // Token expired, require re-authentication
        pb.auth_store().clear();
        // Redirect to login
    }
}
```

### MFA Required
If authentication returns 401 with mfaId:

```rust
if err.status() == 401 {
    if let Some(data) = err.data().as_object() {
        if let Some(_mfa_id) = data.get("mfaId") {
            // Proceed with second authentication factor
        }
    }
}
```

## Related Documentation

- [Collections](./COLLECTIONS.md)
- [API Rules](./API_RULES_AND_FILTERS.md)


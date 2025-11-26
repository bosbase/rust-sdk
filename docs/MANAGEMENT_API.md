# Management API - Rust SDK Documentation

## Overview

This document covers the management API capabilities available in the Rust SDK, which correspond to the features available in the backend management UI.

> **Note**: All management API operations require superuser authentication (🔐).

## Settings Service

The Settings Service provides comprehensive management of application settings, matching the capabilities available in the backend management UI.

### Application Configuration

Manage application settings including meta information, trusted proxy, rate limits, and batch configuration.

#### Get Application Settings

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

let settings = pb.settings.get_application_settings(
    HashMap::new(),
    HashMap::new()
).await?;

println!("App name: {}", settings["meta"]["appName"]);
println!("Rate limits: {:?}", settings["rateLimits"]);
```

#### Update Application Settings

```rust
use serde_json::json;

pb.settings.update_application_settings(
    json!({
        "meta": {
            "appName": "My App",
            "appURL": "https://example.com",
            "hideControls": false
        },
        "trustedProxy": {
            "headers": ["X-Forwarded-For"],
            "useLeftmostIP": true
        },
        "rateLimits": {
            "enabled": true,
            "rules": [
                {
                    "label": "api/users",
                    "duration": 3600,
                    "maxRequests": 100
                }
            ]
        },
        "batch": {
            "enabled": true,
            "maxRequests": 100,
            "interval": 200
        }
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

### Mail Configuration

Manage SMTP email settings and sender information.

#### Get Mail Settings

```rust
let mail_settings = pb.settings.get_mail_settings(
    HashMap::new(),
    HashMap::new()
).await?;

println!("Sender: {}", mail_settings["meta"]["senderName"]);
println!("SMTP host: {}", mail_settings["smtp"]["host"]);
```

#### Update Mail Settings

```rust
pb.settings.update_mail_settings(
    json!({
        "senderName": "My App",
        "senderAddress": "noreply@example.com",
        "smtp": {
            "enabled": true,
            "host": "smtp.example.com",
            "port": 587,
            "username": "user@example.com",
            "password": "password",
            "authMethod": "PLAIN",
            "tls": true,
            "localName": "localhost"
        }
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

#### Test Email

Send a test email to verify SMTP configuration:

```rust
pb.settings.test_mail(
    "test@example.com",
    "verification",  // template: verification, password-reset, email-change, otp, login-alert
    Some("_superusers"),  // collection (optional, defaults to _superusers)
    HashMap::new(),
    HashMap::new()
).await?;
```

**Email Templates:**
- `verification` - Email verification template
- `password-reset` - Password reset template
- `email-change` - Email change confirmation template
- `otp` - One-time password template
- `login-alert` - Login alert template

## Related Documentation

- [Collection API](./COLLECTION_API.md) - Collection management
- [Backups API](./BACKUPS_API.md) - Backup management
- [Logs API](./LOGS_API.md) - Log viewing
- [Health API](./HEALTH_API.md) - Health checks


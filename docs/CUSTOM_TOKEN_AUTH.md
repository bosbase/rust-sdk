# Custom Token Binding and Login - Rust SDK Documentation

## Overview

The Rust SDK and BosBase service support binding a custom token to an auth record (both `users` and `_superusers`) and signing in with that token. The server stores bindings in the `_token_bindings` table (created automatically on first bind; legacy `_tokenBindings`/`tokenBindings` are auto-renamed). Tokens are stored as hashes so raw values aren't persisted.

## API endpoints
- `POST /api/collections/{collection}/bind-token`
- `POST /api/collections/{collection}/unbind-token`
- `POST /api/collections/{collection}/auth-with-token`

## Binding a token

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("http://127.0.0.1:8090");

// Bind for a regular user
pb.collection("users").bind_custom_token(
    "user@example.com",
    "user-password",
    "my-app-token",
    HashMap::new(),
    HashMap::new()
).await?;

// Bind for a superuser
pb.collection("_superusers").bind_custom_token(
    "admin@example.com",
    "admin-password",
    "admin-app-token",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Unbinding a token

```rust
// Stop accepting the token for the user
pb.collection("users").unbind_custom_token(
    "user@example.com",
    "user-password",
    "my-app-token",
    HashMap::new(),
    HashMap::new()
).await?;

// Stop accepting the token for a superuser
pb.collection("_superusers").unbind_custom_token(
    "admin@example.com",
    "admin-password",
    "admin-app-token",
    HashMap::new(),
    HashMap::new()
).await?;
```

## Logging in with a token

```rust
// Login with the previously bound token
let auth = pb.collection("users").auth_with_token(
    "my-app-token",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

println!("Token: {}", auth["token"]);
println!("Record: {:?}", auth["record"]);

// Superuser token login
let super_auth = pb.collection("_superusers").auth_with_token(
    "admin-app-token",
    HashMap::new(),
    HashMap::new(),
    None
).await?;

println!("Token: {}", super_auth["token"]);
println!("Record: {:?}", super_auth["record"]);
```

## Notes

- Binding and unbinding require a valid email and password for the target account.
- The same token value can be used for either `users` or `_superusers` collections; the collection is enforced during login.
- MFA and existing auth rules still apply when authenticating with a token.

## Related Documentation

- [Authentication](./AUTHENTICATION.md) - General authentication guide


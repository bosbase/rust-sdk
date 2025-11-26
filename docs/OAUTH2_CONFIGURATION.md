# OAuth2 Configuration Guide - Rust SDK Documentation

## Overview

This guide explains how to configure OAuth2 authentication providers for auth collections using the BosBase Rust SDK.

OAuth2 allows users to authenticate with your application using third-party providers like Google, GitHub, Facebook, etc. Before you can use OAuth2 authentication, you need to:

1. **Create an OAuth2 app** in the provider's dashboard
2. **Obtain Client ID and Client Secret** from the provider
3. **Register a redirect URL** (typically: `https://yourdomain.com/api/oauth2-redirect`)
4. **Configure the provider** in your BosBase auth collection using the SDK

## Prerequisites

- An auth collection in your BosBase instance
- OAuth2 app credentials (Client ID and Client Secret) from your chosen provider
- Admin/superuser authentication to configure collections

## Supported Providers

The following OAuth2 providers are supported:

- **google** - Google OAuth2
- **github** - GitHub OAuth2
- **gitlab** - GitLab OAuth2
- **discord** - Discord OAuth2
- **facebook** - Facebook OAuth2
- **microsoft** - Microsoft OAuth2
- **apple** - Apple Sign In
- **twitter** - Twitter OAuth2
- **spotify** - Spotify OAuth2
- **kakao** - Kakao OAuth2
- **twitch** - Twitch OAuth2
- **strava** - Strava OAuth2
- **vk** - VK OAuth2
- **yandex** - Yandex OAuth2
- **patreon** - Patreon OAuth2
- **linkedin** - LinkedIn OAuth2
- **instagram** - Instagram OAuth2
- **vimeo** - Vimeo OAuth2
- **digitalocean** - DigitalOcean OAuth2
- **bitbucket** - Bitbucket OAuth2
- **dropbox** - Dropbox OAuth2
- **planningcenter** - Planning Center OAuth2
- **notion** - Notion OAuth2
- **linear** - Linear OAuth2
- **oidc**, **oidc2**, **oidc3** - OpenID Connect (OIDC) providers

## Basic Usage

### 1. Enable OAuth2 for a Collection

First, enable OAuth2 authentication for your auth collection:

```rust
use bosbase::BosBase;
use std::collections::HashMap;

let pb = BosBase::new("https://your-instance.com");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Enable OAuth2 for the "users" collection
pb.collections.enable_oauth2(
    "users",
    HashMap::new(),
    HashMap::new()
).await?;
```

### 2. Add an OAuth2 Provider

Add a provider configuration to your collection:

```rust
use serde_json::json;

// Add Google OAuth2 provider
pb.collections.add_oauth2_provider(
    "users",
    json!({
        "name": "google",
        "clientId": "your-google-client-id",
        "clientSecret": "your-google-client-secret",
        "authURL": "https://accounts.google.com/o/oauth2/v2/auth",
        "tokenURL": "https://oauth2.googleapis.com/token",
        "userInfoURL": "https://www.googleapis.com/oauth2/v2/userinfo",
        "displayName": "Google",
        "pkce": true  // Optional: enable PKCE if supported
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

### 3. Get OAuth2 Configuration

Retrieve the current OAuth2 configuration:

```rust
let config = pb.collections.get_oauth2_config(
    "users",
    HashMap::new(),
    HashMap::new()
).await?;

println!("OAuth2 config: {:?}", config);
```

## User Authentication with OAuth2

Once configured, users can authenticate using OAuth2:

```rust
// Get available OAuth2 providers
let methods = pb.collection("users").list_auth_methods(
    HashMap::new(),
    HashMap::new()
).await?;

if let Some(oauth2) = methods.get("oauth2") {
    if let Some(providers) = oauth2.get("providers").and_then(|p| p.as_array()) {
        for provider in providers {
            println!("Provider: {}", provider["name"]);
        }
    }
}

// Authenticate with OAuth2
// Note: OAuth2 flow typically requires browser interaction
let auth = pb.collection("users").auth_with_oauth2(
    json!({
        "provider": "google"
    }),
    HashMap::new(),
    HashMap::new()
).await?;
```

## Related Documentation

- [Authentication](./AUTHENTICATION.md) - Complete authentication guide
- [Users Collection Guide](./USERS_COLLECTION_GUIDE.md) - Working with users collection


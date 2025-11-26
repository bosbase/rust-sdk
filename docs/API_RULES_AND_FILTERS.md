# API Rules and Filters - Rust SDK Documentation

## Overview

API Rules are your collection access controls and data filters. They control who can perform actions on your collections and what data they can access.

Each collection has 5 rules, corresponding to specific API actions:
- `listRule` - Controls who can list records
- `viewRule` - Controls who can view individual records
- `createRule` - Controls who can create records
- `updateRule` - Controls who can update records
- `deleteRule` - Controls who can delete records

Auth collections have an additional `manageRule` that allows one user to fully manage another user's data.

## Rule Values

Each rule can be set to:

- **`null` (locked)** - Only authorized superusers can perform the action (default)
- **Empty string `""`** - Anyone can perform the action (superusers, authenticated users, and guests)
- **Non-empty string** - Only users that satisfy the filter expression can perform the action

## Important Notes

1. **Rules act as filters**: API Rules also act as record filters. For example, setting `listRule` to `status = "active"` will only return active records.
2. **HTTP Status Codes**: 
   - `200` with empty items for unsatisfied `listRule`
   - `400` for unsatisfied `createRule`
   - `404` for unsatisfied `viewRule`, `updateRule`, `deleteRule`
   - `403` for locked rules when not a superuser
3. **Superuser bypass**: API Rules are ignored when the action is performed by an authorized superuser.

## Setting Rules via SDK

### Rust SDK

```rust
use bosbase::BosBase;
use serde_json::json;
use std::collections::HashMap;

let pb = BosBase::new("http://localhost:8090");
pb.admins().auth_with_password("admin@example.com", "password").await?;

// Create collection with rules
let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "articles",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            },
            {
                "name": "status",
                "type": "select",
                "options": {
                    "values": ["draft", "published"]
                },
                "maxSelect": 1
            },
            {
                "name": "author",
                "type": "relation",
                "options": {
                    "collectionId": "users"
                },
                "maxSelect": 1
            }
        ],
        "listRule": r#"@request.auth.id != "" || status = "published""#,
        "viewRule": r#"@request.auth.id != "" || status = "published""#,
        "createRule": r#"@request.auth.id != ""#,
        "updateRule": r#"author = @request.auth.id || @request.auth.role = "admin""#,
        "deleteRule": r#"author = @request.auth.id || @request.auth.role = "admin""#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Update rules
pb.collections.update(
    "articles",
    json!({
        "listRule": r#"@request.auth.id != "" && (status = "published" || status = "draft")"#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Remove rule (set to empty string for public access)
pb.collections.update(
    "articles",
    json!({
        "listRule": ""  // Anyone can list
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;

// Lock rule (set to null for superuser only)
pb.collections.update(
    "articles",
    json!({
        "deleteRule": json!(null)  // Only superusers can delete
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Filter Syntax

The syntax follows: `OPERAND OPERATOR OPERAND`

### Operators

**Comparison Operators:**
- `=` - Equal
- `!=` - NOT equal
- `>` - Greater than
- `>=` - Greater than or equal
- `<` - Less than
- `<=` - Less than or equal

**String Operators:**
- `~` - Like/Contains (auto-wraps right operand in `%` for wildcard match)
- `!~` - NOT Like/Contains

**Array Operators (Any/At least one of):**
- `?=` - Any Equal
- `?!=` - Any NOT equal
- `?>` - Any Greater than
- `?>=` - Any Greater than or equal
- `?<` - Any Less than
- `?<=` - Any Less than or equal
- `?~` - Any Like/Contains
- `?!~` - Any NOT Like/Contains

**Logical Operators:**
- `&&` - AND
- `||` - OR
- `()` - Grouping
- `//` - Single line comments

## Special Identifiers

### @request.*

Access current request data:

**@request.context** - The context where the rule is used
```rust
listRule: Some(r#"@request.context != "oauth2""#.to_string())
```

**@request.method** - HTTP request method
```rust
updateRule: Some(r#"@request.method = "PATCH""#.to_string())
```

**@request.headers.*** - Request headers (normalized to lowercase, `-` replaced with `_`)
```rust
listRule: Some(r#"@request.headers.x_token = "test""#.to_string())
```

**@request.query.*** - Query parameters
```rust
listRule: Some(r#"@request.query.page = "1""#.to_string())
```

**@request.auth.*** - Current authenticated user
```rust
listRule: Some(r#"@request.auth.id != """#.to_string())
viewRule: Some(r#"@request.auth.email = "admin@example.com""#.to_string())
updateRule: Some(r#"@request.auth.role = "admin""#.to_string())
```

**@request.body.*** - Submitted body parameters
```rust
createRule: Some(r#"@request.body.title != """#.to_string())
updateRule: Some(r#"@request.body.status:isset = false"#.to_string())  // Prevent changing status
```

### @collection.*

Target other collections that aren't directly related:

```rust
// Check if user has access to related collection
listRule: Some(r#"@request.auth.id != "" && @collection.news.categoryId ?= categoryId && @collection.news.author ?= @request.auth.id"#.to_string())
```

### @ Macros (Datetime)

All macros are UTC-based:

- `@now` - Current datetime as string
- `@second` - Current second (0-59)
- `@minute` - Current minute (0-59)
- `@hour` - Current hour (0-23)
- `@weekday` - Current weekday (0-6)
- `@day` - Current day
- `@month` - Current month
- `@year` - Current year
- `@yesterday` - Yesterday datetime
- `@tomorrow` - Tomorrow datetime
- `@todayStart` - Beginning of current day
- `@todayEnd` - End of current day
- `@monthStart` - Beginning of current month
- `@monthEnd` - End of current month
- `@yearStart` - Beginning of current year
- `@yearEnd` - End of current year

**Example:**
```rust
listRule: Some(r#"@request.body.publicDate >= @now"#.to_string())
listRule: Some(r#"created >= @todayStart && created <= @todayEnd"#.to_string())
```

## Field Modifiers

### :isset

Check if a field was submitted in the request (only for `@request.*` fields):

```rust
// Prevent changing role field
updateRule: Some(r#"@request.body.role:isset = false"#.to_string())

// Require email field
createRule: Some(r#"@request.body.email:isset = true"#.to_string())
```

### :length

Check the number of items in an array field (multiple file, select, relation):

```rust
// Check submitted array length
createRule: Some(r#"@request.body.tags:length > 1 && @request.body.tags:length <= 5"#.to_string())

// Check existing record array length
listRule: Some("categories:length = 2".to_string())
listRule: Some("documents:length >= 1".to_string())
```

### :each

Apply condition on each item in an array field:

```rust
// Check if all submitted select options contain "create"
createRule: Some(r#"@request.body.permissions:each ~ "create""#.to_string())

// Check if all existing field values have "pb_" prefix
listRule: Some(r#"tags:each ~ "pb_%""#.to_string())
```

### :lower

Perform case-insensitive string comparisons:

```rust
// Case-insensitive comparison
listRule: Some(r#"@request.body.title:lower = "test""#.to_string())
updateRule: Some(r#"status:lower ~ "active""#.to_string())
```

## geoDistance Function

Calculate Haversine distance between two geographic points in kilometers:

```rust
// Offices within 25km of location
listRule: Some("geoDistance(address.lon, address.lat, 23.32, 42.69) < 25".to_string())

// Using request data
listRule: Some("geoDistance(location.lon, location.lat, @request.query.lon, @request.query.lat) < @request.query.radius".to_string())
```

## Common Rule Examples

### Allow Only Authenticated Users

```rust
let rules = json!({
    "listRule": r#"@request.auth.id != """#,
    "viewRule": r#"@request.auth.id != """#,
    "createRule": r#"@request.auth.id != """#,
    "updateRule": r#"@request.auth.id != """#,
    "deleteRule": r#"@request.auth.id != """#
});
```

### Owner-Based Access

```rust
let rules = json!({
    "viewRule": r#"@request.auth.id != "" && author = @request.auth.id"#,
    "updateRule": r#"@request.auth.id != "" && author = @request.auth.id"#,
    "deleteRule": r#"@request.auth.id != "" && author = @request.auth.id"#
});
```

### Role-Based Access

```rust
// Assuming users have a "role" select field
let rules = json!({
    "listRule": r#"@request.auth.id != "" && @request.auth.role = "admin""#,
    "updateRule": r#"@request.auth.role = "admin" || author = @request.auth.id"#
});
```

### Public with Authentication

```rust
// Public can view published, authenticated can view all
let rules = json!({
    "listRule": r#"@request.auth.id != "" || status = "published""#,
    "viewRule": r#"@request.auth.id != "" || status = "published""#
});
```

### Filtered Results

```rust
// Only show active records
listRule: Some("status = \"active\"".to_string())

// Only show records from last 30 days
listRule: Some("created >= @yesterday".to_string())

// Only show records matching user's organization
listRule: Some(r#"@request.auth.id != "" && organization = @request.auth.organization"#.to_string())
```

## Complete Examples

### Example 1: Blog with Public/Private Posts

```rust
let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "posts",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            },
            {
                "name": "status",
                "type": "select",
                "options": {
                    "values": ["draft", "published", "private"]
                },
                "maxSelect": 1
            },
            {
                "name": "author",
                "type": "relation",
                "options": {
                    "collectionId": "users"
                },
                "maxSelect": 1
            }
        ],
        "listRule": r#"status = "published" || (@request.auth.id != "" && author = @request.auth.id)"#,
        "viewRule": r#"status = "published" || (@request.auth.id != "" && author = @request.auth.id)"#,
        "createRule": r#"@request.auth.id != ""#,
        "updateRule": r#"@request.auth.id != "" && author = @request.auth.id"#,
        "deleteRule": r#"@request.auth.id != "" && author = @request.auth.id"#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

### Example 2: Organization-Based Access

```rust
let collection = pb.collections.create(
    json!({
        "type": "base",
        "name": "documents",
        "fields": [
            {
                "name": "title",
                "type": "text",
                "required": true
            },
            {
                "name": "organization",
                "type": "relation",
                "options": {
                    "collectionId": "organizations"
                },
                "maxSelect": 1
            }
        ],
        "listRule": r#"@request.auth.id != "" && organization = @request.auth.organization"#,
        "viewRule": r#"@request.auth.id != "" && organization = @request.auth.organization"#,
        "createRule": r#"@request.auth.id != "" && @request.body.organization = @request.auth.organization"#,
        "updateRule": r#"@request.auth.id != "" && organization = @request.auth.organization"#,
        "deleteRule": r#"@request.auth.id != "" && organization = @request.auth.organization && @request.auth.role = "admin""#
    }),
    HashMap::new(),
    HashMap::new(),
    None,
    None
).await?;
```

## Best Practices

1. **Start Restrictive**: Begin with locked rules (`null`) and gradually open access
2. **Test Rules**: Test API rules thoroughly before deploying
3. **Use Special Identifiers**: Leverage `@request.auth.*` for user-based access control
4. **Combine Rules**: Use logical operators to create complex access patterns
5. **Document Rules**: Document your API rules for team understanding
6. **Regular Audits**: Regularly review and audit your API rules for security

## Related Documentation

- [Collections](./COLLECTIONS.md) - Collection configuration
- [API Records](./API_RECORDS.md) - Using filters in queries


# OpenAPI Documentation

KoproGo API documentation using OpenAPI 3.0 and Swagger UI.

## Access Swagger UI

**Development**: http://localhost:8080/swagger-ui/
**Production**: https://api.koprogo.com/swagger-ui/

**OpenAPI JSON spec**: http://localhost:8080/api-docs/openapi.json

## Current Status

✅ **Infrastructure Ready**: Swagger UI is configured and running
⚠️ **Incremental Documentation**: Currently only `/health` endpoint is documented
📝 **400+ Endpoints Available**: See `routes.rs` for complete endpoint list

## How to Add Endpoints to OpenAPI

### Step 1: Annotate Handler

Add `#[utoipa::path(...)]` annotation to your handler function.

**Example** (`health.rs`):
```rust
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

/// Health check endpoint
///
/// Returns system health status. No authentication required.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "System is healthy", body = serde_json::Value,
            example = json!({"status": "ok", "service": "koprogo-api"}))
    )
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "koprogo-api"
    }))
}
```

### Step 2: Import Handler in openapi.rs

Add import at top of `infrastructure/openapi.rs`:
```rust
use crate::infrastructure::web::handlers::health::health_check;
use crate::infrastructure::web::handlers::auth::login; // Add this
```

### Step 3: Add to paths() in OpenApi Macro

Add handler name to `paths()` list:
```rust
#[derive(OpenApi)]
#[openapi(
    // ... info, servers ...
    paths(
        health_check,
        login,  // Add this
    ),
    // ...
)]
pub struct ApiDoc;
```

### Step 4: Add DTOs to components() (if needed)

If your handler uses custom DTOs, add them to `schemas()`:
```rust
components(
    schemas(
        LoginDto,       // Add DTOs here
        TokenResponse,
    )
)
```

DTOs must derive `ToSchema`:
```rust
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}
```

## Advanced Annotations

### With Authentication

```rust
#[utoipa::path(
    post,
    path = "/api/v1/buildings",
    tag = "Buildings",
    request_body = CreateBuildingDto,
    responses(
        (status = 201, description = "Building created", body = BuildingResponseDto),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])  // Requires JWT authentication
    )
)]
```

### With Path Parameters

```rust
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}",
    tag = "Buildings",
    params(
        ("id" = uuid::Uuid, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "Building found", body = BuildingResponseDto),
        (status = 404, description = "Building not found")
    )
)]
```

### With Query Parameters

```rust
#[utoipa::path(
    get,
    path = "/api/v1/buildings",
    tag = "Buildings",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i32>, Query, description = "Items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "Buildings list", body = Vec<BuildingResponseDto>)
    )
)]
```

## Priority Endpoints to Document

Suggested order for adding documentation:

### High Priority (Authentication & Core CRUD)
1. **Auth**: login, register, refresh_token, get_current_user
2. **Buildings**: create, list, get, update, delete
3. **Units**: create, list, get, update, delete
4. **Owners**: create, list, get, update
5. **Expenses**: create, list, get, mark_paid

### Medium Priority (Legal Compliance)
6. **Budgets**: create, approve, reject, get_variance
7. **États Datés**: create, mark_generated, mark_delivered
8. **Meetings**: create, list, update, complete
9. **Documents**: upload, download, list, delete
10. **GDPR**: export_data, erase_data, rectify_data

### Lower Priority (Advanced Features)
11. **Payments**: create, refund, list
12. **Local Exchanges**: create, request, complete, leaderboard
13. **Tickets**: create, assign, resolve
14. **Resolutions**: create, cast_vote, close_voting
15. **Board Members**: elect, renew_mandate, list
16. **Quotes**: create, submit, accept, compare
17. **Payment Recovery**: create_reminder, escalate, stats
18. **Notifications**: create, mark_read, list
19. **Gamification**: award_achievement, increment_progress, leaderboard

## Available Tags

- **Health**: System health and monitoring
- **Auth**: Authentication and authorization
- **Buildings**: Building management
- **Units**: Unit management
- **Owners**: Owner management
- **Expenses**: Expense and invoice management
- **Meetings**: General assembly management
- **Budgets**: Annual budget management
- **Documents**: Document upload/download
- **GDPR**: Data privacy compliance
- **Payments**: Payment processing
- **PaymentMethods**: Stored payment methods
- **LocalExchanges**: SEL time-based exchange system
- **Notifications**: Multi-channel notifications
- **Tickets**: Maintenance request system
- **Resolutions**: Meeting voting system
- **BoardMembers**: Board of directors management
- **Quotes**: Contractor quote management
- **EtatsDates**: Property sale documentation
- **PaymentRecovery**: Automated payment reminders

## Testing OpenAPI Spec

```bash
# Run unit tests
cargo test --lib infrastructure::openapi

# Verify JSON output
curl http://localhost:8080/api-docs/openapi.json | jq .

# Test Swagger UI
open http://localhost:8080/swagger-ui/
```

## Swagger UI Features

- **Try it out**: Test endpoints directly from browser
- **Persistent auth**: JWT token saved across page reloads
- **Request duration**: See response times
- **Deep linking**: Share links to specific endpoints
- **Request/Response examples**: See real payloads

## References

- **utoipa docs**: https://docs.rs/utoipa/latest/utoipa/
- **OpenAPI 3.0 spec**: https://spec.openapis.org/oas/v3.0.0
- **Swagger UI**: https://swagger.io/tools/swagger-ui/

## Contribution Guidelines

1. Always add OpenAPI annotations when creating new endpoints
2. Include request/response examples
3. Document all possible status codes (200, 400, 401, 404, 500)
4. Use appropriate tags
5. Add security() for authenticated endpoints
6. Test annotations compile before committing

## Future Improvements

- [ ] Add all 400+ endpoints to OpenAPI spec
- [ ] Generate client SDKs (TypeScript, Python, Rust)
- [ ] Add response examples to all endpoints
- [ ] Document error response schemas
- [ ] Add webhook documentation
- [ ] Generate Postman collection from OpenAPI spec

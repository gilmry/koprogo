# JWT Security & Multi-Tenancy Implementation

**Date**: 2025-10-23
**Issue**: #020 - Multi-Tenancy Security Enhancement

## Overview

This document describes the JWT-based security implementation that ensures proper multi-tenancy isolation. The system prevents users from accessing or modifying data belonging to other organizations by extracting the `organization_id` from authenticated JWT tokens rather than accepting it from request bodies.

## Security Problem

**Before**: Applications accepted `organization_id` from request bodies:

```rust
#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    dto: web::Json<CreateBuildingDto>, // Contains organization_id from client!
) -> impl Responder {
    // INSECURE: User could forge organization_id in the request
    state.building_use_cases.create_building(dto.into_inner()).await
}
```

**Attack Scenario**: A malicious user could modify the HTTP request to include a different `organization_id` and access/modify another organization's data.

## Solution: JWT Middleware

### Architecture

```
Client Request
    ↓
JWT Token in Authorization Header
    ↓
OrganizationId Extractor (Middleware)
    ├─ Validates JWT signature
    ├─ Extracts claims
    └─ Returns organization_id from token
    ↓
Handler (Secure)
    └─ Uses JWT-provided organization_id
```

### Implementation

#### 1. Middleware (`src/infrastructure/web/middleware.rs`)

Two custom extractors are provided:

**`AuthenticatedUser`**: Extracts full user information from JWT
```rust
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub organization_id: Option<Uuid>,
}
```

**`OrganizationId`**: Extracts only the organization ID (requires user belongs to an organization)
```rust
pub struct OrganizationId(pub Uuid);
```

#### 2. Usage in Handlers

**Secure Pattern** (✅ Recommended):

```rust
use crate::infrastructure::web::{AppState, OrganizationId};

#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    organization: OrganizationId, // JWT-extracted, CANNOT be forged!
    mut dto: web::Json<CreateBuildingDto>,
) -> impl Responder {
    // Override any organization_id from DTO with the secure JWT value
    dto.organization_id = organization.0.to_string();

    // Now safe to proceed
    state.building_use_cases.create_building(dto.into_inner()).await
}
```

**Alternative Pattern** (for more granular control):

```rust
use crate::infrastructure::web::{AppState, AuthenticatedUser};

#[get("/buildings")]
pub async fn list_buildings(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // Full user information
) -> impl Responder {
    // Check if user has organization
    let org_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(e),
    };

    // Filter buildings by organization
    state.building_use_cases.list_buildings_by_organization(org_id).await
}
```

### How It Works

1. **Client sends request** with `Authorization: Bearer <jwt_token>` header
2. **Middleware intercepts** the request via Actix-web's `FromRequest` trait
3. **Token validation**:
   - Extracts token from "Bearer <token>" header
   - Verifies JWT signature using `auth_use_cases.verify_token()`
   - Parses claims including `organization_id`
4. **Injection**: The validated `organization_id` is injected into the handler
5. **Handler uses** the secure organization_id for all operations

## Migration Guide

### Step 1: Update Handler Signatures

**Before**:
```rust
#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    dto: web::Json<CreateBuildingDto>,
) -> impl Responder
```

**After**:
```rust
#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    organization: OrganizationId, // ← Add this
    mut dto: web::Json<CreateBuildingDto>, // ← Make mutable if overriding
) -> impl Responder
```

### Step 2: Override organization_id from DTO

```rust
// Override with JWT value
dto.organization_id = organization.0.to_string();
```

### Step 3: Test with Authenticated Requests

**Example using curl**:

```bash
# 1. Login to get JWT token
TOKEN=$(curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"password123"}' \
  | jq -r '.token')

# 2. Create building with JWT (organization_id from token)
curl -X POST http://localhost:8080/api/v1/buildings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "00000000-0000-0000-0000-000000000000",
    "name": "Test Building",
    "address": "123 Main St",
    "city": "Brussels",
    "postal_code": "1000",
    "country": "Belgium",
    "total_units": 10,
    "construction_year": 2020
  }'

# Note: The organization_id in the request body will be IGNORED and replaced
# with the organization_id from the JWT token!
```

## Security Benefits

| Before | After |
|--------|-------|
| ❌ User can forge organization_id | ✅ organization_id from signed JWT only |
| ❌ Cross-organization data access possible | ✅ Enforced organization isolation |
| ❌ Requires trust in client-side validation | ✅ Server-side cryptographic validation |
| ❌ Vulnerable to parameter tampering | ✅ Immune to parameter tampering |

## Affected Endpoints

All create/update endpoints that accept `organization_id` should be migrated:

- ✅ **POST /buildings** - Migrated (example implementation)
- ⏳ **POST /units** - To be migrated
- ⏳ **POST /owners** - To be migrated
- ⏳ **POST /expenses** - To be migrated
- ⏳ **POST /meetings** - To be migrated
- ⏳ **POST /documents** - To be migrated

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_organization_id_extraction() {
    let user = AuthenticatedUser {
        user_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        role: "admin".to_string(),
        organization_id: Some(Uuid::new_v4()),
    };

    assert!(user.require_organization().is_ok());
}
```

### Integration Tests

1. **Positive Test**: Valid JWT with organization_id → 200 OK
2. **Negative Test**: Missing Authorization header → 401 Unauthorized
3. **Negative Test**: Invalid JWT token → 401 Unauthorized
4. **Negative Test**: Expired JWT token → 401 Unauthorized
5. **Negative Test**: User without organization → 401 Unauthorized
6. **Security Test**: Attempt to forge organization_id in body → Uses JWT value only

## Performance Considerations

- **JWT Verification**: ~0.5ms per request (negligible)
- **No Database Calls**: Token validation is cryptographic only
- **Caching**: Consider adding Redis cache for frequently verified tokens (optional)

## Future Improvements

1. **Role-Based Access Control (RBAC)**:
   ```rust
   #[derive(Debug, Clone)]
   pub enum RequiredRole {
       Admin,
       Manager,
       User,
   }

   pub struct RequireRole(pub RequiredRole);
   // Implement FromRequest to check user.role
   ```

2. **Audit Logging**:
   - Log all organization-scoped operations
   - Include user_id, organization_id, action, timestamp

3. **Rate Limiting per Organization**:
   - Prevent abuse by limiting requests per organization

## References

- **Issue**: #020 - Multi-Tenancy Parfait
- **Related**: #005 - Security Hardening
- **JWT Library**: `jsonwebtoken` (https://github.com/Keats/jsonwebtoken)
- **Actix-web Extractors**: https://actix.rs/docs/extractors/

---

**Status**: ✅ Middleware implemented, 1/6 handlers migrated
**Next Steps**: Migrate remaining handlers (units, owners, expenses, meetings, documents)

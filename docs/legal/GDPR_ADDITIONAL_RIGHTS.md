# GDPR Additional Rights Implementation (Articles 16, 18, 21)

## Overview

This document describes the implementation of three additional GDPR rights in the KoproGo platform:
- **Article 16**: Right to Rectification
- **Article 18**: Right to Restriction of Processing
- **Article 21**: Right to Object

## Architecture Decision

These rights are implemented as **request-based workflows** rather than immediate actions, for several reasons:

1. **Validation Required**: Changes need to be verified before application
2. **Audit Trail**: Complete history of requests and approvals
3. **Admin Control**: Prevents accidental or malicious data corruption
4. **Legal Compliance**: Some objections can be rejected with compelling grounds

## Article 16 - Right to Rectification

### Domain Model

```rust
pub struct GdprRectificationRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: RectificationStatus, // Pending, Approved, Rejected, Applied
    pub changes: Vec<FieldChange>,
    pub reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

pub struct FieldChange {
    pub entity_type: String, // "User", "Owner", "Building"
    pub entity_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
}
```

### Workflow

1. **User submits correction request** via API
   - Specifies which fields are incorrect
   - Provides new correct values
   - Optional reason/justification

2. **Admin reviews request**
   - Verifies accuracy of new values
   - Checks for potential conflicts
   - Approves or rejects

3. **System applies corrections** (if approved)
   - Updates specified fields
   - Marks request as "Applied"
   - Sends confirmation email

### API Endpoints

```
POST   /api/v1/gdpr/rectification          - Submit correction request
GET    /api/v1/gdpr/rectification          - List user's requests
GET    /api/v1/gdpr/rectification/:id      - Get request details

Admin endpoints:
GET    /api/v1/admin/gdpr/rectification    - List all pending requests
PUT    /api/v1/admin/gdpr/rectification/:id/approve
PUT    /api/v1/admin/gdpr/rectification/:id/reject
POST   /api/v1/admin/gdpr/rectification/:id/apply
```

### Use Cases

```rust
- create_rectification_request(user_id, changes, reason)
- get_user_rectification_requests(user_id)
- list_pending_rectifications(admin) // SuperAdmin only
- approve_rectification(request_id, admin_id)
- reject_rectification(request_id, admin_id, reason)
- apply_rectification(request_id) // Actually update the data
```

## Article 18 - Right to Restriction of Processing

### Domain Model

```rust
pub struct GdprRestrictionRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: RestrictionStatus, // Pending, Active, Lifted, Expired, Rejected
    pub reason: RestrictionReason,
    pub justification: Option<String>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

pub enum RestrictionReason {
    AccuracyContested,      // Article 18(1)(a)
    UnlawfulProcessing,     // Article 18(1)(b)
    LegalClaims,            // Article 18(1)(c)
    ObjectionPending,       // Article 18(1)(d)
}
```

### Workflow

1. **User requests restriction**
   - Selects one of 4 legal grounds
   - Provides justification
   - May specify duration (or indefinite)

2. **Admin reviews and activates**
   - Verifies grounds are valid
   - Sets duration if applicable
   - Activates restriction

3. **System enforces restriction**
   - Data can be stored but not processed
   - Exception: with user consent or for legal claims
   - Automatic expiration if duration set

4. **Restriction lifted**
   - Admin lifts manually, OR
   - Automatic expiration, OR
   - User withdraws request

### API Endpoints

```
POST   /api/v1/gdpr/restriction            - Request restriction
GET    /api/v1/gdpr/restriction            - List user's restrictions
GET    /api/v1/gdpr/restriction/:id        - Get restriction details
DELETE /api/v1/gdpr/restriction/:id        - Withdraw restriction request

Admin endpoints:
GET    /api/v1/admin/gdpr/restriction      - List all restrictions
PUT    /api/v1/admin/gdpr/restriction/:id/activate
PUT    /api/v1/admin/gdpr/restriction/:id/lift
PUT    /api/v1/admin/gdpr/restriction/:id/reject
```

### Use Cases

```rust
- create_restriction_request(user_id, reason, justification, duration)
- get_user_restrictions(user_id)
- check_user_restriction_status(user_id) -> bool // Is processing restricted?
- activate_restriction(request_id, admin_id, duration)
- lift_restriction(request_id, admin_id)
- reject_restriction(request_id, admin_id, reason)
- expire_restrictions() // Cron job to expire old restrictions
```

### Implementation Notes

- **Middleware Check**: Add middleware to check if user has active restriction before processing operations
- **Exception Handling**: Allow processing if:
  - User gives explicit consent
  - Required for legal claims
  - Required for protection of rights of another person
- **Notification**: User must be notified before restriction is lifted

## Article 21 - Right to Object

### Domain Model

```rust
pub struct GdprObjectionRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: ObjectionStatus, // Pending, Accepted, Rejected, Partial
    pub objection_type: ObjectionType,
    pub processing_purposes: Vec<ProcessingPurpose>,
    pub justification: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

pub enum ObjectionType {
    LegitimateInterests,        // Article 21(1)
    DirectMarketing,            // Article 21(2) - ABSOLUTE RIGHT
    Profiling,                  // Article 21(3)
    AutomatedDecisionMaking,    // Article 21(4)
    ScientificResearch,         // Article 21(6)
}

pub struct ProcessingPurpose {
    pub purpose: String,
    pub accepted: Option<bool>, // Granular control per purpose
}
```

### Workflow

1. **User objects to processing**
   - Selects objection type
   - Lists affected purposes
   - Provides justification (required for non-marketing)

2. **System handles based on type**
   - **Direct Marketing**: AUTOMATIC acceptance (absolute right)
   - **Other types**: Admin review required

3. **Admin review** (for non-marketing objections)
   - Verify if compelling legitimate grounds exist
   - Accept objection OR
   - Reject with detailed justification
   - Partial acceptance possible

4. **System enforces objection**
   - Stop processing for accepted purposes
   - Update user preferences
   - Send confirmation email

### API Endpoints

```
POST   /api/v1/gdpr/objection              - Submit objection
GET    /api/v1/gdpr/objection              - List user's objections
GET    /api/v1/gdpr/objection/:id          - Get objection details

Admin endpoints:
GET    /api/v1/admin/gdpr/objection        - List all pending objections
PUT    /api/v1/admin/gdpr/objection/:id/accept
PUT    /api/v1/admin/gdpr/objection/:id/reject
PUT    /api/v1/admin/gdpr/objection/:id/partial
```

### Use Cases

```rust
- create_objection_request(user_id, objection_type, purposes, justification)
- get_user_objections(user_id)
- check_user_objections(user_id, purpose) -> bool // Is this purpose objected?
- auto_accept_marketing_objection(request_id) // Automatic for marketing
- accept_objection(request_id, admin_id)
- reject_objection(request_id, admin_id, grounds)
- partial_accept_objection(request_id, admin_id, accepted_purposes)
```

### Implementation Notes

- **Marketing Objections**: Must be processed immediately (Article 21(2))
- **Compelling Grounds**: Rejection only allowed if controller demonstrates:
  - Compelling legitimate grounds that override interests/rights of data subject, OR
  - Legal claims establishment, exercise, or defense
- **Profiling**: Linked to marketing objections (Article 21(3))
- **Research**: Can be rejected if necessary for public interest research (Article 21(6))

## Database Schema

### Tables

```sql
CREATE TABLE gdpr_rectification_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    organization_id UUID REFERENCES organizations(id),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL, -- Pending, Approved, Rejected, Applied
    reason TEXT,
    processed_at TIMESTAMPTZ,
    processed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE gdpr_rectification_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES gdpr_rectification_requests(id),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    old_value TEXT,
    new_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE gdpr_restriction_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    organization_id UUID REFERENCES organizations(id),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL, -- Pending, Active, Lifted, Expired, Rejected
    reason VARCHAR(50) NOT NULL, -- AccuracyContested, UnlawfulProcessing, etc.
    justification TEXT,
    effective_from TIMESTAMPTZ,
    effective_until TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    processed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE gdpr_objection_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    organization_id UUID REFERENCES organizations(id),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL, -- Pending, Accepted, Rejected, Partial
    objection_type VARCHAR(50) NOT NULL, -- DirectMarketing, Profiling, etc.
    justification TEXT,
    processed_at TIMESTAMPTZ,
    processed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE gdpr_objection_purposes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES gdpr_objection_requests(id),
    purpose VARCHAR(100) NOT NULL,
    accepted BOOLEAN, -- NULL = pending, TRUE = accepted, FALSE = rejected
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Indexes

```sql
CREATE INDEX idx_rectification_user_id ON gdpr_rectification_requests(user_id);
CREATE INDEX idx_rectification_status ON gdpr_rectification_requests(status);
CREATE INDEX idx_restriction_user_id ON gdpr_restriction_requests(user_id);
CREATE INDEX idx_restriction_status ON gdpr_restriction_requests(status);
CREATE INDEX idx_restriction_active ON gdpr_restriction_requests(status, effective_until);
CREATE INDEX idx_objection_user_id ON gdpr_objection_requests(user_id);
CREATE INDEX idx_objection_status ON gdpr_objection_requests(status);
CREATE INDEX idx_objection_type ON gdpr_objection_requests(objection_type);
```

## Audit Logging

All GDPR request operations must be logged in the audit_logs table:

```rust
enum AuditEventType {
    // Existing...
    GdprDataExported,
    GdprDataErased,

    // New events
    GdprRectificationRequested,
    GdprRectificationApproved,
    GdprRectificationRejected,
    GdprRectificationApplied,

    GdprRestrictionRequested,
    GdprRestrictionActivated,
    GdprRestrictionLifted,
    GdprRestrictionRejected,
    GdprRestrictionExpired,

    GdprObjectionRequested,
    GdprObjectionAccepted,
    GdprObjectionRejected,
    GdprObjectionPartiallyAccepted,
}
```

## Email Notifications

Extend EmailService with new methods:

```rust
impl EmailService {
    // Rectification
    pub async fn send_rectification_submitted(&self, user_email, request_id)
    pub async fn send_rectification_approved(&self, user_email, changes)
    pub async fn send_rectification_rejected(&self, user_email, reason)
    pub async fn send_rectification_applied(&self, user_email, changes)

    // Restriction
    pub async fn send_restriction_submitted(&self, user_email, reason)
    pub async fn send_restriction_activated(&self, user_email, duration)
    pub async fn send_restriction_lifted(&self, user_email)
    pub async fn send_restriction_rejected(&self, user_email, reason)

    // Objection
    pub async fn send_objection_submitted(&self, user_email, objection_type)
    pub async fn send_objection_accepted(&self, user_email, purposes)
    pub async fn send_objection_rejected(&self, user_email, grounds)
}
```

## Rate Limiting

Apply same rate limits as other GDPR operations (10 requests/hour per user):

```rust
// In middleware.rs
pub struct GdprRateLimit {
    // ... existing fields

    // Add new GDPR paths to rate limiting
    rate_limited_paths: vec![
        "/api/v1/gdpr/export",
        "/api/v1/gdpr/erase",
        "/api/v1/gdpr/rectification",
        "/api/v1/gdpr/restriction",
        "/api/v1/gdpr/objection",
        // Admin paths excluded from rate limiting
    ],
}
```

## Testing Strategy

### Unit Tests
- Domain entity state transitions
- Business logic validation
- Edge cases (expired restrictions, invalid reasons)

### Integration Tests
- Database CRUD operations
- Repository implementations
- Use case orchestration

### E2E Tests
- Full API workflows
- Admin approval/rejection
- Email notifications sent
- Audit logs created

### BDD Scenarios

```gherkin
Feature: GDPR Right to Rectification (Article 16)

  Scenario: User requests personal data correction
    Given I am an authenticated user
    And my email is incorrect in the system
    When I submit a rectification request for my email
    Then the request should be marked as "Pending"
    And I should receive a confirmation email
    And an audit log should be created

Feature: GDPR Right to Restriction (Article 18)

  Scenario: User contests data accuracy and requests restriction
    Given I am an authenticated user
    When I request restriction with reason "AccuracyContested"
    Then the request should be pending admin approval
    And an admin should be notified

Feature: GDPR Right to Object (Article 21)

  Scenario: User objects to direct marketing
    Given I am an authenticated user
    When I object to "DirectMarketing"
    Then the objection should be automatically accepted
    And marketing processing should stop immediately
    And I should receive confirmation
```

## Implementation Phases

### Phase 8.1 - Domain Layer (DONE ✅)
- [x] Domain entities created
- [x] Unit tests written (14 tests)
- [x] Added to entities/mod.rs

### Phase 8.2 - Database Layer (TODO)
- [ ] Create migrations for 4 new tables
- [ ] Implement repository traits
- [ ] Implement PostgreSQL repositories
- [ ] Write integration tests

### Phase 8.3 - Application Layer (TODO)
- [ ] Define DTOs for API
- [ ] Create use cases for each right
- [ ] Add audit event types
- [ ] Write use case unit tests

### Phase 8.4 - Infrastructure Layer (TODO)
- [ ] Implement HTTP handlers
- [ ] Add routes to routes.rs
- [ ] Extend email service
- [ ] Update rate limiting middleware

### Phase 8.5 - Testing & Documentation (TODO)
- [ ] E2E tests
- [ ] BDD scenarios
- [ ] Update API documentation
- [ ] Add CHANGELOG entries

## Compliance Notes

### Article 16 - Right to Rectification
- Must be completed "without undue delay"
- Controller must communicate corrections to recipients (if possible)
- User can request list of recipients

### Article 18 - Right to Restriction
- User must be informed before restriction is lifted
- 4 specific grounds (cannot be restricted for other reasons)
- During restriction: storage allowed, other processing requires consent

### Article 21 - Right to Object
- Direct marketing objection is ABSOLUTE (cannot be rejected)
- Other objections can be overridden by compelling legitimate grounds
- Must stop processing unless controller demonstrates grounds
- Applies to legitimate interests and public interest tasks

## Security Considerations

1. **Authorization**: Only user or SuperAdmin can access requests
2. **Data Integrity**: Old values stored for audit trail
3. **Approval Workflow**: Prevents unauthorized data changes
4. **Rate Limiting**: Prevents abuse of request system
5. **Audit Logging**: Complete trail of all operations
6. **Email Verification**: User notified of all actions

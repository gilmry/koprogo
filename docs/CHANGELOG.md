# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Database
- Added GDPR anonymization fields (`is_anonymized`, `anonymized_at`) to `users` and `owners` tables for GDPR Article 17 compliance
- Added indexes `idx_users_is_anonymized` and `idx_owners_is_anonymized` for efficient GDPR queries

### Features
- Added GDPR export domain entities (`GdprExport`, `UserData`, `OwnerData`, `RelatedData`) for Article 15 compliance
  - Pure domain layer with no external dependencies
  - JSON serialization support
  - 9 unit tests (100% coverage)
- **Added GDPR domain entities for Articles 16, 18, 21** (Phase 8)
  - `GdprRectificationRequest`: Right to rectification (Article 16)
    - Track field changes with old/new values
    - Approval workflow (Pending → Approved → Applied)
    - Support for User and Owner entity corrections
    - 4 unit tests
  - `GdprRestrictionRequest`: Right to restriction of processing (Article 18)
    - 4 grounds for restriction (accuracy contested, unlawful processing, legal claims, objection pending)
    - Temporal restrictions with duration support
    - Active status tracking with expiration
    - 5 unit tests
  - `GdprObjectionRequest`: Right to object (Article 21)
    - 5 objection types (legitimate interests, direct marketing, profiling, automated decisions, research)
    - Absolute right for direct marketing
    - Partial acceptance support (some purposes accepted, others rejected)
    - 5 unit tests
- Added `GdprRepository` port trait for data aggregation and anonymization operations
  - 6 methods: aggregate_user_data, anonymize_user/owner, find_owner_ids, check_legal_holds, is_anonymized
  - Mock implementation with 4 unit tests
- Added GDPR DTOs for API endpoints (`GdprExportResponseDto`, `GdprEraseRequestDto`, `GdprEraseResponseDto`)
  - Full JSON serialization support
  - Domain-to-DTO conversions with From traits
  - RFC3339 timestamp formatting
  - 6 unit tests (100% coverage)
- Added GDPR use cases (`GdprUseCases`) for business logic orchestration
  - `export_user_data()`: Export all personal data with authorization
  - `erase_user_data()`: Anonymize user + owners with validation
  - `can_erase_user()`: Check legal holds
  - Authorization: self-service + SuperAdmin bypass
  - Legal holds checking, partial anonymization handling
  - 9 unit tests with mocks (100% coverage)
- Implemented PostgreSQL GDPR repository (`PostgresGdprRepository`)
  - Full implementation of `GdprRepository` trait (6 methods)
  - Multi-table JOIN queries for comprehensive data aggregation
  - SQL UPDATE statements for user/owner anonymization
  - Email-based owner discovery and legal holds validation
  - Compile-time verified queries with sqlx macro
  - Private helper methods for data fetching
  - Fixed domain entity: `OwnerData.organization_id` now `Option<Uuid>`
  - Updated .sqlx query cache for CI/CD compatibility

### API
- Added GDPR REST API endpoints for data privacy compliance
  - `GET /api/v1/gdpr/export` - Export user personal data (Article 15)
  - `DELETE /api/v1/gdpr/erase` - Request data erasure (Article 17)
  - `GET /api/v1/gdpr/can-erase` - Check erasure eligibility
- **Added GDPR Admin endpoints (SuperAdmin only)**
  - `GET /api/v1/admin/gdpr/audit-logs` - List audit logs with pagination/filters
  - `GET /api/v1/admin/gdpr/users/:id/export` - Admin-initiated data export
  - `DELETE /api/v1/admin/gdpr/users/:id/erase` - Admin-initiated data erasure
- Implemented HTTP handlers (`gdpr_handlers.rs`, `admin_gdpr_handlers.rs`)
  - `export_user_data()`: Full data export with authentication
  - `erase_user_data()`: Anonymization with legal holds validation
  - `can_erase_user()`: Pre-flight erasure check
  - `list_audit_logs()`: Paginated audit log viewing with filtering
  - `admin_export_user_data()`: SuperAdmin export any user
  - `admin_erase_user_data()`: SuperAdmin erase any user
- Integrated GdprUseCases into AppState and routes
- Updated E2E test setup with GDPR use cases
- All endpoints protected by JWT authentication (AuthenticatedUser middleware)
- SuperAdmin bypass for cross-organization access
- Audit logging includes `admin_initiated` flag for admin operations

### Security
- GDPR endpoints implement proper authorization (self-service + SuperAdmin)
- Legal holds validation prevents premature data erasure
- HTTP status codes follow security best practices (401, 403, 409, 410)
- **Audit log persistence with 7-year retention** (GDPR Article 30 compliance)
  - All GDPR operations logged to `audit_logs` table with database persistence
  - `AuditLogger` service for async logging (stdout + database)
  - `PostgresAuditLogRepository` with 6 methods (create, find_by_id, find_all_paginated, find_recent, find_failed_operations, delete_older_than, count_by_filters)
  - 5 GDPR event types: GdprDataExported, GdprDataExportFailed, GdprDataErased, GdprDataErasureFailed, GdprErasureCheckRequested
  - `retention_until` field set to NOW() + 7 years by default (Belgium GDPR requirement)
  - Integrated into AppState and all GDPR handlers
- **Rate limiting for GDPR endpoints** (Abuse prevention)
  - `GdprRateLimit` middleware limits GDPR operations to 10 requests/hour per user
  - Only applies to `/api/v1/gdpr/*` and `/api/v1/admin/gdpr/*` endpoints
  - Returns HTTP 429 Too Many Requests with Retry-After header when limit exceeded
  - Uses in-memory tracking with automatic window reset
  - 3 unit tests validating rate limiting behavior
- **IP address and User Agent capture in audit logs** (GDPR Article 30 compliance)
  - All GDPR endpoints now capture client IP and User-Agent for audit trails
  - Supports X-Forwarded-For and X-Real-IP headers for proxy/load balancer scenarios
  - Falls back to peer address if headers not available
  - Client info stored in `audit_logs` table for forensic analysis
  - Applied to all 5 GDPR handlers (export, erase, can-erase, admin export, admin erase)
- **Email notifications for GDPR operations** (User transparency)
  - `EmailService` for sending GDPR-related notifications via SMTP
  - Data export completion emails with security warnings
  - Data erasure confirmation emails with anonymization summary
  - Admin-initiated operation notifications
  - Configurable via environment variables (SMTP_ENABLED, SMTP_HOST, etc.)
  - Falls back to logging when email disabled (development mode)
  - Uses `lettre` crate with async support and TLS encryption

### Tests
- All 186 unit tests passing (3 GDPR handler tests + 1 AuditLogger test + 3 rate limit tests)
- **2 new E2E tests for audit log persistence** (`tests/e2e_gdpr_audit.rs`)
  - Verifies audit logs are created in database
  - Validates 7-year retention policy
  - Tests GdprDataExported and GdprErasureCheckRequested events
- **BDD Cucumber scenarios for GDPR operations** (`tests/features/gdpr.feature`)
  - 15 scenarios documenting GDPR workflows (Articles 15 & 17)
  - User data export scenarios with owner records and unit ownerships
  - Data erasure scenarios with legal holds validation
  - SuperAdmin cross-organization access scenarios
  - Security scenarios (rate limiting, audit logging, cross-org access)
  - 25+ step definitions in `tests/bdd.rs`
  - All BDD tests compile with zero warnings
- Zero clippy warnings
- Code formatted with cargo fmt
- E2E test infrastructure ready for GDPR scenarios

### Documentation

### Fixed

### Changed

### Removed

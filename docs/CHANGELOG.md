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
- Implemented HTTP handlers (`gdpr_handlers.rs`)
  - `export_user_data()`: Full data export with authentication
  - `erase_user_data()`: Anonymization with legal holds validation
  - `can_erase_user()`: Pre-flight erasure check
- Integrated GdprUseCases into AppState and routes
- Updated E2E test setup with GDPR use cases
- All endpoints protected by JWT authentication (AuthenticatedUser middleware)
- SuperAdmin bypass for cross-organization access

### Security
- GDPR endpoints implement proper authorization (self-service + SuperAdmin)
- Legal holds validation prevents premature data erasure
- HTTP status codes follow security best practices (401, 403, 409, 410)

### Tests
- All 179 unit tests passing (3 GDPR handler tests added)
- Zero clippy warnings
- Code formatted with cargo fmt
- E2E test infrastructure ready for GDPR scenarios

### Documentation

### Fixed

### Changed

### Removed

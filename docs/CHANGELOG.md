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

### API

### Security

### Tests

### Documentation

### Fixed

### Changed

### Removed

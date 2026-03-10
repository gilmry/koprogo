========================================
GDPR Compliance Documentation
========================================

.. contents:: Table of Contents
   :depth: 3

Overview
========

KoproGo implements complete GDPR (General Data Protection Regulation)
compliance for Belgian ASBL (copropriete) management. The system covers
all key GDPR articles relevant to property management.

Implemented Articles
====================

+----------+-------------------------------+-------------------------------+
| Article  | Right                         | Endpoint                      |
+==========+===============================+===============================+
| Art. 15  | Right to Access               | ``GET /gdpr/export``          |
+----------+-------------------------------+-------------------------------+
| Art. 16  | Right to Rectification        | ``PUT /gdpr/rectify``         |
+----------+-------------------------------+-------------------------------+
| Art. 17  | Right to Erasure              | ``DELETE /gdpr/erase``        |
+----------+-------------------------------+-------------------------------+
| Art. 18  | Right to Restriction          | ``PUT /gdpr/restrict-         |
|          |                               | processing``                  |
+----------+-------------------------------+-------------------------------+
| Art. 21  | Right to Object               | ``PUT /gdpr/marketing-        |
|          |                               | preference``                  |
+----------+-------------------------------+-------------------------------+
| Art. 30  | Records of Processing         | Audit logs on all operations  |
+----------+-------------------------------+-------------------------------+

Architecture
============

The GDPR system follows hexagonal architecture::

    Domain Layer (user.rs)
      - rectify_data(), restrict_processing(), set_marketing_opt_out()
      - can_process_data(), can_send_marketing() checks

    Application Layer (gdpr_use_cases.rs)
      - Authorization: users can only modify their own data
      - Orchestrates repository calls + audit logging

    Infrastructure Layer
      - gdpr_handlers.rs: REST API endpoints (6 user + 3 admin)
      - gdpr_repository_impl.rs: PostgreSQL persistence
      - Audit event logging (async, non-blocking)

API Reference
=============

User Self-Service Endpoints
---------------------------

All endpoints require authentication. Users can only access their own data.

**GET /api/v1/gdpr/export** - Export Personal Data (Art. 15)

Returns all personal data in JSON format:

- Profile information (name, email, phone)
- Unit ownership history
- Payment records
- Notification preferences
- Audit log entries

Response: ``200 OK`` with JSON export data.

**PUT /api/v1/gdpr/rectify** - Correct Personal Data (Art. 16)

Request body::

    {
        "email": "new@email.com",        // optional
        "first_name": "Corrected",        // optional
        "last_name": "Name"               // optional
    }

At least one field must be provided. Email format validated.

Responses:

- ``200 OK``: Data corrected successfully
- ``400 Bad Request``: No fields provided or invalid email format
- ``404 Not Found``: User not found

**DELETE /api/v1/gdpr/erase** - Erase Personal Data (Art. 17)

Anonymizes all personal data. Cannot be undone.

Pre-check available: ``GET /api/v1/gdpr/can-erase`` verifies no legal
holds prevent erasure (e.g., pending financial obligations).

Responses:

- ``200 OK``: Data anonymized, user session invalidated
- ``400 Bad Request``: Legal hold prevents erasure
- ``404 Not Found``: User not found

**PUT /api/v1/gdpr/restrict-processing** - Restrict Processing (Art. 18)

Limits data processing temporarily. Sets ``processing_restricted = true``.

When restricted:

- ``can_process_data()`` returns ``false``
- Marketing and non-essential processing disabled
- Core legal obligations (accounting, tax) continue

Responses:

- ``200 OK``: Processing restricted
- ``400 Bad Request``: Already restricted

**PUT /api/v1/gdpr/marketing-preference** - Object to Marketing (Art. 21)

Request body::

    {
        "opt_out": true
    }

Sets marketing opt-out preference. When opted out:

- ``can_send_marketing()`` returns ``false``
- No promotional emails or notifications sent
- Transactional notifications (payment due, meeting invite) continue

Admin Endpoints
---------------

SuperAdmin-only endpoints for GDPR administration.

**GET /api/v1/gdpr/admin/audit-logs** - View Audit Logs

Returns all GDPR-related audit events with IP address and user-agent tracking.

**POST /api/v1/gdpr/admin/export/:user_id** - Admin Export User Data

Export any user's data (for regulatory requests).

**DELETE /api/v1/gdpr/admin/erase/:user_id** - Admin Erase User Data

Anonymize any user's data (for regulatory compliance).

Database Schema
===============

User table GDPR fields (migration ``20251120000000``)::

    processing_restricted     BOOLEAN DEFAULT FALSE
    processing_restricted_at  TIMESTAMPTZ
    marketing_opt_out         BOOLEAN DEFAULT FALSE
    marketing_opt_out_at      TIMESTAMPTZ

Partial indexes for admin queries::

    idx_users_processing_restricted  WHERE processing_restricted = TRUE
    idx_users_marketing_opt_out      WHERE marketing_opt_out = TRUE

Audit Events
============

All GDPR operations are logged with:

- Event type (e.g., ``GdprDataExported``, ``GdprDataRectified``)
- User ID (who performed the action)
- Target user ID (whose data was affected)
- IP address and user-agent
- Timestamp

Event types:

- ``GdprDataExported``, ``GdprDataExportFailed``
- ``GdprDataRectified``, ``GdprDataRectificationFailed``
- ``GdprDataErased``, ``GdprDataErasureFailed``
- ``GdprProcessingRestricted``, ``GdprProcessingRestrictionFailed``
- ``GdprMarketingOptOut``, ``GdprMarketingOptIn``, ``GdprMarketingPreferenceChangeFailed``

Frontend Components
===================

**User GDPR Panel** (``GdprDataPanel.svelte``):

- Data export with preview modal and download button
- Account erasure with confirmation dialog
- Data rectification form (email, first name, last name)
- Processing restriction toggle
- Marketing preference toggle
- Route: ``/settings/gdpr``

**Admin GDPR Panel** (``AdminGdprPanel.svelte``):

- User search with email filter
- Per-user export and erase actions
- Audit log viewer with event type filtering
- Route: ``/admin/gdpr``

E2E Tests: ``frontend/tests/e2e/Gdpr.spec.ts`` (5 scenarios).

Security
========

- **Rate limiting**: 5 GDPR requests per 15 minutes per IP
- **Authentication**: JWT required for all endpoints
- **Authorization**: Users can only access/modify their own data
- **Audit trail**: All operations logged with IP/user-agent (Art. 30)
- **Encryption**: Data at rest encrypted via LUKS (AES-XTS-512)
- **Backup encryption**: GPG-encrypted backups with S3 off-site storage

Belgian Legal Context
=====================

- **Belgian Data Protection Authority (APD/GBA)**: Supervisory authority
- **ASBL specifics**: Copropriete data processing under legitimate interest (Art. 6.1.f)
- **Retention periods**: Financial records 7 years (Belgian accounting law), personal data until erasure request
- **Data processor agreements**: Required with Stripe (payments), email providers

File Locations
==============

- Domain: ``backend/src/domain/entities/user.rs`` (GDPR methods)
- Use cases: ``backend/src/application/use_cases/gdpr_use_cases.rs``
- Handlers: ``backend/src/infrastructure/web/handlers/gdpr_handlers.rs``
- DTOs: ``backend/src/application/dto/gdpr_dto.rs``
- Migration: ``backend/migrations/20251120000000_add_gdpr_complementary_fields.sql``
- Frontend panel: ``frontend/src/components/GdprDataPanel.svelte``
- Admin panel: ``frontend/src/components/admin/AdminGdprPanel.svelte``
- E2E tests: ``frontend/tests/e2e/Gdpr.spec.ts``
- BDD tests: ``backend/tests/features/gdpr.feature``

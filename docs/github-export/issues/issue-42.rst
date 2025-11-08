===============================================================================
Issue #42: feat: Implement GDPR data export & deletion (Right to be forgotten)
===============================================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: phase:vps,track:software priority:critical
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/42>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   KoproGo processes personal data (names, emails, addresses, phone numbers) subject to **GDPR** (General Data Protection Regulation). Current implementation has:
   
   ✅ **Implemented:**
   - Audit logging foundation (`infrastructure/audit.rs`)
   - Data redaction in logs (IP addresses, errors)
   - Organization-scoped data isolation
   
   ❌ **Missing:**
   - Right to access (data export)
   - Right to erasure (right to be forgotten)
   - Right to data portability
   - Consent management
   - Data retention policies
   
   ## GDPR Requirements
   
   **Article 15 - Right of Access:**
   User can request a copy of all their personal data.
   
   **Article 17 - Right to Erasure:**
   User can request deletion of their personal data ("right to be forgotten").
   
   **Article 20 - Right to Data Portability:**
   User can receive their data in a structured, machine-readable format.
   
   ## Objective
   
   Implement GDPR-compliant data export and deletion mechanisms.
   
   ## Implementation Plan
   
   ### 1. Data Export (Right to Access)
   
   **New API endpoint:** `GET /api/v1/gdpr/export`
   
   **Response format:** JSON with all user-related data
   ```json
   {
     "export_date": "2025-10-27T10:00:00Z",
     "user": {
       "id": "uuid",
       "email": "user@example.com",
       "first_name": "John",
       "last_name": "Doe",
       "role": "Owner",
       "created_at": "2025-01-15T09:00:00Z"
     },
     "owner_profile": {
       "id": "uuid",
       "address": "...",
       "phone": "...",
       // Full owner data
     },
     "units_owned": [
       {
         "building": "Building A",
         "unit_number": "101",
         "ownership_percentage": 50.0,
         "start_date": "2023-01-01"
       }
     ],
     "expenses": [
       {
         "description": "Maintenance fee Q1 2025",
         "amount": 350.00,
         "due_date": "2025-03-31",
         "paid": true
       }
     ],
     "documents": [
       {
         "title": "Meeting Minutes Jan 2025",
         "type": "MeetingMinutes",
         "uploaded_at": "2025-01-20"
       }
     ],
     "meetings_attended": [...],
     "audit_log": [
       // All actions performed by this user
     ]
   }
   ```
   
   **Implementation:**
   - **Use Case:** `GdprUseCases::export_user_data(user_id: Uuid)`
   - Aggregate data from all entities (User, Owner, UnitOwner, Expense, Meeting, Document, AuditLog)
   - Format as structured JSON
   - Optional: Generate PDF summary
   
   **Files:**
   - `backend/src/application/use_cases/gdpr_use_cases.rs` (new)
   - `backend/src/infrastructure/web/handlers/gdpr_handlers.rs` (new)
   - `backend/src/application/dto/gdpr_dto.rs` (new)
   
   ### 2. Data Deletion (Right to Erasure)
   
   **New API endpoint:** `DELETE /api/v1/gdpr/erase`
   
   **Process:**
   1. Verify user authentication
   2. Check if deletion is legally permissible (no legal obligations to retain)
   3. Perform cascading deletion or anonymization
   4. Log deletion in audit trail (compliance requirement)
   5. Send confirmation email
   
   **Deletion Strategy:**
   
   **Anonymization vs. Deletion:**
   - **Financial records** (Expenses): Keep for legal compliance (7 years Belgium), **anonymize** owner linkage
   - **Meeting records**: Keep for legal compliance, **anonymize** personal data
   - **Audit logs**: Keep for compliance, **redact** personal identifiers
   - **User account**: **Delete** after legal retention periods
   
   **Database changes:**
   ```sql
   -- Add anonymization flag
   ALTER TABLE owners ADD COLUMN is_anonymized BOOLEAN DEFAULT FALSE;
   ALTER TABLE users ADD COLUMN deletion_requested_at TIMESTAMP;
   ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP;
   
   -- Anonymization function
   CREATE OR REPLACE FUNCTION anonymize_owner(owner_id UUID) RETURNS VOID AS $$
   BEGIN
       UPDATE owners
       SET
           first_name = 'Anonymized',
           last_name = 'User',
           email = CONCAT('anonymized-', owner_id, '@deleted.local'),
           phone = NULL,
           address = NULL,
           is_anonymized = TRUE
       WHERE id = owner_id;
   END;
   $$ LANGUAGE plpgsql;
   ```
   
   **Implementation:**
   - **Use Case:** `GdprUseCases::erase_user_data(user_id: Uuid)`
   - Soft delete user account (mark `deleted_at`)
   - Anonymize linked owner profile
   - Remove personal data from audit logs (keep action types)
   - Schedule hard deletion after legal retention period (7 years)
   
   **Cascade rules:**
   - User → Soft delete, mark as anonymized
   - Owner → Anonymize personal fields
   - UnitOwner → Keep ownership records (legal), anonymize owner reference
   - Documents uploaded by user → Delete files, keep metadata (anonymized)
   - Audit logs → Redact personal identifiers, keep event types
   
   ### 3. Consent Management (Optional, Future)
   
   **New table:** `user_consents`
   ```sql
   CREATE TABLE user_consents (
       id UUID PRIMARY KEY,
       user_id UUID NOT NULL REFERENCES users(id),
       consent_type VARCHAR(50) NOT NULL, -- 'terms_of_service', 'privacy_policy', 'marketing'
       version VARCHAR(20) NOT NULL,
       granted_at TIMESTAMP NOT NULL,
       revoked_at TIMESTAMP,
       ip_address VARCHAR(45),
       user_agent TEXT
   );
   ```
   
   **Purpose:**
   - Track user consent for data processing
   - Allow consent withdrawal
   - Audit consent history
   
   **Deferred to future sprint** (not critical for MVP)
   
   ### 4. Data Retention Policies
   
   **Configuration:** `backend/.env`
   ```
   GDPR_RETENTION_AUDIT_LOGS_DAYS=2555  # 7 years
   GDPR_RETENTION_FINANCIAL_DAYS=2555   # 7 years (Belgium law)
   GDPR_RETENTION_DELETED_USERS_DAYS=30 # Grace period before anonymization
   ```
   
   **Implementation:**
   - Cron job to purge old audit logs (keep last 7 years)
   - Cron job to anonymize soft-deleted users after 30 days
   - Cron job to hard-delete anonymized data after legal retention period
   
   **File:** `backend/src/infrastructure/cron/gdpr_retention.rs` (new)
   
   ### 5. Admin Dashboard for GDPR Requests
   
   **New page:** `frontend/src/pages/admin/gdpr-requests.astro`
   
   **Features:**
   - List all user data export requests
   - List all deletion requests
   - Approve/deny deletion requests (SuperAdmin only)
   - Download exported data
   - Audit trail of GDPR actions
   
   **Component:** `frontend/src/components/admin/GdprRequestList.svelte`
   
   ### 6. User-Facing GDPR Controls
   
   **New page:** `frontend/src/pages/settings/privacy.astro`
   
   **Features:**
   - Button: "Download my data" (triggers export)
   - Button: "Delete my account" (confirmation modal)
   - Display current consents
   - Revoke consents (if implemented)
   
   **Component:** `frontend/src/components/settings/PrivacySettings.svelte`
   
   ## Testing & Validation
   
   - [ ] Export generates complete JSON with all user data
   - [ ] Export includes data from all entities (User, Owner, Units, Expenses, Meetings, Documents)
   - [ ] Deletion anonymizes owner profile correctly
   - [ ] Deletion does not break financial records integrity
   - [ ] Audit logs record GDPR actions (export, deletion)
   - [ ] Email confirmation sent after deletion
   - [ ] Retention cron job purges old data correctly
   - [ ] SuperAdmin can review GDPR requests
   
   ## Security Considerations
   
   - **Authentication required**: Only user can export/delete their own data (or SuperAdmin)
   - **Confirmation required**: Deletion requires password re-authentication + confirmation email
   - **Audit trail**: All GDPR actions logged in audit_logs
   - **Irreversible warning**: UI warns user deletion is permanent
   - **Grace period**: 30-day soft delete before anonymization
   
   ## Documentation
   
   - [ ] Update `CLAUDE.md` with GDPR implementation
   - [ ] Create `docs/GDPR_COMPLIANCE.md` with procedures
   - [ ] Document data retention policies
   - [ ] Create user-facing privacy policy update
   - [ ] Document anonymization procedures
   
   ## Legal Compliance Checklist
   
   - [ ] **Right to access** (Article 15) - Data export API
   - [ ] **Right to erasure** (Article 17) - Deletion API
   - [ ] **Right to data portability** (Article 20) - JSON export format
   - [ ] **Data retention** (Article 5) - Retention policies enforced
   - [ ] **Audit trail** (Article 30) - GDPR actions logged
   - [ ] **Notification** (Article 19) - User notified of deletion
   
   ## Acceptance Criteria
   
   - [ ] User can export all their personal data via API/UI
   - [ ] User can request account deletion via API/UI
   - [ ] Deletion anonymizes personal data while preserving legal records
   - [ ] Audit logs track all GDPR actions
   - [ ] SuperAdmin dashboard shows GDPR requests
   - [ ] Retention policies enforced via cron jobs
   - [ ] Email confirmations sent for export/deletion
   - [ ] Documentation complete
   - [ ] Legal review passed (if applicable)
   
   ## Migration
   
   **New database migration:**
   `backend/migrations/XXX_add_gdpr_fields.sql`
   ```sql
   -- Add anonymization flags
   ALTER TABLE owners ADD COLUMN is_anonymized BOOLEAN DEFAULT FALSE;
   ALTER TABLE users ADD COLUMN deletion_requested_at TIMESTAMP;
   ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP;
   
   -- Add consent table (optional, future)
   CREATE TABLE user_consents (...);
   ```
   
   ## Effort Estimate
   
   **Medium** (2-3 days)
   - Day 1: Data export use case + API endpoint
   - Day 2: Data deletion use case + anonymization logic + migration
   - Day 3: Frontend UI + admin dashboard + testing
   
   ## Related
   
   - Supports: GDPR compliance
   - Depends on: Audit logging (already implemented)
   - Enables: EU market deployment
   
   ## References
   
   - GDPR full text: https://gdpr-info.eu/
   - Article 15 (Right to access): https://gdpr-info.eu/art-15-gdpr/
   - Article 17 (Right to erasure): https://gdpr-info.eu/art-17-gdpr/
   - Belgian data retention law: https://www.autoriteprotectiondonnees.be/

.. raw:: html

   </div>


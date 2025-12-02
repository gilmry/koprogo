==========================================================================================
Issue #64: GDPR Article 21: Implement direct marketing objection features (Phase 2 - K3s)
==========================================================================================

:State: **CLOSED**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-30
:Updated: 2025-11-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/64>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   KoproGo is an open-source property management platform without marketing features. However, GDPR Article 21 (Right to Object) requires implementing direct marketing objection capabilities for compliance, even if not actively used.
   
   ## Article 21(2) - Direct Marketing
   
   > The data subject shall have the right to object at any time to processing of personal data concerning him or her for direct marketing purposes, which includes profiling to the extent that it is related to such direct marketing.
   
   This is an **absolute right** - objections cannot be rejected.
   
   ## Required Implementation
   
   ### 1. Database Schema
   Already designed in `docs/GDPR_ADDITIONAL_RIGHTS.md`:
   - `gdpr_objection_requests` table
   - `gdpr_objection_purposes` table
   - Support for `DirectMarketing` objection type
   
   ### 2. Domain Layer (✅ DONE - Phase 8.1)
   - `GdprObjectionRequest` entity created
   - `ObjectionType::DirectMarketing` enum variant
   - Auto-acceptance logic for marketing objections
   - 5 unit tests passing
   
   ### 3. Backend Features (TODO)
   
   #### API Endpoints
   ```
   POST   /api/v1/gdpr/objection              - Submit objection
   GET    /api/v1/gdpr/objection              - List user's objections
   GET    /api/v1/gdpr/objection/:id          - Get objection details
   
   Admin endpoints:
   GET    /api/v1/admin/gdpr/objection        - List all objections (auto-approved)
   ```
   
   #### Use Cases
   ```rust
   - create_marketing_objection(user_id, purposes) -> auto_accept()
   - check_marketing_consent(user_id, purpose) -> bool
   - get_user_objections(user_id)
   ```
   
   #### Email Notifications
   ```rust
   EmailService::send_marketing_objection_confirmed(user_email, purposes)
   ```
   
   ### 4. User Preferences System
   
   Create a `user_preferences` table to track marketing consent:
   ```sql
   CREATE TABLE user_preferences (
       id UUID PRIMARY KEY,
       user_id UUID NOT NULL REFERENCES users(id),
       preference_key VARCHAR(100) NOT NULL,
       preference_value BOOLEAN NOT NULL,
       updated_at TIMESTAMPTZ NOT NULL,
       UNIQUE(user_id, preference_key)
   );
   
   -- Example preferences:
   -- 'marketing.email.enabled'
   -- 'marketing.sms.enabled'
   -- 'marketing.profiling.enabled'
   ```
   
   ### 5. Middleware Integration
   
   Add middleware to check marketing preferences before:
   - Sending emails (if we add email features later)
   - SMS notifications (if we add SMS features later)
   - Analytics/profiling (if we add tracking)
   
   ### 6. Frontend UI (Phase 10-11)
   
   Privacy settings page:
   - [ ] Toggle switches for marketing preferences
   - [ ] "Unsubscribe from all marketing" button
   - [ ] History of objection requests
   - [ ] Clear explanation of data usage
   
   ## Implementation Priority
   
   **Priority: Medium** (P2)
   
   **Reasoning:**
   - Legal requirement for GDPR compliance
   - No active marketing currently, but framework needed for future
   - Users should have control even if features unused
   - Relatively simple to implement (mostly CRUD + auto-approval)
   
   ## Roadmap Placement
   
   **Phase 2 - K3s Infrastructure** (Mar - May 2026)
   - Fits well with "Community Features" milestone
   - After core platform is stable (Phase 1 VPS complete)
   - Before mobile app launch (Phase 3)
   
   ## Acceptance Criteria
   
   - [ ] Database migrations created
   - [ ] Repository implementations with tests
   - [ ] Use cases with auto-approval logic
   - [ ] API endpoints (user + admin)
   - [ ] Email notifications sent
   - [ ] Audit logging for all operations
   - [ ] Rate limiting applied (same as other GDPR endpoints)
   - [ ] Integration tests passing
   - [ ] E2E tests with Playwright
   - [ ] Frontend privacy settings page
   - [ ] Documentation updated
   
   ## Related
   
   - Issue #28 GDPR Article 15 & 17 implementation (✅ Done)
   - Phase 8 GDPR Additional Rights (in progress)
   - Frontend GDPR interfaces (Phase 10-11)
   
   ## Notes
   
   - Marketing objections are **automatically approved** (absolute right)
   - No admin review needed for `ObjectionType::DirectMarketing`
   - Must be processed "without undue delay" per Article 21(3)
   - Consider implementing opt-in consent mechanism for future marketing features
   
   ## References
   
   - `docs/GDPR_ADDITIONAL_RIGHTS.md` - Complete implementation guide
   - `backend/src/domain/entities/gdpr_objection.rs` - Domain model
   - GDPR Article 21: https://gdpr-info.eu/art-21-gdpr/

.. raw:: html

   </div>


===================================================================================================
Issue #65: GDPR Articles 16 & 18: Implement rectification and restriction features (Phase 2 - K3s)
===================================================================================================

:State: **CLOSED**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-30
:Updated: 2025-11-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/65>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Complete implementation of GDPR Articles 16 (Right to Rectification) and 18 (Right to Restriction of Processing) for the KoproGo platform.
   
   ## Article 16 - Right to Rectification
   
   > The data subject shall have the right to obtain from the controller without undue delay the rectification of inaccurate personal data concerning him or her.
   
   Users must be able to request corrections to their personal data.
   
   ## Article 18 - Right to Restriction of Processing
   
   > The data subject shall have the right to obtain from the controller restriction of processing where one of the following applies...
   
   Users can request temporary "pause" of data processing (without deletion) for 4 specific legal grounds.
   
   ## Required Implementation
   
   ### 1. Database Schema
   Already designed in `docs/GDPR_ADDITIONAL_RIGHTS.md`:
   - `gdpr_rectification_requests` table
   - `gdpr_rectification_changes` table
   - `gdpr_restriction_requests` table
   
   ### 2. Domain Layer (✅ DONE - Phase 8.1)
   - `GdprRectificationRequest` entity (4 unit tests)
   - `GdprRestrictionRequest` entity (5 unit tests)
   - `FieldChange` tracking for corrections
   - `RestrictionReason` enum with 4 legal grounds
   
   ### 3. Backend Features (TODO)
   
   #### Article 16 API Endpoints
   ```
   POST   /api/v1/gdpr/rectification          - Submit correction request
   GET    /api/v1/gdpr/rectification          - List user's requests
   GET    /api/v1/gdpr/rectification/:id      - Get request details
   
   Admin endpoints:
   GET    /api/v1/admin/gdpr/rectification    - List pending requests
   PUT    /api/v1/admin/gdpr/rectification/:id/approve
   PUT    /api/v1/admin/gdpr/rectification/:id/reject
   POST   /api/v1/admin/gdpr/rectification/:id/apply
   ```
   
   #### Article 18 API Endpoints
   ```
   POST   /api/v1/gdpr/restriction            - Request restriction
   GET    /api/v1/gdpr/restriction            - List user's restrictions
   GET    /api/v1/gdpr/restriction/:id        - Get restriction details
   DELETE /api/v1/gdpr/restriction/:id        - Withdraw restriction
   
   Admin endpoints:
   GET    /api/v1/admin/gdpr/restriction      - List all restrictions
   PUT    /api/v1/admin/gdpr/restriction/:id/activate
   PUT    /api/v1/admin/gdpr/restriction/:id/lift
   PUT    /api/v1/admin/gdpr/restriction/:id/reject
   ```
   
   #### Use Cases - Rectification
   ```rust
   - create_rectification_request(user_id, changes, reason)
   - get_user_rectification_requests(user_id)
   - list_pending_rectifications(admin)
   - approve_rectification(request_id, admin_id)
   - reject_rectification(request_id, admin_id, reason)
   - apply_rectification(request_id) // Actually update the data
   ```
   
   #### Use Cases - Restriction
   ```rust
   - create_restriction_request(user_id, reason, justification, duration)
   - get_user_restrictions(user_id)
   - check_user_restriction_status(user_id) -> bool
   - activate_restriction(request_id, admin_id, duration)
   - lift_restriction(request_id, admin_id)
   - reject_restriction(request_id, admin_id, reason)
   - expire_restrictions() // Cron job
   ```
   
   ### 4. Email Notifications
   
   Rectification emails:
   ```rust
   - send_rectification_submitted(user_email, request_id)
   - send_rectification_approved(user_email, changes)
   - send_rectification_rejected(user_email, reason)
   - send_rectification_applied(user_email, changes)
   ```
   
   Restriction emails:
   ```rust
   - send_restriction_submitted(user_email, reason)
   - send_restriction_activated(user_email, duration)
   - send_restriction_lifted(user_email)
   - send_restriction_rejected(user_email, reason)
   ```
   
   ### 5. Middleware Integration (Article 18)
   
   Add restriction checking middleware:
   ```rust
   // Check if user has active restriction before processing operations
   pub async fn check_restriction_status(user_id: Uuid) -> Result<(), Error> {
       if has_active_restriction(user_id).await? {
           return Err(Error::ProcessingRestricted);
       }
       Ok(())
   }
   ```
   
   Exceptions:
   - With user consent
   - For legal claims
   - For protection of rights of another person
   
   ### 6. Audit Events
   
   New audit event types:
   ```rust
   - GdprRectificationRequested
   - GdprRectificationApproved
   - GdprRectificationRejected
   - GdprRectificationApplied
   
   - GdprRestrictionRequested
   - GdprRestrictionActivated
   - GdprRestrictionLifted
   - GdprRestrictionRejected
   - GdprRestrictionExpired
   ```
   
   ### 7. Frontend UI (Phase 10-11)
   
   Rectification page:
   - [ ] Form to request corrections (select entity, field, new value)
   - [ ] List of pending/approved/rejected requests
   - [ ] Status tracking with timeline
   - [ ] Admin approval interface
   
   Restriction page:
   - [ ] Form to request restriction (select reason, justification, duration)
   - [ ] List of active/lifted/expired restrictions
   - [ ] Withdraw restriction button
   - [ ] Admin activation/lifting interface
   
   ## Implementation Priority
   
   **Priority: Medium** (P2)
   
   **Reasoning:**
   - Legal requirement for GDPR compliance
   - User-facing features that build trust
   - Admin workflow needed for data quality
   - Relatively complex (approval workflows, middleware)
   
   ## Roadmap Placement
   
   **Phase 2 - K3s Infrastructure** (Mar - May 2026)
   - After core platform stabilization (Phase 1)
   - Alongside community features
   - Before mobile app (Phase 3)
   
   ## Acceptance Criteria
   
   ### Article 16 - Rectification
   - [ ] Database migrations created
   - [ ] Repository implementations with tests
   - [ ] Use cases with approval workflow
   - [ ] API endpoints (user + admin)
   - [ ] Email notifications
   - [ ] Audit logging
   - [ ] Rate limiting
   - [ ] Integration tests
   - [ ] E2E tests
   - [ ] Frontend forms and admin UI
   - [ ] Documentation
   
   ### Article 18 - Restriction
   - [ ] Database migrations created
   - [ ] Repository implementations with tests
   - [ ] Use cases with temporal logic
   - [ ] Middleware enforcement
   - [ ] API endpoints (user + admin)
   - [ ] Email notifications
   - [ ] Audit logging
   - [ ] Rate limiting
   - [ ] Cron job for expiration
   - [ ] Integration tests
   - [ ] E2E tests
   - [ ] Frontend forms and admin UI
   - [ ] Documentation
   
   ## Related
   
   - Issue #28 GDPR Article 15 & 17 (✅ Done)
   - Issue #64 GDPR Article 21 Marketing (Phase 2)
   - Phase 8 GDPR Additional Rights (in progress)
   
   ## Compliance Notes
   
   ### Article 16
   - Must be completed "without undue delay"
   - Controller must communicate corrections to recipients
   - User can request list of recipients
   
   ### Article 18
   - User must be informed before restriction is lifted (Article 18(3))
   - Only 4 legal grounds allowed (Article 18(1) a-d)
   - During restriction: storage allowed, other processing requires consent
   - Data should be marked as restricted in UI
   
   ## Technical Challenges
   
   1. **Field-level tracking**: Need to track old/new values for any field
   2. **Multi-entity corrections**: Users can correct User, Owner, Building data
   3. **Recipient notification**: Must track where data was shared (future)
   4. **Restriction enforcement**: Middleware must check every operation
   5. **Temporal logic**: Restrictions can expire, need background job
   6. **Admin workload**: Balance between user control and data integrity
   
   ## References
   
   - `docs/GDPR_ADDITIONAL_RIGHTS.md` - Complete implementation guide
   - `backend/src/domain/entities/gdpr_rectification.rs` - Domain model
   - `backend/src/domain/entities/gdpr_restriction.rs` - Domain model
   - GDPR Article 16: https://gdpr-info.eu/art-16-gdpr/
   - GDPR Article 18: https://gdpr-info.eu/art-18-gdpr/

.. raw:: html

   </div>


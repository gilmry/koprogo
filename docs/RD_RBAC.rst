====================================================
R&D: RBAC Architecture — Roles, Permissions & Tenancy
====================================================

Issues: #220, #221
Status: Partially Implemented
Phase: Jalon 1 (Security)

.. contents::
   :depth: 2

Current State
=============

KoproGo uses a **role-based access control** system implemented in multiple phases:

**Phase 1 (Implemented)**:

- Fixed roles: ``SUPERADMIN``, ``SYNDIC``, ``ACCOUNTANT``, ``OWNER``
- Multi-role assignments per user via ``user_role_assignments`` table
- Active role switching via ``POST /auth/switch-role``
- JWT carries ``role`` and ``organization_id`` claims
- Middleware ``AuthenticatedUser`` extracts and validates claims
- Role checks scattered across handler functions (hardcoded ``if role == "SYNDIC"``)

**Phase 2 (This R&D — Issues #220, #221)**:

- Fine-grained permissions (ABAC hybrid)
- Guest/tenant roles for community features (RESIDENT, GUEST, BOARD_MEMBER)
- Building-scoped roles with time-limited memberships
- Centralized permission middleware (avoiding scattered role checks)
- Casbin-based policy engine for scalability

Current Implementation Gaps
============================

1. **No resource-level permissions**: A syndic can access any building (should be scoped)
2. **No permission inheritance**: Board members use owner permissions (unclear baseline)
3. **No guest roles**: Contractors, notaries, and residents cannot authenticate
4. **Hard-coded role checks**: Every handler contains ``if role == "SYNDIC"`` patterns
5. **No audit trail**: Permission changes are not logged

Architecture Decision: Three Approaches Evaluated
=================================================

We evaluated three RBAC/ABAC approaches for Phase 2:

+------------------+---------------------------+--------+----------+---------------+
| Approach         | Complexity                | Scale  | GDPR Fit | Recommendation|
+==================+===========================+========+==========+===============+
| Fixed Roles (✅) | Low (current state)       | 100K   | Good     | Keep for MVP  |
+------------------+---------------------------+--------+----------+---------------+
| Casbin RBAC      | Medium (community-based)  | 1M     | Good     | Phase 2       |
+------------------+---------------------------+--------+----------+---------------+
| OpenFGA (Zanzibar)| High (enterprise-grade)   | 10M+   | Excellent| Phase 3+      |
+------------------+---------------------------+--------+----------+---------------+

**Recommendation**: Casbin for Phase 2 (proven Rust library, models as code, flexible)

Why Casbin?
-----------

- **Mature Rust library**: casbin-rs, 2K+ GitHub stars
- **Model as code**: RBAC/ABAC/ACL defined in config (perm.conf), not hardcoded
- **PostgreSQL storage**: Policies persisted, no Redis dependency
- **Fine-grained control**: Resource-level + action-level permissions
- **Community tested**: Used in production by KubeEdge, CloudWeaver, Gitea

Phase 2: Casbin Integration Design
===================================

Casbin uses a policy model (perm.conf) and policy store (CSV or PostgreSQL):

**Request format** (what we're checking):

.. code-block:: text

   r = sub, org, obj, act
   (subject=user_id, org=organization_id, object=building_id, action=edit_expense)

**Policy rules** (stored in PostgreSQL):

.. code-block:: text

   p = sub, org, obj, act
   p, syndic_role, org_123, building_456, edit_expense
   p, accountant_role, org_123, building_456, view_expense
   p, owner_role, org_123, unit_789, vote_meeting

**Role hierarchy** (user → role → policies):

.. code-block:: text

   g = _, _, _
   g, user_alice, syndic_role, org_123    # Alice is syndic in org 123
   g, user_bob, owner_role, org_123       # Bob is owner in org 123

**Evaluation engine**:

.. code-block:: text

   m = g(r.sub, p.sub, r.org) && r.org == p.org && r.obj == p.obj && r.act == p.act
   (Is subject assigned role? Org matches? Object matches? Action matches?)

Example: Synidic Building Scoping
==================================

**Current (no resource scoping)**:

.. code-block:: rust

   // Any syndic can edit any expense (dangerously broad)
   #[post("/expenses/{id}/approve")]
   pub async fn approve_expense(
       user: AuthenticatedUser,
       repo: web::Data<ExpenseRepository>,
   ) -> impl Responder {
       if user.role != "SYNDIC" {
           return error("Forbidden");
       }
       repo.approve(id).await  // No building check!
   }

**With Casbin (fine-grained)**:

.. code-block:: rust

   #[post("/expenses/{id}/approve")]
   pub async fn approve_expense(
       user: AuthenticatedUser,
       repo: web::Data<ExpenseRepository>,
       enforcer: web::Data<Enforcer>,
   ) -> impl Responder {
       // Get expense's building
       let expense = repo.get(id).await?;

       // Check: can user (alice) act (approve_expense) on (building_123) in (org_456)?
       if !enforcer.enforce(vec![
           user.id.to_string(),
           user.org_id.to_string(),
           expense.building_id.to_string(),
           "approve_expense".to_string(),
       ])? {
           return error("Forbidden");
       }
       repo.approve(id).await
   }

Community/Guest Roles (Issue #221 — Design)
============================================

New roles for community features (SEL, bookings, announcements, contractor reports):

**RESIDENT**: Lives in building, doesn't own unit (tenant)

- Can: participate in SEL, view shared documents, comment on announcements
- Cannot: vote in AG, approve expenses

**GUEST**: Temporary access (contractor, notary, inspector)

- Can: submit contractor reports (magic link), generate états datés, book common areas
- Cannot: view financial data, create expenses
- Duration: Time-limited (valid_until field)

**BOARD_MEMBER**: Council member with advisory role

- Can: All owner permissions + create/review decisions, lead polls
- Cannot: Unilaterally approve large expenses (requires board vote)
- Duration: Mandate-based (mandate_start, mandate_end)

**Database schema** (building-scoped):

.. code-block:: sql

   -- Building-scoped community role assignment
   CREATE TABLE community_memberships (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       user_id         UUID NOT NULL REFERENCES users(id),
       building_id     UUID NOT NULL REFERENCES buildings(id),
       community_role  VARCHAR(50) NOT NULL,  -- RESIDENT, GUEST, BOARD_MEMBER
       valid_until     TIMESTAMPTZ,           -- NULL = permanent
       granted_by      UUID REFERENCES users(id),
       grant_reason    VARCHAR(255),          -- Why assigned (for audit)
       created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       UNIQUE (user_id, building_id, community_role),
       CHECK (valid_until IS NULL OR valid_until > created_at)
   );

   -- Index for quick membership lookup
   CREATE INDEX idx_community_memberships_user_building
       ON community_memberships(user_id, building_id);
   CREATE INDEX idx_community_memberships_active
       ON community_memberships(building_id)
       WHERE valid_until IS NULL OR valid_until > NOW();

**Casbin policies for community roles**:

.. code-block:: text

   g, user_alice, resident_role, org_123            # Alice is resident in org 123
   g2, user_alice, community_member, building_456   # Alice is community member of building 456

   # RESIDENT: can participate in SEL
   p, resident_role, org_123, building_456, create_exchange
   p, resident_role, org_123, building_456, view_announcements

   # GUEST: contractor can submit reports
   p, guest_role, org_123, building_456, submit_contractor_report
   p, guest_role, org_123, ticket_789, write_notes

   # BOARD_MEMBER: can manage board decisions
   p, board_member_role, org_123, building_456, create_board_decision
   p, board_member_role, org_123, building_456, review_expenses

Implementation Roadmap
======================

**Week 1-2: Foundation**

- Add ``casbin`` and ``actix-casbin`` to Cargo.toml
- Create ``backend/src/infrastructure/auth/casbin_enforcer.rs``
- Design RBAC model file (``backend/rbac_model.conf``)
- Create ``casbin_policies`` PostgreSQL table (policy storage)

**Week 3-4: Migration**

- Add ``community_memberships`` table
- Create migration for Casbin policy storage
- Seed default policies (SUPERADMIN, SYNDIC, ACCOUNTANT, OWNER)
- Add middleware: ``CasbinAuthMiddleware``

**Week 5-6: Handler Updates**

- Replace hardcoded role checks with Casbin enforcer
- Update 50+ handlers in priority order (expenses, buildings, users)
- Add permission DTOs for error responses
- Add audit logging for permission denials

**Week 7-8: Testing & Community Roles**

- Write 100+ Casbin policy tests (matrix tests)
- Implement community role assignment endpoints
- Add RESIDENT/GUEST/BOARD_MEMBER role endpoints
- E2E tests for multi-building syndic scenarios

**Estimated**: 8 weeks (2 developer-months)

Database Changes Summary
========================

1. **casbin_policies** table (policy storage)
2. **community_memberships** table (guest/resident/board roles)
3. Add ``permission_cache_version`` to users table (cache invalidation)
4. Indexes: user_building, active_memberships, policy_lookup

GDPR Compliance
===============

**Data minimization**: Only store role + timestamp, not full permission lists
**Audit trail**: All permission changes logged to ``audit_log`` table
**Consent**: Community roles require explicit owner approval (grandfather's consent model)
**Right to erasure**: Anonymize community_memberships when user is deleted

Backward Compatibility
======================

The current fixed-role system will continue working during Phase 2:

- Casbin policies layer on top (additive, not replacing)
- JWT claims still carry ``role`` field (for backward compatibility)
- Handlers can support both old and new auth until fully migrated
- Gradual migration: test-driven, feature-flagged per handler

Risk Mitigation
===============

**Risk**: Over-permissioning during migration (granting too much access)

**Mitigation**:
- Use principle of least privilege: start DENY, explicitly ALLOW
- Audit every policy addition (require code review)
- Run monthly permission audits comparing current vs intended state

**Risk**: Performance regression (Casbin enforcement overhead)

**Mitigation**:
- Cache Casbin decisions in Redis (1-hour TTL)
- Benchmark enforcement cost (target < 1ms per decision)
- Use partial indexes on casbin_policies for common queries

**Risk**: Policy explosion (hundreds of rules = hard to maintain)

**Mitigation**:
- Use role hierarchy (Casbin g2, g3 for role composition)
- Document policy intent in comments (``# Policy: Syndic can edit building data``)
- Regular cleanup: remove unused roles/policies quarterly

Related Issues
==============

- **#220**: Fine-grained RBAC (this R&D)
- **#221**: Guest/community roles (building-scoped)
- **#85**: Contractor access for ticket system (GUEST role use case)
- **#88**: Board member roles for convocations
- **#78**: 2FA for strong authentication

References
==========

- `Casbin Rust library <https://github.com/casbin/casbin-rs>`_
- `Casbin RBAC model examples <https://casbin.org/docs/rbac/>`_
- `Article 577 CC (Belgian copropriété hierarchy) <https://www.justetmieux.be/>`_
- `GDPR Article 32 (access control) <https://gdpr-info.eu/art-32-gdpr/>`_

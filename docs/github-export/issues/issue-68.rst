=================================================================================
Issue #68: fix(tests): BDD tests fail with super_admin role constraint violation
=================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-30
:Updated: 2025-10-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/68>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 🐛 Problem
   
   3 BDD Cucumber tests are failing systematically in CI with database constraint violations:
   
   ### Failing Tests:
   1. **GDPR - User exports personal data** - Audit log assertion fails
   2. **GDPR - SuperAdmin exports another user's data** - `users_role_check` constraint violation
   3. **GDPR - SuperAdmin erases user data** - `users_role_check` constraint violation
   
   ### Error Message:
   ```
   Database(PgDatabaseError { 
     severity: Error, 
     code: "23514", 
     message: "new row for relation \"users\" violates check constraint \"users_role_check\"",
     constraint: Some("users_role_check")
   })
   ```
   
   ### Root Cause:
   BDD step definition uses `super_admin` (snake_case) when creating test superadmin users:
   ```rust
   // tests/bdd.rs:1184
   #[given("I am a SuperAdmin")]
   async fn i_am_a_superadmin(world: &mut World) {
       // Creates user with role: "super_admin" (snake_case)
   }
   ```
   
   But the database constraint `users_role_check` only accepts specific role values, likely `superadmin` (no underscore).
   
   ## 🔍 Investigation Needed:
   1. Check the exact values allowed by `users_role_check` constraint
   2. Verify the UserRole enum definition
   3. Determine if `super_admin` or `superadmin` is the correct value
   
   ## 📝 Files to Check:
   - `backend/tests/bdd.rs` (line ~1184) - BDD step definition
   - `backend/migrations/*_create_users_table.sql` - Check constraint definition
   - `backend/src/domain/entities/user.rs` - UserRole enum
   
   ## ✅ Solution Options:
   
   ### Option 1: Fix BDD step to use correct role
   ```rust
   role: UserRole::SuperAdmin.to_string() // Uses enum directly
   ```
   
   ### Option 2: Update database constraint
   If `super_admin` is the correct value, update the check constraint to accept it.
   
   ### Option 3: Skip failing tests temporarily
   Mark tests as `#[ignore]` until fixed:
   ```rust
   #[ignore = "Role constraint issue - see #68"]
   #[tokio::test]
   async fn test_superadmin_export() { ... }
   ```
   
   ## 🎯 Acceptance Criteria:
   - [ ] All 3 BDD tests pass
   - [ ] CI pipeline succeeds
   - [ ] Role naming is consistent across codebase
   
   ## 📊 Impact:
   - **Current**: CI fails on every push
   - **After fix**: CI green, GDPR tests fully covered
   
   ## 🔗 Related:
   - #66 - E2E test database cleanup
   - #67 - Phase 13-14: Documentation and QA review
   
   ---
   
   **Priority**: High (blocks CI)
   **Labels**: bug, tests, gdpr, bdd
   **Milestone**: Phase 1 - VPS MVP

.. raw:: html

   </div>


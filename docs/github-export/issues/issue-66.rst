=====================================================================
Issue #66: E2E: Admin login timeouts after user logout in GDPR tests
=====================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-30
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/66>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   3 out of 5 GDPR E2E tests are failing due to admin login timeouts that occur after a test user logs out. The tests have been temporarily skipped with `test.describe.skip` to unblock Phase 12 completion.
   
   ## Affected Tests
   
   Located in `frontend/tests/e2e/Gdpr.spec.ts`:
   
   1. **Mixed Scenario: User Creates Data, Admin Exports** (line 180)
      - User registers, logs in, creates activity, logs out
      - Admin tries to log in → **TIMEOUT** waiting for navigation
   
   2. **Audit Logs Verification** (line 251)
      - User exports data, logs out
      - Admin logs in and checks audit logs → **ERR_ABORTED on /login**
   
   3. **Cross-Organization Access** (line 305)
      - Create 2 test users
      - Admin logs in to verify cross-org access → **TIMEOUT** on navigation
   
   ## Root Cause (Hypothesis)
   
   Browser state (localStorage/cookies) is not being properly cleared between user logout and admin login within the same test. This might cause:
   - Stale auth tokens interfering with new login
   - Race condition in `authStore.init()` 
   - Session cookie conflicts
   
   ## Working Tests (for comparison)
   
   These 2 tests pass successfully:
   
   1. **Complete User Journey** ✅
      - User registers, logs in via UI, exports, erases account
      - No admin login involved
   
   2. **Admin Operations** ✅
      - User registers via API (never logs in via UI)
      - Admin logs in fresh without prior user session
      - Successfully exports and erases user data
   
   ## Error Examples
   
   ### Timeout Error
   ```
   Error: page.waitForURL: Timeout 10000ms exceeded.
   =========================== logs ===========================
   waiting for navigation to "/admin" to be finished
   ============================================================
   ```
   
   ### ERR_ABORTED
   ```
   GET http://localhost:3000/login net::ERR_ABORTED
   ```
   
   ## Investigation Steps
   
   1. Add explicit `page.context().clearCookies()` after user logout
   2. Add `localStorage.clear()` via `page.evaluate()` 
   3. Check if `authStore.logout()` properly clears all state
   4. Consider using separate browser contexts for user vs admin sessions
   5. Add debug logging to track token state transitions
   
   ## Test Status
   
   - ✅ 2/5 passing (40%)
   - ⏭️ 3/5 skipped (timeout issues)
   
   ## Related Files
   
   - `frontend/tests/e2e/Gdpr.spec.ts` (test file)
   - `frontend/src/stores/auth.ts` (authStore implementation)
   - `frontend/src/components/admin/AdminGdprPanel.svelte`
   - `frontend/src/components/GdprDataPanel.svelte`
   
   ## Priority
   
   Medium - Tests are temporarily skipped, but should be fixed before production deployment.

.. raw:: html

   </div>


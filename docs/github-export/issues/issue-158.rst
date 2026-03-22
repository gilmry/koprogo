====================================================================
Issue #158: E2E tests have 200+ compilation errors after API changes
====================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,priority:high release:v0.5.0,testing e2e
:Assignees: Unassigned
:Created: 2025-12-06
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/158>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problem
   
   After recent API changes, **19 E2E test files have 200+ compilation errors** that prevent CI from passing.
   
   ## Root Cause
   
   The E2E tests were written before recent API changes and use outdated:
   - Function signatures (wrong number of parameters)
   - DTO structures (missing fields like `total_tantiemes`, `organization_id`, `quota`)
   - Type mismatches (String vs Uuid, Option<T> vs T)
   - Obsolete imports and method names
   
   ## Affected Files (19 files)
   
   1. `tests/e2e_unit_owner.rs` - **29 errors**
   2. `tests/e2e_auth.rs` - **4 errors**
   3. `tests/e2e_board_dashboard.rs` - **4 errors**
   4. `tests/e2e_documents.rs` - **8 errors**
   5. `tests/e2e_convocations.rs` - **45 errors**
   6. `tests/e2e_resolutions.rs` - **83 errors** (worst offender)
   7. `tests/e2e_tickets.rs` - **39 errors**
   8. `tests/e2e_payments.rs` - **37 errors**
   9. `tests/e2e_notifications.rs` - **34 errors**
   10. `tests/e2e_quotes.rs` - **44 errors**
   11. `tests/e2e_budget.rs` - multiple errors
   12. `tests/e2e_local_exchange.rs` - **48 errors** (partially fixed)
   13. Others with fewer errors
   
   ## Common Error Patterns
   
   ### 1. `FinancialReportUseCases::new` signature changed
   **Error**: Takes 3 args but 2 provided
   **Fix**: Add `journal_entry_repo` parameter
   
   ### 2. `CreateBuildingDto` missing fields
   **Error**: Missing `total_tantiemes`, `organization_id`
   **Fix**: Add fields to DTO initialization
   
   ### 3. `LoginResponse` field renamed
   **Error**: No field `access_token`, should be `token`
   **Fix**: Update field access
   
   ### 4. `CreateUnitDto` missing fields
   **Error**: Missing `organization_id`, `quota`
   **Fix**: Add required fields
   
   ### 5. Type mismatches
   **Error**: Expected `Uuid`, found `String` (and vice versa)
   **Fix**: Use `.to_string()` or `Uuid::parse_str()`
   
   ## Impact
   
   - ✅ **Main codebase (lib)**: Compiles fine
   - ✅ **BDD tests**: Pass
   - ❌ **E2E tests**: 200+ compilation errors
   - ❌ **CI pipeline**: Fails on `cargo clippy --all-targets`
   
   ## Temporary Workaround Applied
   
   - Updated `Makefile` to use `--all-targets` (aligns local dev with CI)
   - Fixed unused imports in 5 files (partial cleanup)
   - **CI still fails** because of remaining compilation errors
   
   ## Recommended Fix Strategy
   
   ### Option A: Quick Fix (Recommended)
   Temporarily exclude E2E tests from CI while fixing them:
   
   ```yaml
   # .github/workflows/ci.yml
   - name: Run Clippy
     run: |
       cd backend
       cargo clippy --lib --all-features -- -D warnings  # Only lib, not tests
   ```
   
   Then fix tests one by one in separate PRs.
   
   ### Option B: Mass Fix (Time-consuming)
   Create a shared `tests/e2e_common.rs` with `setup_test_db()` and make all tests use it.
   
   ### Option C: Rewrite Tests (Nuclear option)
   Delete obsolete tests and rewrite using current API patterns from `e2e.rs` (which works).
   
   ## Action Items
   
   - [ ] Choose fix strategy
   - [ ] Update CI to exclude failing E2E tests temporarily
   - [ ] Create sub-issues for each test file
   - [ ] Fix tests one by one
   - [ ] Re-enable E2E tests in CI once fixed
   
   ## Related Commits
   
   - `5eb6266` - Partial fix (unused imports)
   - `838a88a` - HOTFIX (migration restore)
   - Makefile updated to use `--all-targets`
   
   ## Priority
   
   **HIGH** - Blocks CI, prevents detection of real issues

.. raw:: html

   </div>


==============================================================================
Issue #69: Add Playwright E2E tests for unit management and document features
==============================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-31
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/69>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Add comprehensive Playwright E2E tests for the new features developed:
   
   ### Features to test
   
   #### 1. Unit Management
   - [ ] **Unit Creation Modal** (`UnitCreateModal.svelte`)
     - Test opening modal from building detail page
     - Test form validation (number, type, floor, surface, quota)
     - Test successful unit creation
     - Test error handling
     - **Required testids**: `unit-create-modal`, `unit-create-form`, `unit-number-input`, `unit-type-select`, `unit-floor-input`, `unit-surface-input`, `unit-quota-input`, `unit-create-submit`
   
   - [ ] **Unit Edit Modal** (`UnitEditModal.svelte`)
     - Test opening edit modal from unit list
     - Test editing all fields
     - Test quota validation
     - Test successful update
     - **Required testids**: `unit-edit-modal`, `unit-edit-form`, `unit-edit-submit`, `unit-edit-button-{id}`
   
   #### 2. Multi-Owner Management
   - [ ] **Add Owner to Unit** (`UnitOwnerAddModal.svelte`)
     - Test opening add owner modal
     - Test owner search functionality
     - Test ownership percentage validation (0.01-100%)
     - Test primary contact checkbox
     - Test successful owner assignment
     - **Required testids**: `owner-add-modal`, `owner-search-input`, `owner-select`, `ownership-percentage-input`, `primary-contact-checkbox`, `owner-add-submit`
   
   - [ ] **Edit Unit Owner** (`UnitOwnerEditModal.svelte`)
     - Test opening edit modal from unit owners list
     - Test modifying ownership percentage
     - Test toggling primary contact
     - Test validation that total = 100%
     - **Required testids**: `owner-edit-modal`, `owner-edit-percentage`, `owner-edit-primary`, `owner-edit-submit`, `owner-edit-button-{id}`
   
   - [ ] **Unit Owners Display** (`UnitOwners.svelte`)
     - Test displaying list of owners with percentages
     - Test visual validation (green for 100%, red for ≠100%)
     - Test responsive layout (320px width)
     - **Required testids**: `unit-owners-list`, `unit-owner-card-{id}`, `ownership-total`, `ownership-warning`
   
   #### 3. Document Management
   - [ ] **Expense Detail Page** (`expense-detail.astro`, `ExpenseDetail.svelte`)
     - Test navigation from expense list
     - Test displaying expense details
     - Test document upload for expense
     - Test document list display
     - **Required testids**: `expense-detail`, `expense-info`, `expense-documents`, `expense-upload-button`, `expense-document-list`
   
   - [ ] **Meeting Detail Page** (`meeting-detail.astro`, `MeetingDetail.svelte`)
     - Test navigation from meeting list
     - Test displaying meeting details
     - Test document upload for meeting
     - Test document list display
     - **Required testids**: `meeting-detail`, `meeting-info`, `meeting-documents`, `meeting-upload-button`, `meeting-document-list`
   
   - [ ] **Document Lists**
     - Test clickable expense cards linking to detail page
     - Test clickable meeting cards linking to detail page
     - Test document count badges
     - **Required testids**: `expense-card-{id}`, `meeting-card-{id}`, `document-count-{id}`
   
   #### 4. Navigation & Footer
   - [ ] **Responsive Navigation**
     - Test navigation on mobile (320px width)
     - Test role switcher in user dropdown menu
     - Test no horizontal scroll on all screen sizes
     - **Required testids**: Already exist (`navigation`, `role-selector`, `user-menu-dropdown`)
   
   - [ ] **Footer**
     - Test language selector in footer
     - Test sync status indicator
     - Test footer responsive layout
     - **Required testids**: `footer`, `footer-language-selector`, `footer-sync-status`, `footer-copyright`
   
   ### Implementation Notes
   1. Add all required `data-testid` attributes to components
   2. Follow existing test patterns in `tests/e2e/` directory
   3. Use testcontainers for database isolation
   4. Test both happy paths and error scenarios
   5. Ensure tests work on mobile viewports (320px, 375px, 768px)
   
   ### Acceptance Criteria
   - [ ] All components have appropriate testid attributes
   - [ ] Tests cover all user journeys for unit management
   - [ ] Tests cover multi-owner workflows (add, edit, validation)
   - [ ] Tests cover document management for expenses and meetings
   - [ ] Tests verify responsive behavior on mobile
   - [ ] All tests pass consistently
   - [ ] Code coverage maintained or improved
   
   ### Related Components
   - `frontend/src/components/UnitCreateModal.svelte`
   - `frontend/src/components/UnitEditModal.svelte`
   - `frontend/src/components/UnitOwnerAddModal.svelte`
   - `frontend/src/components/UnitOwnerEditModal.svelte`
   - `frontend/src/components/UnitOwners.svelte`
   - `frontend/src/components/ExpenseDetail.svelte`
   - `frontend/src/components/ExpenseDocuments.svelte`
   - `frontend/src/components/MeetingDetail.svelte`
   - `frontend/src/components/MeetingDocuments.svelte`
   - `frontend/src/pages/expense-detail.astro`
   - `frontend/src/pages/meeting-detail.astro`
   - `frontend/src/layouts/Layout.astro`

.. raw:: html

   </div>


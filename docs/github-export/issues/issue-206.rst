=========================================================================================
Issue #206: feat(frontend): Complete UI action wiring for Community & Contractor features
=========================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:vps track:software,priority:high release:v0.5.0
:Assignees: Unassigned
:Created: 2026-02-19
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/206>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Backend APIs are 100% complete for Community (bookings, sharing) and Contractor (work reports, inspections) features. However, several frontend action flows have placeholder `alert()` calls instead of real API integration.
   
   ## Confirmed Gaps (found by code audit 2026-02-19)
   
   ### 🔴 Alert placeholders (broken UX)
   
   | File | Line | Issue |
   |------|------|-------|
   | `frontend/src/pages/booking-detail.astro` | 105 | `onclick="alert('Booking functionality coming soon!')"` |
   | `frontend/src/pages/sharing-detail.astro` | 92 | `onclick="alert('Loan request feature coming soon!')"` |
   
   ### 🟡 Missing create/detail components
   
   | Feature | Missing Components | API Module |
   |---------|-------------------|------------|
   | Resource Bookings | `BookingCreateModal.svelte` | `bookings.ts` ✅ |
   | Object Sharing | `SharedObjectCreateModal.svelte`, `LoanRequestModal.svelte` | `sharing.ts` ✅ |
   | Work Reports | `WorkReportCreateForm.svelte`, `WorkReportDetail.svelte` | `work-reports.ts` ✅ |
   | Technical Inspections | `InspectionCreateForm.svelte`, `InspectionDetail.svelte` | `inspections.ts` ✅ |
   
   ### 🟡 Shell pages without content
   
   | Page | Current state |
   |------|--------------|
   | `/settings` | Empty shell — no UserSettings component |
   | `/owner/contact` | Redirects to `/profile` instead of showing syndic contact |
   
   ## Acceptance Criteria
   
   ### Sprint 1 — Fix alert() placeholders
   - [ ] `BookingCreateModal.svelte` — date/time picker, duration, calls `bookingsApi.createBooking()`
   - [ ] `BookingDetail.svelte` (Svelte component replacing inline HTML) — show booking history, cancel action
   - [ ] `SharedObjectCreateModal.svelte` — add new object to share, calls `sharingApi.createObject()`
   - [ ] `LoanRequestModal.svelte` — request to borrow, calls `sharingApi.requestLoan()`
   - [ ] Loan return action in `SharedObjectCard.svelte` — calls `sharingApi.returnObject()`
   
   ### Sprint 2 — Work Reports & Inspections CRUD
   - [ ] `WorkReportCreateForm.svelte` — create report, calls `workReportsApi.create()`
   - [ ] `WorkReportDetail.svelte` — show detail + close/add-note actions
   - [ ] Add "+ New Report" button in `WorkReportList.svelte`
   - [ ] `InspectionCreateForm.svelte` — schedule inspection, calls `inspectionsApi.create()`
   - [ ] `InspectionDetail.svelte` — show detail + schedule/complete/fail actions
   - [ ] Add "+ Schedule" button in `InspectionList.svelte`
   
   ### Sprint 3 — Settings & Owner pages
   - [ ] `UserSettings.svelte` — change password, language preference, timezone
   - [ ] `SyndicContactPanel.svelte` — calls `GET /public/buildings/:slug/syndic`, mounted at `/owner/contact`
   
   ## Technical Notes
   
   - All API modules exist and are complete — this is **frontend-only work**
   - Follow existing patterns: `onMount` → `apiFetch`, `toast.error()` on failure, loading state
   - Use existing UI components: `Modal.svelte`, `FormInput.svelte`, `Button.svelte`
   - No backend changes required
   
   ## Related Issues
   
   - #99 Community modules (notices, skills, sharing, bookings) — backend complete, this closes the frontend gap
   - #52/#134 Contractor backoffice — backend complete, this closes the frontend gap

.. raw:: html

   </div>


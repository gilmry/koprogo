=============================================================================================
Issue #404: fix(type-safety): align 33 Svelte components with backend enum contracts (P7-702)
=============================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: audit-2026-04,svelte5-runes type-safety
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/404>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 7 — STORY-P7-702 follow-up to #403.
   
   Once the Sprint 7 foundation (`feat(type-safety): extend utoipa schema coverage`) committed the Rust enum source of truth and re-export scaffolding, `svelte-check --threshold error` surfaced **102 real contract drifts** in 33 Svelte components that had been masked by hand-written enums.
   
   This PR fixes all of them. Final state: **0 production svelte-check errors**, **64/64 vitest tests still green**.
   
   ## What shipped
   
   Commit `e8aba88`. Three independent batches executed in parallel.
   
   **Batch A — Enum alignment (9 components):**
   - `ObjectCategoryBadge`: dropped non-existent variants, now uses the 8 backend categories
   - `ObjectConditionBadge`: dropped New/LikeNew/Poor, use 4 backend variants (Excellent, Good, Fair, Used)
   - `NotificationPreferences`: removed 13 frontend-only NotificationType variants, kept the 10 backend ones
   - `BudgetDetail`: lowercase \"approved\"/\"draft\" → PascalCase matching backend (x4)
   - `CampaignStatusBadge/Detail/List`: added missing AwaitingAGVote + Cancelled
   - `InspectionList`: Record index cast for string key lookup
   - `WorkReportDetail`: discriminated narrowing for WarrantyType.custom object
   
   **Batch B — Svelte 5 runes interop (10 components):**
   - `BuildingDetail/List`, `OrganizationList`, `UserListAdmin`: Modal `bind:isOpen` → `isOpen` + `onclose` callback (6 modals total)
   - `OwnerList`, `SyndicDashboard`: `on:close/on:save` → `onclose/onsave` props
   - `NotificationDropdown/Bell/Item`, `NotificationList`: drop `createEventDispatcher`, use runes callbacks
   - `RouteGuard`, `NotificationBell`: wrap async init in `\$effect` with cancel flag
   - `Button`: added \"ghost\" variant (was used by AgVideoSession)
   
   **Batch C — Data-shape drifts + i18n typing (14 components):**
   - `BoardMemberList`, `DecisionTracker`: fallback fields absent from backend responses
   - `LegalHelper`: use exported `API_BASE_URL` instead of `api.baseUrl`
   - `ExpenseList` + `types.ts`: add `approval_status` field to Expense
   - `FormInput`: cast autocomplete to `AutoFill`; widen value to `number|null`
   - `BuildingForm`: normalize `construction_year` nullability
   - `SeedManager`: narrow unknown caught error
   - `NoticeDetail`: proper `\$state<Notice | null>(null)` generic typing
   - `PollDetail`, `QuoteDetail`: non-null narrowing
   - `AchievementList`, `ChallengeList`, `ExchangeDetail`: wrap i18n substitution vars in `{ values: { ... } }` per svelte-i18n `MessageObject` schema
   
   ## Verification
   
   ```bash
   docker compose exec frontend sh -c \"cd /app && npx svelte-check --threshold error\"
   # → 0 production errors (was 102)
   
   docker compose exec frontend sh -c \"cd /app && npx vitest run\"
   # → 64/64 tests passing
   ```
   
   The Rust enum is now the single source of truth for every enum variant visible in the UI; TypeScript refuses any divergence.
   
   Depends on #403.

.. raw:: html

   </div>


=====================================================================================
Issue #403: feat(type-safety): extend utoipa schema coverage to 41 enums (P7-701/704)
=====================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: audit-2026-04,type-safety
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/403>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 7 — STORY-P7-701 and STORY-P7-704 from the [Maury plan](../blob/feature/dev/docs/cowork/Plan-Maury-2026-04-13.md).
   
   After the 7-iteration UI audit (v1→v7, April 2026) exposed structural desalignment between the Rust backend and the Svelte frontend on enum values, case sensitivity, and field names, Sprint 7 closes the type-safety loop by:
   
   1. Making the Rust DTO/entity enums the single source of truth
   2. Exporting the OpenAPI spec via utoipa
   3. Re-exporting all frontend enums from the generated `api.d.ts`
   4. TypeScript refuses any divergence at compile time
   
   ## What shipped
   
   **Backend** (commit `18d5756`):
   - `#[derive(utoipa::ToSchema)]` added to 36 domain enums across 17 entities:
     - bookings (ResourceType, BookingStatus, RecurringPattern)
     - budgets (BudgetStatus)
     - convocations (ConvocationType, ConvocationStatus, AttendanceStatus)
     - energy campaigns (CampaignType, CampaignStatus, EnergyType, ContractType)
     - etats dates (EtatDateStatus, EtatDateLanguage)
     - achievements (AchievementCategory, AchievementTier)
     - challenges (ChallengeType, ChallengeStatus)
     - technical inspections (InspectionType, InspectionStatus)
     - local exchanges (ExchangeType, ExchangeStatus, CreditStatus, ParticipationLevel)
     - notices (NoticeType, NoticeCategory, NoticeStatus)
     - payment reminders (ReminderLevel, ReminderStatus, DeliveryMethod)
     - quotes (QuoteStatus)
     - skills (SkillCategory, ExpertiseLevel)
     - work reports (WorkType, WarrantyType)
     - shared objects (SharedObjectCategory, ObjectCondition)
   - All 41 enum schemas registered in `openapi.rs` `components(schemas)`
   - Fixed pre-existing compile error (`&*self.pool` on non-Deref `Pool<Postgres>`)
   
   **Frontend** (same commit):
   - `api.d.ts` regenerated from live OpenAPI spec (docs/api/openapi.json)
   - 18 `lib/api/*.ts` wrappers migrated to re-export enums from `api.d.ts`:
   
     ```ts
     export type Xxx = components[\"schemas\"][\"Xxx\"];
     export const Xxx = { Foo: \"Foo\" as const, ... } satisfies Record<string, Xxx>;
     ```
   
   ## Result
   
   - Hand-written enum count: **45 → 7** (those 7 are intentional UI-only states without backend counterpart)
   - 64/64 vitest tests passing in Docker compose dev
   - This commit surfaced 102 svelte-check errors which were real enum mismatches (see follow-up #X for component alignment)
   
   Closes the 'enum mismatch' class of bugs that drove audit v1→v7.

.. raw:: html

   </div>


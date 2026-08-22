=======================================================================================
Issue #405: ci(anti-drift): end-to-end Rust→OpenAPI→TypeScript→Svelte contract (P7-705)
=======================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: audit-2026-04,type-safety
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/405>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 7 — STORY-P7-705. Commit `0ee2124`.
   
   After Sprint 7 made the Rust utoipa-annotated DTOs the single source of truth (#403, #404), CI needed to enforce this contract so no drift can sneak past future PRs.
   
   ## Problem with previous CI
   
   The old `contract-types` job only verified that `src/types/api.d.ts` matched a static `openapi.yaml` committed to the repo. It did NOT catch:
   - A new `#[derive(ToSchema)]` or `#[utoipa::path]` added to Rust without re-running the spec export
   - A DTO/enum variant removed from Rust but still referenced in frontend wrappers
   - A handwritten enum reintroduction in `lib/api/*.ts`
   
   ## New 4-gate pipeline (contract-types job)
   
   1. **Rust → JSON**: build backend, run `export_openapi` binary → fresh `/tmp/openapi.fresh.json`
   2. **JSON committed?**: diff `docs/api/openapi.json` vs fresh — fails if ToSchema/utoipa::path changed without `make openapi-export`
   3. **JSON → TypeScript**: `npm run types:generate` and `git diff --exit-code` on api.d.ts
   4. **TypeScript → Svelte**: `npx svelte-check --threshold error` — fails on any enum mismatch
   5. **Guard**: cap hand-written enums in `lib/api/*.ts` at 4 (currently 2)
   
   ## New vitest job
   
   Runs the 64 Svelte component unit tests on every push.
   
   ## oasdiff migration
   
   Updated to prefer `openapi.json` (live export) and fall back to `openapi.yaml` during migration.
   
   ## Result
   
   After this commit, any attempt to rename a Rust enum variant without rebuilding the pipeline — or any attempt to reintroduce a hand-written enum — breaks CI at the earliest possible stage.

.. raw:: html

   </div>


================================================================================================
Issue #585: [Story 5.1] Table acp_enabled_modules + ModuleGuard middleware + ModuleDisabledError
================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/585>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.1 — Table `acp_enabled_modules` + `ModuleGuard` middleware + `ModuleDisabledError`
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.1-module-registry` · Refs: #556
   
   ## Goal
   
   Table + entité + use-cases enable/disable + middleware Actix `ModuleGuard` + extension `AppError::ModuleDisabled`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR39 ; INV-25, ADR-0015
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : **ADR-0015** (Modularité par ACP — module registry)
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Admin enable module=community ACP X → menu Communauté visible côté syndic ACP X ; route `/community/*` répond 200
   - **@edge** : Module activé puis désactivé puis réactivé → `archived_at` cycling → données intactes (cf. INV-27)
   - **@security** : Syndic ACP avec Compta désactivée tente `/expenses` → 403 `ModuleDisabled{module:accounting}`
   - **@negative** : Module name invalide ("foobar") → 422 ; module=identity tentative disable → 403 (toujours actif)
   
   ## data-testid
   
   `module-enable-submit`, `module-disable-submit`, `module-list-row-{{name}}`
   
   ## Files
   
   - `backend/migrations/20260620_010000_create_acp_enabled_modules.sql` + DOWN
   - `backend/src/domain/entities/acp_enabled_module.rs`
   - `backend/src/application/ports/module_registry.rs`
   - `backend/src/application/use_cases/module_registry_use_cases.rs`
   - `backend/src/infrastructure/web/middleware/module_guard.rs`
   - `backend/src/domain/errors/app_error.rs` (extension ModuleDisabled)
   - `backend/tests/features/module_registry.feature`
   
   ## Definition of Done
   
   - [ ] Table + entité AcpEnabledModule
   - [ ] Port ModuleRegistry + use-cases enable/disable + is_enabled
   - [ ] Middleware ModuleGuard sur routes non-identity
   - [ ] AppError::ModuleDisabled
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.1
   - Architecture ADR-0015 : [`docs/maury/refonte-ux-multi-role-acp/architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) §4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


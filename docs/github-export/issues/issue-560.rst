===============================================================================================
Issue #560: [Story 1.3] Filtrage role-based list_buildings + list_acps (scope_guard middleware)
===============================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,slice-1
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/560>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 1.3 — Filtrage role-based `list_buildings` + `list_acps` (scope_guard middleware)
   
   > Maury Phase 6 Exécution · Slice 1 · Story `story/1.3-list-buildings-role-based` · Refs: #556
   
   ## Goal
   
   Adapter les use-cases liste pour filtrer par scope (admin tout, syndic cabinet, owner ses ACPs, contractor via MagicLink). Introduit `ListScope` enum + middleware `scope_guard`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR4 ; INV-3, INV-7
   - **Effort** : M
   - **Deps** : Story 1.1, Story 1.2
   - **ADR refs** : ADR-0010
   - **Cluster coord** : si use-case touche Decimal/Result legacy → migrer dans la même PR (audit pré-PR)
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Admin GET `/buildings` → tout ; Syndic GET → ACPs de son organization seulement ; Owner GET → ACPs où user a `UserRoleAssignment owner`
   - **@edge** : User multi-rôle (admin ET syndic A) → admin domine (voit tout)
   - **@security** : Syndic cabinet B forge query param `acp_id=cabinet_A` → 403 `AcpNotInScope`
   - **@negative** : User non-auth tente GET `/buildings` → 401 ; query sans scope_id explicite (rôle non-admin) → 400
   
   ## data-testid
   
   — (filtre transparent côté backend, pas de nouvel UI)
   
   ## Files
   
   - `backend/src/application/use_cases/list_buildings_use_case.rs` (refacto)
   - `backend/src/application/use_cases/list_acps_use_case.rs` (NEW)
   - `backend/src/infrastructure/web/middleware/scope_guard.rs` (NEW)
   - `backend/tests/features/list_buildings_role_based.feature`
   
   ## Definition of Done
   
   - [ ] `ListScope` enum (`All` / `OrganizationScope(org_id)` / `AcpScope(acp_id)` / `UserScope(user_id)`)
   - [ ] `list_buildings_use_case` refactoré avec filtrage
   - [ ] `list_acps_use_case` créé avec filtrage
   - [ ] Middleware `scope_guard` Actix injecte `AcpScope` selon role+user
   - [ ] BDD `list_buildings_role_based.feature` 4-cat VERT
   - [ ] Pas de nouveau `Result<_, String>` (audit grep pré-PR)
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §3 Story 1.3
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


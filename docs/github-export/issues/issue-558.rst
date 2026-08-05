=======================================================
Issue #558: [Story 1.1] Entité ACP backend + CRUD /acps
=======================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,priority:high rust,legal-compliance governance,maury track-h-conformite,slice-1
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/558>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 1.1 — Entité ACP backend + CRUD `/acps`
   
   > Maury Phase 6 Exécution · Slice 1 · Story `story/1.1-acp-domain-entity` · Refs: #556
   
   ## Goal
   
   Créer l'entité `Acp` (domain) + port `AcpRepository` + use-cases CRUD + adapter PostgreSQL + handlers Actix + migration SQL `create_acps`. Premier maillon de la refacto domaine `Organization(0..1) → ACP(1..N) → Building(1..N)` (Art. 3.84 CC).
   
   ## Contexte Maury
   
   - **FR/INV** : FR1, FR3 ; INV-1, INV-2
   - **Effort** : L
   - **Deps** : Story 0.1 (caractérisation verte)
   - **ADR refs** : **ADR-0010** (ACP comme racine d'agrégat distincte d'Organization)
   - **Cluster coord** : NEW use-case → AppError natif, pas de dette #555 ; pas de #433 (ACP n'a pas de champ monétaire Decimal)
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Admin POST `/acps {name, address, organization_id?}` → 201 + Acp persistée + audit ; GET `/acps` filtré rôle → admin voit toutes, syndic voit celles de son cabinet
   - **@edge** : ACP avec `organization_id = null` (ACP auto-gérée) → autorisée ; ACP avec 0 building lié → autorisée
   - **@security** : Syndic cabinet B tente accès ACP cabinet A → 403 `AcpNotInScope` ; user non-admin tente POST `/acps` → 403
   - **@negative** : POST avec `organization_id` inexistante → 422 ; PUT sur ACP inexistante → 404 typé
   
   ## data-testid
   
   `acp-create-submit`, `acp-list-row-{{id}}`, `acp-edit-submit`
   
   ## Files
   
   - `backend/src/domain/entities/acp.rs`
   - `backend/src/application/ports/acp_repository.rs`
   - `backend/src/application/use_cases/acp_use_cases.rs`
   - `backend/src/application/dto/acp_dto.rs`
   - `backend/src/infrastructure/database/repositories/acp_repository_impl.rs`
   - `backend/src/infrastructure/web/handlers/acp_handlers.rs`
   - `backend/migrations/20260601_010000_create_acps.sql` + DOWN
   - `backend/tests/integration/acp_test.rs`
   - `backend/tests/features/acp.feature` (BDD 4-cat)
   
   ## Definition of Done
   
   - [ ] Entité `Acp` domaine avec invariants validés en `::new()`
   - [ ] Port `AcpRepository` trait async
   - [ ] Use-cases CRUD (create/list/get/update/delete) avec filtrage rôle (admin/syndic)
   - [ ] Adapter PostgreSQL (sqlx)
   - [ ] Handlers Actix `/acps` (POST/GET/PUT/DELETE)
   - [ ] Migration SQL `create_acps` + DOWN (rollback testable)
   - [ ] DTO `Acp` sérialise `organization_id` nullable correctement
   - [ ] Tests integration testcontainers VERTS
   - [ ] BDD `acp.feature` 4-cat VERT (@happy + @edge + @security + @negative)
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §3 Story 1.1
   - Architecture ADR-0010 : [`docs/maury/refonte-ux-multi-role-acp/architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) §4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


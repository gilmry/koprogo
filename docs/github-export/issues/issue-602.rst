===================================================================================================
Issue #602: Slice 1 gap: PostgresBuildingRepository + Building domain entity not migrated to acp_id
===================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug
:Assignees: Unassigned
:Created: 2026-05-25
:Updated: 2026-05-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/602>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Après merge de slice 1 (stories 1.1 + 1.2 + 1.3 + 1.4) sur `feature/dev`, **deux gaps** subsistent qui cassent `/buildings` en runtime :
   
   ### Gap 1 — Repository SQL non migré
   
   `backend/src/infrastructure/database/repositories/building_repository_impl.rs` réfère encore `organization_id` dans **20+ emplacements** (INSERT, SELECT, UPDATE, WHERE, ORDER BY, JOIN) alors que la migration `20260601040000_buildings_acp_id_not_null.sql` a `DROP COLUMN organization_id`.
   
   Conséquence runtime : toute query `/api/v1/buildings*` → erreur Postgres `column "organization_id" does not exist`.
   
   ### Gap 2 — Domain entity non migrée
   
   `backend/src/domain/entities/building.rs` :
   - Ligne 41 : `pub organization_id: Uuid,` (champ pas renommé en `acp_id`)
   - Pas de champ `acp_id` du tout
   
   Le `From<Row>` du repository ne peut donc pas mapper la nouvelle colonne `acp_id` même si on changeait les SELECT.
   
   ## Cause
   
   Stories 1.3 et 1.4 ont touché les *use_cases* + *middleware* + *DTO* mais pas l'infra repository SQL ni l'entité domain. Slice 1 a mergé avec un état runtime cassé pour `/buildings`. cargo check / cargo test --lib ne détectent pas (sqlx offline + mocks).
   
   ## Recette proposée
   
   **Story 1.5 (hotfix slice 1)** ou patch direct sur `feature/dev` :
   
   1. `backend/src/domain/entities/building.rs` : renommer `organization_id` → `acp_id` partout (entité, constructeur, tests in-module)
   2. `backend/src/infrastructure/database/repositories/building_repository_impl.rs` : remplacer toutes occurrences `organization_id` → `acp_id` dans SQL strings + `row.get(...)` + `.bind(...)` + filters
   3. `backend/src/application/use_cases/list_buildings_use_case.rs` : aligner `filters.organization_id` → `filters.acp_id` si pas déjà fait
   4. Regen `.sqlx/` offline cache (`cargo sqlx prepare` via docker)
   5. Tests integration testcontainer ajoutent un cas `/buildings` → 200 réel (pas mock) pour gater ce type de régression future
   
   ## Critères de validation
   
   - [ ] `docker compose run --rm backend bash -c "cargo sqlx prepare --workspace"` succès
   - [ ] `cargo test --tests` GREEN (incl. testcontainer/integration)
   - [ ] curl `http://localhost/api/v1/buildings` (admin token) → 200 + json valide
   - [ ] BDD scenarios `list_buildings_role_based.feature` exécutés sur Postgres réel (pas que mock) → GREEN
   
   ## Liens
   
   - Story 1.2 : #559 (migrations 020000/030000/040000)
   - Story 1.3 : #560 (use_cases + middleware) — **n'a pas touché repository SQL**
   - Story 1.4 : #561 (Building::is_conformant + BuildingMetrics) — **n'a pas touché domain field name**
   - Slice 1 plan : `docs/maury/refonte-ux-multi-role-acp/stories.md` §3
   - Découvert lors de Story 2.1 (#562, agent rapport final)
   
   ## Priorité
   
   **Bloquant pour slice 2** ? Non, slice 2 stories 2.1-2.5 ne dépendent pas de `/buildings` runtime sain (2.1 ne touche pas, 2.2-2.5 dépendent FE selector qui mock l'API en Vitest). Mais bloquant pour toute démo / Story 2.5 E2E réelle.
   
   **Recommandation** : patcher avant slice 2.5 (E2E refonte-ux), idéalement avant 2.4 (Navigation refacto qui demande des buildings réels via API).

.. raw:: html

   </div>


=======================================================================================================================
Issue #603: [SECURITY] Regression GET-by-id verify_org_access skip on 7 handlers post-acp_id migration (#602 follow-up)
=======================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,security
:Assignees: Unassigned
:Created: 2026-05-25
:Updated: 2026-05-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/603>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Lors du hotfix #602 (migration `Building.organization_id` → `acp_id`), **7 handlers GET-by-id** ont vu leur appel `user.verify_org_access(building.organization_id)` **supprimé** car `BuildingResponseDto.organization_id` n'existe plus.
   
   ### Handlers concernés
   
   - `backend/src/infrastructure/web/handlers/building_handlers.rs` (get_building / get_building_with_metrics)
   - `backend/src/infrastructure/web/handlers/budget_handlers.rs`
   - `backend/src/infrastructure/web/handlers/expense_handlers.rs`
   - `backend/src/infrastructure/web/handlers/meeting_handlers.rs`
   - `backend/src/infrastructure/web/handlers/resolution_handlers.rs`
   - `backend/src/infrastructure/web/handlers/unit_handlers.rs`
   - `backend/src/infrastructure/web/handlers/work_report_handlers.rs`
   
   Aussi : `backend/src/application/use_cases/board_member_use_cases.rs` skippe un filtre org→building (même cause).
   
   ## Risque
   
   **Niveau** : Moyen
   **Exploitabilité** : ID-guessing direct (UUIDv4 → faible mais non nul si IDs fuités via logs / GET-list cross-tenant)
   
   Les WHERE-clauses scope au niveau use_case `list_buildings` (Story 1.3) restent en place — donc le LIST endpoint reste isolé multi-tenant. Le risque est **uniquement sur GET /buildings/{id}/...** où un user authentifié peut potentiellement accéder à un building d'une autre organization s'il connait l'UUID.
   
   ## Cause
   
   `Building.organization_id` (champ direct) a été remplacé par `Building.acp_id` (FK vers `acps.id`, qui lui-même a un `organization_id`). Pour vérifier l'org actuel : il faut résoudre `acp_id` → `acp` → `organization_id` via une lookup repository, qui n'a pas été câblée dans les handlers pendant le hotfix #602 (scope strict).
   
   ## Recette proposée
   
   ### Option A — Inject AcpRepository dans handlers (clean)
   
   1. `AppState` : ajouter `acp_use_cases: Arc<AcpUseCases>` (déjà présent ? vérifier `main.rs`)
   2. Helper `helpers/scope_guard.rs` (créé Story 1.3) : ajouter `pub async fn verify_acp_org_access(user: &User, acp_id: Uuid, acp_repo: &dyn AcpRepository) -> Result<(), AppError>` qui résout `acp.organization_id` et appelle `user.verify_org_access`
   3. Appeler ce helper dans les 7 handlers GET-by-id avant de retourner le DTO
   
   ### Option B — Dénormaliser organization_id sur BuildingResponseDto (rapide mais redondant)
   
   Le repository fait un JOIN `buildings JOIN acps ON buildings.acp_id = acps.id` et expose `organization_id` calculé sur le DTO. Simple mais introduit un champ qui ne correspond plus à la colonne DB.
   
   **Recommandation** : Option A, plus propre architecturalement.
   
   ## Critères de validation
   
   - [ ] 7 handlers GET-by-id appellent `verify_acp_org_access` avant retour
   - [ ] BDD scenario `@security` ajouté : "Syndic cabinet A tente GET /buildings/{id_cabinet_B}" → 403
   - [ ] `bdd_list_buildings_role_based` reste GREEN
   - [ ] `buildings_runtime_test` reste GREEN
   - [ ] Pas de régression sur les use_cases qui font déjà la check au niveau use_case
   
   ## Priorité
   
   **Bloquant pour démo prod** : oui (info leak cross-tenant possible)
   **Bloquant pour slice 2** : non, mais à patcher avant 2.5 E2E + avant toute promotion vers `dev` (CI gate).
   
   ## Liens
   
   - Hotfix #602 (commits f6e1d98..59f6079 sur feature/dev)
   - Stories 1.3 (#560) effective_org_filter pattern à étendre

.. raw:: html

   </div>


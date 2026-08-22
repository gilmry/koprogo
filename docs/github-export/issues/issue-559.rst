===============================================================================================
Issue #559: [Story 1.2] Migration data buildings.organization_id → acp_id (3 étapes + rollback)
===============================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,priority:high rust,legal-compliance maury,track-h-conformite slice-1
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/559>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 1.2 — Migration data `buildings.organization_id → acp_id` (3 étapes + rollback)
   
   > Maury Phase 6 Exécution · Slice 1 · Story `story/1.2-buildings-migration-data` · Refs: #556
   
   ## Goal
   
   Migration data en 3 étapes (NULLABLE → backfill → NOT NULL) avec script de rollback complet. Pour chaque `organization` existante : créer ACP miroir, backfill `buildings.acp_id`, supprimer `buildings.organization_id`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR2, FR9 ; INV-1
   - **Effort** : M
   - **Deps** : Story 1.1 (entité Acp existe)
   - **ADR refs** : ADR-0010
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Migration appliquée sur DB de dev (≥1 organization avec ≥1 building) → 0 building orphelin + `audit_event` créé par ACP miroir
   - **@edge** : Organization sans building → ACP miroir créée mais reste sans building (pas d'orphelin)
   - **@security** : Migration nécessite backup explicite (variable `BACKUP_CONFIRMED=true`) ; sinon refuse
   - **@negative** : Migration en 3 étapes interrompue après étape 2 → rollback automatique restaure schema initial sans perte data
   
   ## data-testid
   
   — (migration backend pure)
   
   ## Files
   
   - `backend/migrations/20260601_020000_add_buildings_acp_id.sql` (NULLABLE)
   - `backend/migrations/20260601_030000_backfill_buildings_acp_id.sql` (data)
   - `backend/migrations/20260601_040000_buildings_acp_id_not_null.sql` (ALTER + DROP organization_id)
   - `backend/migrations/20260601_020000_DOWN.sql`, `20260601_030000_DOWN.sql`, `20260601_040000_DOWN.sql`
   - `backend/tests/integration/migration_acp_backfill_test.rs` (testcontainers, valide aller-retour)
   
   ## Definition of Done
   
   - [ ] 3 migrations UP + 3 migrations DOWN écrites et idempotentes
   - [ ] Test integration testcontainer valide UP → DOWN → UP avec data inchangée
   - [ ] Garde `BACKUP_CONFIRMED=true` en place (refus sinon)
   - [ ] 0 building orphelin post-migration (assertion testée)
   - [ ] `audit_event` créé par création d'ACP miroir
   - [ ] Documentation runbook ops (étapes manuelles + commande de rollback)
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §3 Story 1.2
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


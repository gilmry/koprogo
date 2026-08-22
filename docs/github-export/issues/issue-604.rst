================================================================================================================================
Issue #604: [BLOCKER] check_board_syndic_incompatibility trigger references dropped buildings.organization_id (post-ACP refacto)
================================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug
:Assignees: Unassigned
:Created: 2026-05-26
:Updated: 2026-05-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/604>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   `POST /auth/register` avec `role=syndic` retourne **400** systématiquement.
   
   Cause : trigger PL/pgSQL `check_board_syndic_incompatibility` (migration `20251101000002`) référence `b.organization_id` qui a été DROPPED par la migration `20260601040000_buildings_acp_id_not_null.sql` (refacto Story 1.2 ACP).
   
   ## Impact
   
   - **Bloque** tout register syndic via API
   - **Bloque** seed/demo
   - **Bloque** Story 2.5 E2E AC `@happy` + `@edge` (slice 2 refonte-ux), qui exigent un syndic UI-loggable
   
   ## Recette proposée
   
   Patcher les 2 fonctions PL/pgSQL référençant `b.organization_id` :
   1. Localiser trigger + fonction : `backend/migrations/20251101000002_*.sql`
   2. Remplacer par JOIN sur `acps` : `SELECT a.organization_id FROM buildings b JOIN acps a ON a.id = b.acp_id WHERE b.id = ...`
   3. Nouvelle migration `20260602_XXXXXX_fix_board_syndic_trigger_acp_join.sql` (UP + .down.sql) qui DROP TRIGGER+FUNCTION puis CREATE avec JOIN
   4. Test : `POST /auth/register role=syndic` → 201
   
   ## Critères de validation
   
   - [ ] cargo test --test buildings_runtime_test reste GREEN (3/3)
   - [ ] cargo test --test bdd_list_buildings_role_based reste GREEN (8/8)
   - [ ] cargo test --test bdd_acp reste GREEN (17/17)
   - [ ] curl POST /auth/register role=syndic → 201
   - [ ] BDD/E2E Story 2.5 @happy + @edge peuvent runner
   
   ## Priorité
   
   Bloquant pour démo / seed multi-rôle / E2E slice 2 complète.
   
   ## Liens
   
   - Story 1.2 commits : c3dd8ad..dcadd24 (refacto ACP migration)
   - Story 2.5 (#566) — découvert pendant E2E
   - Hotfix #602 (#602) — séries de gaps slice 1
   - Hotfix #603 (#603) — verify_acp_org_access
   
   @gilmry à toi de fermer manuellement quand patch mergé.

.. raw:: html

   </div>


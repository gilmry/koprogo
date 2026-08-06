=====================================================================================================
Issue #660: 3 tests e2e backend pré-existants rouges (Decimal serde / quorum fixtures / ticket stats)
=====================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: None
:Assignees: Unassigned
:Created: 2026-07-25
:Updated: 2026-07-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/660>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Trois tests e2e backend échouent sur `feature/dev`, **indépendamment de Story H15** (`units.organization_id → acp_id`).
   
   **Preuve de non-régression** : `git stash` des changements H15 puis exécution sur la base pré-H15 (`aecf92c`) → **échecs identiques, mêmes lignes**. H15 n'introduit aucune régression.
   
   | Test | Symptôme |
   |---|---|
   | `e2e_unit_owner::test_add_owner_to_unit_success` (+3 autres) | `body["ownership_percentage"]` = `"1.00000"` (Decimal sérialisé en **String**) vs `assert_eq!(…, 1.0)` (float) — `e2e_unit_owner.rs:127` |
   | `e2e_resolutions::test_cast_vote_*` (4 tests) | `create_resolution` répond **sans `id`** → `resolution["id"].as_str().unwrap()` panique (`:518` / `:525`) |
   | `e2e_tickets::test_get_ticket_statistics` | 1 échec isolé (les 16 autres du fichier sont verts) |
   
   ## Causes (hypothèses à confirmer)
   
   1. **Contrat de sérialisation Decimal** — l'API renvoie `"1.00000"` (String) là où le test attend un nombre. À trancher : le **test** doit-il asserter la String / parser, ou le **champ** doit-il sérialiser en number ? ⚠️ Attention au drift `openapi.json` / `api.d.ts` si on change le contrat.
   2. **Interaction avec le gate quorum (Story H10)** — `create_resolution` exige le quorum (Art. 3.87 §5) ; or `create_test_fixtures` crée une réunion **sans** `validate_quorum` → rejet. Les fixtures e2e n'ont pas suivi l'ajout du gate. Même famille que le triage BDD #540 (« spec/fixtures obsolètes vs règle métier »).
   3. **`test_get_ticket_statistics`** — la struct `TicketStatistics` possède bien un champ `total` ([ticket_use_cases.rs:498](backend/src/application/use_cases/ticket_use_cases.rs#L498)), ce n'est donc **pas** un mismatch de clé. Le test poste 3 tickets **sans vérifier leur statut** puis asserte `total >= 3` → suspecter soit un POST ticket qui échoue silencieusement, soit un non-200 sur `/buildings/{id}/tickets/statistics`. Confirmation runtime **bloquée** par le build cassé (issue soeur `lopdf`).
   
   ## Recette
   
   - Un sous-fix par groupe, **RED-first**, en comprenant la cause (test faux / prod faux / spec obsolète).
   - ⛔ **Pas de fix « pour que ça passe »** (règle CRITICAL.md).
   - Pour (1) : décision de contrat explicite + régénération openapi/api.d.ts si le champ change.
   - Pour (2) : poser le quorum dans les fixtures (aligne les e2e sur la règle légale H10).
   
   ## Critères de sortie
   
   - [ ] Les 3 groupes verts.
   - [ ] Cause tracée par groupe (test / prod / spec).
   - [ ] Zéro drift de contrat (gate oasdiff vert) si (1) touche la sérialisation.
   
   ## Contexte
   
   Isolés le 2026-07-24 pendant la validation de Story H15 (Track H #618). H15 a été mergé sur cette base : les échecs lui **pré-existent** et ne le bloquent pas.

.. raw:: html

   </div>


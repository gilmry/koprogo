=========================================================================================================
Issue #550: Playwright E2E — 7 échecs FE séparés de #548 (expense list / meeting detail / resolutions h1)
=========================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,javascript track:software,testing
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/550>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Sur le gate CI de la PR #549 (run `26119007106`, commit `9d081c7`), Playwright E2E remonte **7 échecs** distincts.
   
   Le root-fix #548 (`frontend/src/lib/db.ts` lazy-init, commit `9a55c1a`) est **confirmé efficace** : plus aucune occurrence de `Database not initialized` dans la trace. Les 7 échecs Playwright restants sont **fonctionnels FE, séparés de #548**.
   
   ## Symptômes (traces CI)
   
   3 zones FE distinctes :
   
   1. **Invoices / Expenses** — `Create via API and see it in the list`
      - `await expect(page.locator(`text=${expenseName}`)).toBeVisible({ timeout: 15000 })` → element(s) not found
      - Hypothèse : pas de rafraîchissement de la liste après création via API (cache / store Svelte 5 non invalidé).
   
   2. **Meetings** — `Navigate to meeting detail page`
      - `await expect(page.locator(`text=Detail Meeting ${timestamp}`)).toBeVisible({ timeout: 10000 })` → element(s) not found
      - Confirmé en retry1 (pas du flake).
      - Hypothèse : route ou rendu du détail meeting cassé (guard d'auth/role, ou loading state non résolu).
   
   3. **Resolutions** — `Navigate to meeting detail page`
      - `await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 })` → `Received: hidden`
      - Hypothèse : h1 masqué (CSS `visibility` / `hidden` attr, ou loading state non résolu).
   
   ## Cause probable (à investiguer)
   
   À ce stade : **bug FE fonctionnel**, *pas* le périmètre cache/auth de #548. Cf. mémoire interne `project_frontend-bugs-found.md` (patterns récurrents identifiés lors de précédents passages Playwright).
   
   ## Recette (proposée)
   
   1. Reproduire chaque scénario en local (Playwright headed via docker compose) pour discriminer flake vs bug réel.
   2. Trier par zone — si 3 bugs distincts → 3 sous-tâches (Expense list / Meeting detail / Resolutions h1).
   3. RED-first (test reproducteur isolé) avant fix, selon TDD/BDD 4 catégories (#427).
   
   ## Critères d'acceptation
   
   - [ ] Les 3 scénarios Playwright cités passent en CI (gate `Playwright E2E Tests` vert sur #549 ou successeur).
   - [ ] Couverture `@happy + @edge + @security + @negative` ajoutée si manquante.
   - [ ] Aucun retour de `Database not initialized` (verrouille la garantie #548 sur le long terme).
   
   ## Liens
   
   - Run CI : https://github.com/gilmry/koprogo/actions/runs/26119007106 (job Playwright `76816370655`)
   - PR gate concernée : #549
   - Issue #548 (root-fix cache auth — distinct, déjà fixée)
   
   ## Hors-scope
   
   Ce ticket ne couvre **pas** :
   - La régression `Database not initialized` (= #548, déjà fixée par `9a55c1a`).
   - Le gate IaC lint (couvert par #354 / commits `be87351` + `cc659e3`).

.. raw:: html

   </div>


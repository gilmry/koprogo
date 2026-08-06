=================================================================================
Issue #557: [Story 0.1] Suite caractérisation 6 specs (gel comportement existant)
=================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,testing e2e,playwright maury,track-h-conformite slice-0
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/557>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 0.1 — Suite caractérisation 6 specs (gel comportement existant)
   
   > Maury Phase 6 Exécution · Slice 0 · Story `story/0.1-characterization-suite` · Refs: #556
   
   ## Goal
   
   Créer 6 specs Playwright dans `frontend/tests/e2e/characterization/` qui figent les flows existants HEAD `feature/dev` **avant toute refonte applicative**. Ces specs **DOIVENT rester VERTES sur toutes les slices ultérieures** (gate CI inter-slice Tx.1).
   
   ## Contexte Maury
   
   - **FR/INV** : FR43, FR44 ; pas d'INV (régression safety net pure)
   - **Effort** : M
   - **Deps** : aucune (story d'entrée Phase 6)
   - **ADR refs** : ADR-0013 (arborescence tests caractérisation + refonte)
   - **Cluster coord** : —
   - **Mémoires** : [[fe-refactor-test-driven]] niveau 1, [[multirole-narrative-scenarios]], [[bdd-seed-dates-relative]]
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : login admin/syndic/owner + dashboard initial chacun → screenshots stables, durée < 30s par flow
   - **@edge** : flow building creation admin → assignation organization → visible syndic → 100% VERT
   - **@security** : aucune (pas de nouveau comportement testé, juste fige)
   - **@negative** : si une spec caractérisation tourne ROUGE sur HEAD pré-refonte → STOP. Le test est bugué ou le HEAD est déjà cassé. Investigation avant slice 1.
   
   ## data-testid
   
   Aucun ajouté ici (specs utilisent les sélecteurs existants `getByText`/`role=` — **accepté pour caractérisation, interdit dans `refonte-ux/`**).
   
   ## Files
   
   - `frontend/tests/e2e/characterization/00-login-and-dashboards.spec.ts`
   - `frontend/tests/e2e/characterization/01-building-creation-flow.spec.ts`
   - `frontend/tests/e2e/characterization/02-ag-full-cycle.spec.ts`
   - `frontend/tests/e2e/characterization/03-expense-and-payment.spec.ts`
   - `frontend/tests/e2e/characterization/04-owner-view.spec.ts`
   - `frontend/tests/e2e/characterization/05-notifications-sync.spec.ts`
   - `frontend/tests/e2e/helpers/auth.ts` (loginAsSyndic[WithBuilding], loginAsAdmin, loginAsOwner — réutilisation #550)
   - `playwright.config.ts` (project `characterization` runner séparé)
   - `package.json` (script `test:characterization`)
   
   ## Definition of Done
   
   - [ ] 6 specs créées et tournant VERTES sur HEAD `feature/dev` pré-refonte
   - [ ] Helpers shared multi-rôle (`loginAsAdmin/Syndic/Owner`) en place
   - [ ] `playwright.config.ts` expose le project `characterization` avec runner séparé
   - [ ] Script `npm run test:characterization` opérationnel
   - [ ] CI gate Tx.1 activé (job dédié) — voir story Tx.1
   - [ ] AC 4-cat verts (incl. early-warning @negative si rouge sur HEAD)
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §2 Story 0.1
   - Epic Maury : #556
   - Architecture ADR-0013 : [`docs/maury/refonte-ux-multi-role-acp/architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) §4
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


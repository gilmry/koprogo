=======================================================================
Issue #563: [Story 2.2] Composant BuildingSelector.svelte + store scope
=======================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,maury track-h-conformite,slice-2
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/563>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 2.2 — Composant `BuildingSelector.svelte` + store scope
   
   > Maury Phase 6 Exécution · Slice 2 · Story `story/2.2-building-selector-component` · Refs: #556
   
   ## Goal
   
   Composant global (top-left layout) avec dropdown + autocomplete + favoris star + portefeuilles équipe. Conditionné par rôle (visible si admin/syndic/accountant.*). Store `scope` Svelte 5 runes réactif (selectedBuildingId/AcpId/PortfolioId).
   
   ## Contexte Maury
   
   - **FR/INV** : FR37 ; brief C1
   - **Effort** : M
   - **Deps** : Story 2.1
   - **ADR refs** : ADR-0011, ADR-0012 (data-testid)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic ouvre selector → typing "immeu" → autocomplete 3 résultats < 200ms → click building → store mis à jour → menus contextualisés
   - **@edge** : Cabinet avec 100 ACPs et 500 buildings → autocomplete reste < 200ms (debounce 150ms + pagination 20)
   - **@security** : Owner ne voit pas le selector (RBAC role-based render) ; building cliqué hors scope → 403 + reset selector
   - **@negative** : Aucun building → message "Aucun immeuble dans votre périmètre" + lien vers admin si syndic
   
   ## data-testid
   
   `building-selector-input`, `building-selector-result-{{id}}`, `building-selector-favorite-{{id}}`, `building-selector-clear`
   
   ## Files
   
   - `frontend/src/lib/components/global/BuildingSelector.svelte` (NEW)
   - `frontend/src/stores/scope.svelte.ts` (NEW, Svelte 5 runes)
   - `frontend/src/lib/api/buildings.ts` (extension search endpoint)
   - `frontend/src/lib/components/global/__tests__/BuildingSelector.test.ts` (Vitest RED-GREEN-BLUE)
   - `frontend/src/layouts/AppLayout.astro` (intégration top-left)
   
   ## Definition of Done
   
   - [ ] `BuildingSelector.svelte` créé avec dropdown + autocomplete + favoris
   - [ ] Store `scope.svelte.ts` Svelte 5 runes
   - [ ] API search endpoint avec debounce 150ms + pagination 20
   - [ ] Vitest 4-cat VERT (incl. perf < 200ms)
   - [ ] a11y axe-core VERT sur composant
   - [ ] data-testid systématiques (ADR-0012)
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §4 Story 2.2
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


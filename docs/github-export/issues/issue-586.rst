====================================================================
Issue #586: [Story 5.2] UI ModuleGate.svelte + store enabled_modules
====================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,maury track-h-conformite,slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/586>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.2 — UI `ModuleGate.svelte` + store `enabled_modules`
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.2-module-gate-ui` · Refs: #556
   
   ## Goal
   
   Composant `<ModuleGate module="community">…</ModuleGate>` masque enfants si module désactivé. Store `enabled_modules` synced sur sélection ACP.
   
   ## Contexte Maury
   
   - **FR/INV** : FR39 (UI) ; ADR-0015
   - **Effort** : S
   - **Deps** : Story 5.1
   - **ADR refs** : ADR-0015
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : ACP avec community activé → contenu rendu ; ACP sans → fragment vide (pas placeholder)
   - **@edge** : Bascule ACP en cours de session → store re-fetch + UI re-render
   - **@security** : ModuleGate côté UI ne remplace pas middleware backend (defense-in-depth)
   - **@negative** : Module name inconnu dans gate → erreur typée console + fragment vide
   
   ## data-testid
   
   `module-gate-{{module}}` (présent ssi rendu)
   
   ## Files
   
   - `frontend/src/lib/components/global/ModuleGate.svelte`
   - `frontend/src/stores/enabled_modules.svelte.ts`
   - `frontend/src/lib/api/modules.ts`
   - `frontend/src/lib/components/global/__tests__/ModuleGate.test.ts`
   
   ## Definition of Done
   
   - [ ] Composant ModuleGate avec slot rendering conditionnel
   - [ ] Store enabled_modules Svelte 5 runes synced sur scope
   - [ ] Vitest 4-cat VERT
   - [ ] data-testid présent
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.2
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


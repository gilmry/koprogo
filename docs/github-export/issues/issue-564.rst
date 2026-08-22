===========================================================================
Issue #564: [Story 2.3] Composant ContextBanner.svelte (bannière 3-niveaux)
===========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,maury track-h-conformite,slice-2
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/564>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 2.3 — Composant `ContextBanner.svelte` (bannière 3-niveaux)
   
   > Maury Phase 6 Exécution · Slice 2 · Story `story/2.3-context-banner-component` · Refs: #556
   
   ## Goal
   
   Bannière contextuelle `Cabinet · ACP · Immeuble` quand building sélectionné. Couleur conformité (vert/orange/rouge selon `is_conformant`).
   
   ## Contexte Maury
   
   - **FR/INV** : FR38 ; brief C1, INV-1
   - **Effort** : S
   - **Deps** : Story 1.4 (is_conformant), Story 2.2 (store scope)
   - **ADR refs** : ADR-0012 (data-testid)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Building sélectionné conformant → bannière verte avec 3 niveaux `Cabinet Maury · ACP Résidence X · Immeuble A`
   - **@edge** : ACP auto-gérée (organization_id=null) → bannière 2 niveaux `ACP · Immeuble` (cabinet absent)
   - **@security** : Bannière respecte filtrage rôle (un syndic ne voit pas le cabinet d'un autre)
   - **@negative** : Aucun building sélectionné → bannière masquée (pas placeholder vide)
   
   ## data-testid
   
   `context-banner`, `context-banner-cabinet`, `context-banner-acp`, `context-banner-building`, `context-banner-conformity-icon`
   
   ## Files
   
   - `frontend/src/lib/components/global/ContextBanner.svelte` (NEW)
   - `frontend/src/lib/components/global/__tests__/ContextBanner.test.ts`
   - `frontend/src/layouts/AppLayout.astro` (intégration sous header)
   
   ## Definition of Done
   
   - [ ] `ContextBanner.svelte` créé avec 3 niveaux + couleur conformité
   - [ ] Cas ACP auto-gérée (2 niveaux) géré
   - [ ] Vitest 4-cat VERT
   - [ ] a11y axe-core VERT (contraste WCAG 2.1 AA sur les 3 couleurs conformité)
   - [ ] data-testid présents
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §4 Story 2.3
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


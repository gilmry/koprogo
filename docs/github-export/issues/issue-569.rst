================================================================================
Issue #569: [Story 3.3] PWA Contractor (manifest + Service Worker + UX 3 écrans)
================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,e2e playwright,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/569>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.3 — PWA Contractor (manifest + Service Worker + UX 3 écrans)
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.3-pwa-contractor-3screens` · Refs: #556
   
   ## Goal
   
   PWA install-able sur mobile contractor. 3 écrans max : (1) résumé scope, (2) action (réponse/devis/évaluation), (3) confirmation. Offline-safe (IndexedDB draft).
   
   ## Contexte Maury
   
   - **FR/INV** : FR6 (suite) ; brief C13, mémoire [[fe-refactor-test-driven]]
   - **Effort** : M
   - **Deps** : Story 3.2
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Contractor sur Android Chrome → ouvre `/c/<token>` → install prompt → installe → ouvre PWA → flow 3 écrans → soumet réponse → confirmation
   - **@edge** : Contractor offline → écrit réponse → reconnecté → sync auto IndexedDB → 200 + audit
   - **@security** : PWA installée avec token expiré → écran "Lien expiré" + bouton "Demander un nouveau lien" (mailto syndic)
   - **@negative** : SW cache stale après release → query param `?v=<version>` force re-register + re-fetch (apprentissage #549)
   
   ## data-testid
   
   `pwa-screen-1-summary`, `pwa-screen-2-action`, `pwa-screen-3-confirm`, `pwa-install-prompt`
   
   ## Files
   
   - `frontend/public/manifest.webmanifest` (extension scope `/c/*`)
   - `frontend/public/sw.js` (cache strategy network-first /magic-links + cache-first /c/*)
   - `frontend/src/lib/components/pwa/MagicLinkContractorPage.svelte` (NEW)
   - `frontend/tests/e2e/refonte-ux/slice-3-magic-link-pwa/pwa-contractor.spec.ts` (Playwright `--device "Pixel 7"`)
   
   ## Definition of Done
   
   - [ ] Manifest étendu scope /c/*
   - [ ] Service Worker avec stratégies cache adaptées
   - [ ] Composant 3 écrans Svelte 5
   - [ ] IndexedDB pour drafts offline
   - [ ] Spec Playwright `--device "Pixel 7"` 4-cat VERT
   - [ ] a11y axe-core VERT
   - [ ] Cache busting `?v=<version>` après release
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.3
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


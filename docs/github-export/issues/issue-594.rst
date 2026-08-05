=================================================================================
Issue #594: [Story Tx.2] Helpers shared multi-rôle (extension #550) — closes #550
=================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,testing e2e,playwright maury,track-h-conformite slice-Tx
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/594>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story Tx.2 — Helpers shared multi-rôle (extension #550)
   
   > Maury Phase 6 Exécution · Slice Transversal · Story `story/Tx.2-helpers-shared-multirole` · Refs: #556 · Closes #550
   
   ## Goal
   
   Compléter `frontend/tests/e2e/helpers/auth.ts` avec tous les rôles + variantes WithBuilding/WithAcp/WithMagicLink. Zéro helper local autorisé dans `refonte-ux/`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR44 ; #550
   - **Effort** : S
   - **Deps** : Story 0.1
   - **ADR refs** : ADR-0013
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : `loginAsContractorMagicLink(page, token)` fonctionne sur mobile Pixel 7
   - **@edge** : Helper avec building inexistant → fail clair, pas timeout silencieux
   - **@security** : Helpers ne loggent jamais credentials en clair
   - **@negative** : Helper UI-login détecté dans `refonte-ux/` → CI lint fail
   
   ## Files
   
   - `frontend/tests/e2e/helpers/auth.ts` (extension)
   - `frontend/tests/e2e/helpers/building.ts`
   - `frontend/tests/e2e/helpers/magic-link.ts`
   - `.github/workflows/ci.yml` (lint check)
   
   ## Definition of Done
   
   - [ ] auth.ts couvre tous les rôles métier + variantes (WithBuilding/WithAcp/WithMagicLink)
   - [ ] building.ts + magic-link.ts helpers shared
   - [ ] CI lint refus UI-login local dans refonte-ux/
   - [ ] Closes #550
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §8 Story Tx.2
   - #550 Playwright stratification · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


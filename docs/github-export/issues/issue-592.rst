========================================================================
Issue #592: [Story 5.8] Gate CI a11y axe-core + data-testid + Lighthouse
========================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,testing maury,track-h-conformite slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/592>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.8 — Gate CI a11y axe-core + data-testid + Lighthouse
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.8-ci-a11y-testid-gate` · Refs: #556
   
   ## Goal
   
   CI gate : axe-core ≥ 90 sur pages refonte, ESLint plugin `koprogo-testid-required`, Lighthouse a11y ≥ 90. Bloque PR si violations.
   
   ## Contexte Maury
   
   - **FR/INV** : FR45 ; NFR2, mémoires [[a11y-wcag-aa-baseline]], [[data-testid-systematic]]
   - **Effort** : M
   - **Deps** : toutes stories slice 2-5 (avoir le code à auditer)
   - **ADR refs** : ADR-0012 (data-testid), ADR-0013 (arborescence)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : PR slice 5 mergeable → axe-core PASS + Lighthouse a11y ≥ 90 + zéro testid manquant
   - **@edge** : Composant avec aria-label correct → axe-core PASS même sans testid (lint check séparé)
   - **@security** : Tentative push sans testid sur button interactif → CI fail bloquant (gate dur à partir slice 5)
   - **@negative** : Lighthouse score 89 → fail (≥90 minimum)
   
   ## data-testid
   
   (cible : tous les composants nouveaux/refactorés des slices 1-5)
   
   ## Files
   
   - `.github/workflows/ci.yml` (ajout jobs axe-core + lighthouse + lint-testid)
   - `frontend/eslint-plugins/koprogo-testid-required.js` (NEW)
   - `frontend/playwright.config.ts` (project a11y-audit)
   - `docs/ci/A11Y_GATE.md` (doc gate)
   
   ## Definition of Done
   
   - [ ] Job CI axe-core ≥ 90 (bloquant)
   - [ ] Job CI Lighthouse a11y ≥ 90 (bloquant)
   - [ ] ESLint plugin koprogo-testid-required (bloquant slice 5+)
   - [ ] Doc A11Y_GATE.md publiée
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.8
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


===========================================================
Issue #591: [Story 5.7] Onboarding modulaire wizard ≤ 5 min
===========================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software accessibility,e2e playwright,maury track-h-conformite,slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/591>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.7 — Onboarding modulaire wizard ≤ 5 min
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.7-onboarding-wizard` · Refs: #556
   
   ## Goal
   
   Composant `OnboardingWizard.svelte` 5 étapes : profil ACP → recommandation modules → activation → démo → confirmation. KPI < 5min mesuré client-side.
   
   ## Contexte Maury
   
   - **FR/INV** : FR40 ; SC17, C22
   - **Effort** : M
   - **Deps** : Story 5.1, Story 5.2
   - **ADR refs** : ADR-0015
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : User test naïf complète wizard en 4min23 → analytics enregistre → modules sélectionnés activés
   - **@edge** : User skip recommandation → activation modules par défaut (community + identity)
   - **@security** : Wizard accessible que pour admin SaaS lors création nouvelle ACP, jamais bypass
   - **@negative** : Wizard interrompu mi-parcours → reprise possible (state IndexedDB)
   
   ## data-testid
   
   `onboarding-step-{{n}}`, `onboarding-module-toggle-{{module}}`, `onboarding-finish-submit`
   
   ## Files
   
   - `frontend/src/lib/components/onboarding/OnboardingWizard.svelte` (NEW)
   - `frontend/src/lib/components/onboarding/__tests__/OnboardingWizard.test.ts`
   - `frontend/tests/e2e/refonte-ux/slice-5-modularity/onboarding-wizard.spec.ts`
   
   ## Definition of Done
   
   - [ ] OnboardingWizard.svelte 5 étapes + reprise IndexedDB
   - [ ] Analytics client-side temps réel < 5min
   - [ ] Vitest 4-cat VERT
   - [ ] Playwright spec VERT (incl. @edge skip + @negative reprise)
   - [ ] a11y axe-core VERT (wizard accessible clavier seul)
   - [ ] data-testid présents
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.7
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


==============================================================================
Issue #566: [Story 2.5] E2E refonte-ux slice 2 multi-rôle (admin→syndic→owner)
==============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software testing,e2e playwright,maury track-h-conformite,slice-2
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/566>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 2.5 — E2E refonte-ux slice 2 (multi-rôle narratif)
   
   > Maury Phase 6 Exécution · Slice 2 · Story `story/2.5-e2e-slice-2-multirole` · Refs: #556
   
   ## Goal
   
   Spec Playwright slice 2 multi-rôle : admin crée ACP+building → syndic se logue → sélectionne building → bannière 3 niveaux exacte + menus contextualisés → owner se logue → menus restreints + pas de selector.
   
   ## Contexte Maury
   
   - **FR/INV** : FR4, FR11, FR36-FR38 + FR44 (helpers shared)
   - **Effort** : S
   - **Deps** : Story 1.4, Story 2.2, Story 2.3, Story 2.4
   - **ADR refs** : ADR-0013 (arborescence refonte-ux)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Admin login → POST /acps → POST /buildings → logout. Syndic login → selector OK → banner OK → menus 5 OK
   - **@edge** : Bascule selector building A → building B → menus restent stables (pas de reflow > 100ms)
   - **@security** : Syndic cabinet B login → tente accès URL building cabinet A → 403 + redirect
   - **@negative** : Building non-conformant invisible côté syndic mais visible côté admin
   
   ## data-testid
   
   Utilise ceux des stories 2.1-2.4 (pas de nouveau testid ajouté ici).
   
   ## Files
   
   - `frontend/tests/e2e/refonte-ux/slice-2-selector-banner/admin-creates-syndic-selects.spec.ts`
   
   ## Definition of Done
   
   - [ ] Spec Playwright multi-rôle narratif (admin → syndic → owner) VERT
   - [ ] Utilise helpers shared uniquement (FR44 : zéro helper local)
   - [ ] Sélecteurs `getByTestId()` uniquement (pas `getByText` ni `nth-child`)
   - [ ] 4-cat couverts par 1+ scénarios
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] Closes #553 partiellement (E2E vérifiant bug fix complet via story 1.4)
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §4 Story 2.5
   - Epic Maury : #556
   - Bug d'origine : #553
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


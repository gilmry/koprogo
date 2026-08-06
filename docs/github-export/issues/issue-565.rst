========================================================================================
Issue #565: [Story 2.4] Refacto Navigation.svelte (menus conditionnels rôle + sélection)
========================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software security,accessibility maury,track-h-conformite slice-2
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/565>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 2.4 — Refacto `Navigation.svelte` (menus conditionnels rôle + sélection)
   
   > Maury Phase 6 Exécution · Slice 2 · Story `story/2.4-navigation-conditional-refacto` · Refs: #556
   
   ## Goal
   
   Navigation latérale conditionnée par rôle ET sélection ACP/Building. 5 menus principaux : Gestion, Compta, Gouvernance, Communauté, Ticketing. Sous-menus collapsibles.
   
   ## Contexte Maury
   
   - **FR/INV** : FR4 (UI) ; brief C1, C2
   - **Effort** : M
   - **Deps** : Story 2.2, Story 2.3
   - **ADR refs** : ADR-0012 (data-testid)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic + building sélectionné → 5 menus visibles ; Owner → Communauté + Mes lots visibles seulement
   - **@edge** : Admin sans building sélectionné → menus génériques `/admin/*` ; sélection efface menus admin (mode "in-context")
   - **@security** : Accountant.encodeur n'a pas de menu Communauté ; clic forcé URL `/community` → 403 + redirect
   - **@negative** : User authentifié sans aucun UserRoleAssignment → écran "Aucun rôle attribué — contactez votre administrateur"
   
   ## data-testid
   
   `navigation-menu-gestion`, `navigation-menu-compta`, `navigation-menu-gouvernance`, `navigation-menu-communaute`, `navigation-menu-ticketing`, `navigation-submenu-{{key}}`
   
   ## Files
   
   - `frontend/src/lib/components/navigation/Navigation.svelte` (refacto major)
   - `frontend/src/lib/components/navigation/RoleSubmenu.svelte` (NEW)
   - `frontend/src/lib/auth/permissions.ts` (helpers `canSee(role, menu, scope)`)
   - `frontend/src/lib/components/navigation/__tests__/Navigation.test.ts`
   
   ## Definition of Done
   
   - [ ] `Navigation.svelte` refacto avec 5 menus + sous-menus collapsibles
   - [ ] `RoleSubmenu.svelte` créé (slots pour items)
   - [ ] `permissions.ts` helpers `canSee(role, menu, scope)`
   - [ ] Vitest 4-cat VERT
   - [ ] a11y axe-core VERT (focus visible, keyboard navigation, aria-labels)
   - [ ] data-testid systématiques
   - [ ] Pas de menu visible sans `canSee()` true (defense-in-depth + backend middleware)
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §4 Story 2.4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>


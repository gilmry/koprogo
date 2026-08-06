===========================================================================================================
Issue #410: a11y: keyboard navigation for dropdowns (NotificationDropdown, BuildingSelector) (STORY-P7-805)
===========================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: accessibility,audit-2026-04 svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/410>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 8 — STORY-P7-805. WCAG 2.1 Level AA violation.
   
   The audit found **zero keyboard navigation** in dropdown menus. They only respond to mouse clicks — keyboard-only users cannot open or select items.
   
   Affected components:
   - `src/components/notifications/NotificationDropdown.svelte`
   - `src/components/BuildingSelector.svelte`
   - `src/components/Navigation.svelte` (user menu)
   
   ## Scope
   
   1. Use the existing `handleListKeyboard` helper in `src/lib/accessibility.ts` (already present from earlier work — unused).
   2. For each dropdown:
      - Make the trigger `<button aria-expanded={isOpen} aria-haspopup=\"listbox\">`
      - Wrap the list in `role=\"listbox\"` with items `role=\"option\" tabindex=\"-1\"`
      - Wire `onkeydown`:
        - Arrow Up/Down → move focus between items
        - Enter / Space → activate focused item
        - Escape → close the dropdown and restore focus to the trigger
        - Home / End → jump to first / last item
   
   ## Tests
   
   - Add vitest tests that simulate keydown and assert the correct item receives focus
   - Playwright a11y test (per #406) covers the smoke flow
   
   ## Acceptance
   
   - Tab + arrow keys navigate through any dropdown from end to end
   - axe-core reports 0 `aria-expanded` / `listbox` violations
   
   Effort: ~2h.
   
   Part of Sprint 8 roadmap toward v0.1.0 release.

.. raw:: html

   </div>


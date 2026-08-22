================================================================================
Issue #408: a11y: migrate Modal → AccessibleModal with focus trap (STORY-P7-803)
================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: accessibility,audit-2026-04 svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/408>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 8 — STORY-P7-803. WCAG 2.1 Level AA violation.
   
   The audit found two modal implementations:
   - `src/components/ui/Modal.svelte` — handles Escape but **no focus trap** and **no focus return** on close. Used by 20+ call sites.
   - `src/components/ui/AccessibleModal.svelte` — proper focus trap, announce, restoration. Used rarely.
   
   This makes modal-heavy flows (owner create, unit create, building edit, ticket assign, expense approve) inaccessible to keyboard-only users — focus escapes the modal overlay.
   
   ## Scope
   
   Option A (preferred): Backport focus management from AccessibleModal into Modal so all call sites inherit it without edit.
   - Import `trapFocus`, `FocusManager.push/pop` from `src/lib/accessibility.ts`
   - Apply them in the `\$effect` that watches `isOpen`
   - Remove AccessibleModal (or alias it to Modal)
   
   Option B (fallback): Migrate 20+ call sites to `AccessibleModal`.
   
   ## Tests
   
   - Add a vitest case on Modal: opening with an initial focusable child → focus goes there; Tab from last child loops to first; Escape closes; focus returns to the trigger on close.
   
   ## Acceptance
   
   - axe-core reports 0 focus-related violations (per #406)
   - Manual QA: Tab cycle inside OwnerCreateModal does not escape to the page underneath
   
   Effort: 1–2h.
   
   Part of Sprint 8 roadmap toward v0.1.0 release.

.. raw:: html

   </div>


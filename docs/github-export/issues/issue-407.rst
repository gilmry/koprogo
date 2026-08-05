=========================================================================
Issue #407: a11y: label 95 select + 29 input form controls (STORY-P7-802)
=========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: accessibility,audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/407>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 8 — STORY-P7-802. WCAG 2.1 Level A blocker.
   
   The accessibility audit found **57 of 95 `<select>` elements** and **2 text inputs** without proper `id=` / `<label for=id>` linkage. Screen readers cannot announce these fields.
   
   ## Scope
   
   1. Sweep all `frontend/src/components/**/*.svelte` for `<select`, `<input type="text|email|number|date|...">`, `<textarea`
   2. For each unlabeled control:
      - Add a unique `id=\"component-field\"` attribute
      - Add or link a visible `<label for=\"component-field\">...</label>`
      - If the label must be visually hidden (e.g. search icon + input), use `<label class=\"sr-only\" for=...>`
   3. Update `JournalEntryForm.svelte`, `quotes/*.svelte` (highest concentration per audit)
   
   ## Guard
   
   Add a regex check to the `contract-types` CI job:
   ```bash
   unlabeled=$(grep -cE '<(select|input|textarea)[^>]*>' src/components/**/*.svelte | ...)
   ```
   Target: **0 unlabeled form controls** in production code.
   
   ## Acceptance
   
   - axe-core (per #406) reports 0 Level A form-label violations
   - Keyboard-only users can tab through any form and screen-reader announces each field name
   
   ~2–3h of mechanical work; auto-fix via regex + manual review.
   
   Part of Sprint 8 roadmap toward v0.1.0 release.

.. raw:: html

   </div>


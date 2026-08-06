========================================================================================
Issue #409: a11y: replace title= with aria-label= on 47 icon-only buttons (STORY-P7-804)
========================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: accessibility,audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/409>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 8 — STORY-P7-804. WCAG 2.1 Level A violation.
   
   The audit found **47 buttons** containing only an `<svg>` or emoji as child, using `title=` for their name. HTML `title` is not exposed consistently to assistive tech — the correct attribute is `aria-label`.
   
   Example (from `UnitOwners.svelte`):
   ```svelte
   <!-- current -->
   <button onclick={() => handleEdit(owner)} title="Modifier">✏️</button>
   
   <!-- target -->
   <button onclick={() => handleEdit(owner)} aria-label={\$_('common.edit')}>✏️</button>
   ```
   
   ## Scope
   
   1. Grep: `grep -rEn '<button[^>]*title=' src/components/ | grep -vE 'aria-label'`
   2. For each hit:
      - Replace `title="..."` with `aria-label="..."`
      - If the label is user-visible text, prefer `$_('key')` i18n
      - Ensure the visual tooltip is preserved (native browser tooltip from title works on hover but screen readers need aria-label)
   
   ## High-concentration files
   
   - `src/components/UnitOwners.svelte` — edit/delete rows
   - `src/components/quotes/QuoteList.svelte` — action icons
   - `src/components/tickets/TicketList.svelte` — status toggles
   - `src/components/notifications/NotificationBell.svelte` — bell trigger
   
   ## Acceptance
   
   - axe-core reports 0 `button-name` violations (per #406)
   - `grep -rE '<button[^>]*title=' src/components/ | grep -vE 'aria-label'` returns 0 lines
   
   Effort: ~1h.
   
   Part of Sprint 8 roadmap toward v0.1.0 release.

.. raw:: html

   </div>


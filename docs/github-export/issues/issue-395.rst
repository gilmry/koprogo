========================================================================================
Issue #395: refactor(svelte5): runes migration pilot — 4 badge components (STORY-P7-601)
========================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/395>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-6: Svelte 5 runes migration
   
   Pilot on leaf components (no child deps):
   - TicketStatusBadge, TicketPriorityBadge, ResolutionStatusBadge, PollStatusBadge
   - Pattern: `export let` → `$props()`, `$:` → `$derived()`
   - Svelte 5.55 auto-detects runes per-component, no svelte.config.js needed
   - New docs/MIGRATION_SVELTE5_RUNES.md
   
   ### Commit: `e7e0035`

.. raw:: html

   </div>


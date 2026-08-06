====================================================================================================
Issue #398: refactor(svelte5): migrate TicketCreateModal + TicketAssignModal to runes (STORY-P7-602)
====================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/398>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-6: Runes migration — the B3-v7 building_id race is gone
   
   TicketCreateModal:
   - $props() captures live prop value → no more defensive guard
   - $state() for formData → proper reactivity
   - $effect() for building loading
   
   TicketAssignModal:
   - $state() for assignable users list
   - onassigned callback prop
   
   ### Commit: `7653f7f`

.. raw:: html

   </div>


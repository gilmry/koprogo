====================================================================================
Issue #397: refactor(svelte5): migrate Modal + ConfirmDialog to runes (STORY-P7-602)
====================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/397>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-6: Runes migration — Modal is the most-used shared component
   
   - Modal: dispatch('close') → onclose callback, $effect() for keyboard + scroll lock,
     {@render children/footer} Snippet props
   - ConfirmDialog: must migrate together (slot='footer' incompatible with runes Modal)
   - Discovery: `<svelte:fragment slot='X'>` does NOT auto-translate to Snippet props
   
   ### Commit: `b2b12d5`

.. raw:: html

   </div>


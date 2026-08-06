==============================================================================================================
Issue #396: refactor(svelte5): migrate UI foundation to runes — Button, FormInput, FormTextarea (STORY-P7-602)
==============================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software svelte5-runes
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/396>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-6: Runes migration — bottom-up ui/ module
   
   - Button: $props() + {@render children} + restProps for on:click forwarding
   - FormInput: $bindable() for two-way binding + oninput
   - FormTextarea: same pattern as FormInput
   
   Legacy parents work seamlessly (on:click captured via restProps).
   
   ### Commit: `f36b63b`

.. raw:: html

   </div>


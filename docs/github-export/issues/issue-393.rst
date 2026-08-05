=========================================================================================
Issue #393: feat(test): vitest + @testing-library/svelte unit tests (STORY-P7-6a01/02/03)
=========================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software test-coverage
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/393>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-6a: Test coverage gate
   
   Setup vitest 4.1 + @testing-library/svelte + jsdom.
   64 tests across 13 files covering:
   - ui/: Button (7), Modal (6), FormInput (8), FormTextarea (3), FormSelect (2 spec), ConfirmDialog (3)
   - tickets/: StatusBadge (6), PriorityBadge (5), AssignModal (5)
   - resolutions/: StatusBadge (3), CreateForm (7)
   - polls/: StatusBadge (4)
   - meetings/: QuorumPanel (5)
   
   vitest.config.ts: mode production + browser resolve for Svelte 5 compat.
   
   ### Commits: `e3a5f0f`, `a3e82fc`, `652af9e`

.. raw:: html

   </div>


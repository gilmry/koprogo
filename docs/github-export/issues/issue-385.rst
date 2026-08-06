=====================================================================
Issue #385: i18n: align FR/NL/DE/EN with new enum keys + polls.status
=====================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/385>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Audit ref: v5 #B1, v6 #B1, v7 #B15
   
   - tickets.categories: drop general/emergency, add commonAreas/elevator/landscaping/other
   - resolutions.create.majority*: replace Simple/Qualified with TwoThirds/FourFifths/Unanimity
   - resolutions.create.type*: replace 6 invented types with Ordinary/Extraordinary
   - polls.status: NEW block {draft, active, closed, cancelled}
   - resolutions.vote: add unitLabel + needOwnerAndUnit
   
   ### Commit: `1d01d06`

.. raw:: html

   </div>


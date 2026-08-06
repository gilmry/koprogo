====================================================================================
Issue #384: fix(ui): SEL formatDate import, owner dashboard own units, accent polish
====================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: bug,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/384>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Audit ref: v1 (Lot 1A x3), v7 #B11-B12
   
   - ExchangeDetail: import formatDate (was undefined → crash)
   - OwnerDashboard: fetch /owners/me → own units (not all org units)
   - Navigation sidebar: 'Etats dates' → 'États datés'
   - budgets.astro / etats-dates.astro: restore French accents
   
   ### Commit: `e00b721`

.. raw:: html

   </div>


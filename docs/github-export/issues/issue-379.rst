====================================================================================
Issue #379: feat(tickets): sticky modal, aligned enums, syndic workflow, i18n badges
====================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/379>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Audit ref: v1 #P1, v4 (Urgent), v5 (category 9), v6 #B5
   
   ### Changes
   - TicketCreateModal sticky footer + building_id defensive guard
   - TicketPriority: drop Urgent (4 values)
   - TicketCategory: 9 backend values (drop General/Emergency)
   - TicketStatusBadge/PriorityBadge i18n
   - TicketDetail: syndic can Start/Resolve (canManage || isContractor)
   
   ### Commit: `241a336`

.. raw:: html

   </div>


=========================================================================
Issue #378: feat(tickets): enrich responses with requester/assignee names
=========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/378>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Audit ref: v1 #P1 (requester Inconnu + stats empty)
   
   ### Changes
   - `requester_name` + `assigned_to_name` in TicketResponse DTO
   - `UserUseCases::find_display_name(id)` helper
   - `enrich_ticket()` / `enrich_tickets()` in ticket_handlers.rs
   - TicketStatistics `#[serde(rename)]` to match frontend
   
   ### Commit: `00e156b`

.. raw:: html

   </div>


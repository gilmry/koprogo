=====================================================================================================
Issue #389: feat(frontend): re-export enums from api.d.ts + drop TicketStatus.Assigned (STORY-P7-103)
=====================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software type-safety
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/389>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-1: Type-safe contract
   
   Replace hand-written enums with `components['schemas']['X']` re-exports:
   - resolutions.ts: MajorityType (4), ResolutionType (2)
   - tickets.ts: TicketStatus (5, no Assigned), TicketPriority (4), TicketCategory (9)
   - polls.ts: PollType (4), PollStatus (4)
   - Remove dead code: handleStart, TicketStatus.Assigned references
   
   ### Commit: `a78182f`

.. raw:: html

   </div>


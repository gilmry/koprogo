=======================================================================================================
Issue #392: feat(contractors): Contractor domain role + assignment dropdown (STORY-P7-1001/201/202/203)
=======================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/392>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-2 + P7-10: Contractor as first-class role
   
   - UserRole::Contractor variant + Display/FromStr
   - SQL migration: contractor_profiles table (profession, SIREN/VAT, insurance)
   - Seed: Marc Dubois (plombier) + Sophie Leroux (électricienne)
   - GET /tickets/assignable-users endpoint
   - TicketAssignModal: dropdown instead of UUID text input
   - i18n: tickets.assign.* + roles.* block (FR)
   
   ### Commit: `7aaa787`

.. raw:: html

   </div>


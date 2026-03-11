======================================================
Issue #32: Rewrite E2E tests for unit_owner endpoints
======================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: phase:vps,track:software priority:medium
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/32>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   The file  was removed during CI fixes because it had multiple compilation errors. The file needs to be rewritten to properly test the unit_owner HTTP endpoints.
   
   ## Problems with Original Implementation
   
   1. **Used domain entities directly instead of DTOs**:
      - Called  instead of 
      - Called  instead of 
   
   2. **Missing Serde derives**:
      -  was missing 
   
   3. **Incomplete test coverage**:
      - File was ~600 lines but didn't cover all unit_owner endpoints
   
   ## What Needs to be Done
   
   - [ ] Rewrite E2E tests using proper DTOs (CreateBuildingDto, CreateUnitDto, etc.)
   - [ ] Add  to all DTOs used in HTTP requests
   - [ ] Test all unit_owner HTTP endpoints:
     - GET /units/:id/owners (list owners)
     - POST /units/:id/owners (add owner)
     - DELETE /units/:unit_id/owners/:owner_id (remove owner)
     - PUT /units/:unit_id/owners/:owner_id/percentage (update percentage)
     - PUT /units/:unit_id/owners/:owner_id/primary (set primary contact)
     - POST /units/:unit_id/owners/transfer (transfer ownership)
     - GET /owners/:id/units (get owner's units)
     - GET /units/:id/ownership-history (get history)
   - [ ] Use testcontainers for isolated database per test
   - [ ] Follow patterns from  and 
   
   ## Note
   
   Integration tests in  already provide good coverage of the business logic. E2E tests should focus on HTTP layer (endpoints, auth, JSON serialization).
   
   ## References
   
   - Working examples: , 
   - Handlers: 
   - DTOs: 

.. raw:: html

   </div>


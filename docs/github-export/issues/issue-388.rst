=======================================================================
Issue #388: feat(api): OpenAPI type-gen pipeline (STORY-P7-101/102/104)
=======================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software type-safety
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/388>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Epic P7-1: Type-safe frontend↔backend contract
   
   - export_openapi.rs binary (stdout JSON)
   - Makefile targets: openapi-export, openapi-check, types-sync, seed-reset
   - Annotate 14 enum schemas: Resolution/Meeting/Expense/Poll/Ticket types
   - Regenerate frontend/src/types/api.d.ts from live spec
   - frontend/package.json: types:generate now consumes openapi.json
   
   ### Commits: `50afe10`, `c687170`

.. raw:: html

   </div>


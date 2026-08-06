=======================================================================
Issue #383: feat(api): session-expired toast dedup + mask raw DB errors
=======================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement,track:software audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-16
:Updated: 2026-04-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/383>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Audit ref: v1 #P10, v7 #B13
   
   - 401: clear stale token + dedupe toast (5s flag)
   - Mask Postgres errors (fkey/constraint/sqlx) with generic message
   
   ### Commit: `268a24a`

.. raw:: html

   </div>


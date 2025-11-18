================================
infrastructure/web/middleware.rs
================================

:File: ``infrastructure/web/middleware.rs``
:Lines of Code: 398
:Layer: Infrastructure
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``AuthenticatedUser``
- ``OrganizationId``
- ``GdprRateLimitConfig``
- ``GdprRateLimitState``
- ``GdprRateLimit``
- ``GdprRateLimitMiddleware``

Functions
---------

- ``require_organization()``
- ``new()``
- ``check_rate_limit()``
- ``new()``

Source Code
===========

See: ``backend/src/infrastructure/web/middleware.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


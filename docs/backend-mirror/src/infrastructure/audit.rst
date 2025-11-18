=======================
infrastructure/audit.rs
=======================

:File: ``infrastructure/audit.rs``
:Lines of Code: 455
:Layer: Infrastructure
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``AuditLogEntry``

Enumerations
------------

- ``AuditEventType``

Functions
---------

- ``new()``
- ``with_resource()``
- ``with_client_info()``
- ``with_metadata()``
- ``with_error()``
- ``with_details()``
- ``log()``
- ``log_audit_event()``

Source Code
===========

See: ``backend/src/infrastructure/audit.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


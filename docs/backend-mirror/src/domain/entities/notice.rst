=========================
domain/entities/notice.rs
=========================

:File: ``domain/entities/notice.rs``
:Lines of Code: 915
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Notice``

Enumerations
------------

- ``NoticeType``
- ``NoticeCategory``
- ``NoticeStatus``

Functions
---------

- ``new()``
- ``publish()``
- ``archive()``
- ``expire()``
- ``pin()``
- ``unpin()``
- ``is_expired()``
- ``update_content()``
- ``set_expiration()``

Source Code
===========

See: ``backend/src/domain/entities/notice.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


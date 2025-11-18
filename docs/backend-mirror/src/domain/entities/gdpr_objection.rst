=================================
domain/entities/gdpr_objection.rs
=================================

:File: ``domain/entities/gdpr_objection.rs``
:Lines of Code: 252
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``GdprObjectionRequest``
- ``ProcessingPurpose``

Enumerations
------------

- ``ObjectionStatus``
- ``ObjectionType``

Functions
---------

- ``new()``
- ``accept()``
- ``reject()``
- ``partial_accept()``
- ``is_marketing_objection()``
- ``is_pending()``
- ``get_accepted_purposes()``

Source Code
===========

See: ``backend/src/domain/entities/gdpr_objection.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


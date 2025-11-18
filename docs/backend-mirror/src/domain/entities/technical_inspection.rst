=======================================
domain/entities/technical_inspection.rs
=======================================

:File: ``domain/entities/technical_inspection.rs``
:Lines of Code: 269
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``TechnicalInspection``

Enumerations
------------

- ``InspectionType``
- ``InspectionStatus``

Functions
---------

- ``frequency_days()``
- ``display_name()``
- ``new()``
- ``calculate_next_due_date()``
- ``is_overdue()``
- ``days_until_due()``
- ``mark_overdue()``
- ``add_report()``
- ``add_photo()``
- ``add_certificate()``

Source Code
===========

See: ``backend/src/domain/entities/technical_inspection.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


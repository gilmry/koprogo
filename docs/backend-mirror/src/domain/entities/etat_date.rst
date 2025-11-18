============================
domain/entities/etat_date.rs
============================

:File: ``domain/entities/etat_date.rs``
:Lines of Code: 620
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``EtatDate``

Enumerations
------------

- ``EtatDateStatus``
- ``EtatDateLanguage``

Functions
---------

- ``new()``
- ``mark_in_progress()``
- ``mark_generated()``
- ``mark_delivered()``
- ``is_expired()``
- ``is_overdue()``
- ``days_since_request()``
- ``update_financial_data()``
- ``update_additional_data()``

Source Code
===========

See: ``backend/src/domain/entities/etat_date.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


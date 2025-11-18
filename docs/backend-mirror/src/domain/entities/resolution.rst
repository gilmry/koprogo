=============================
domain/entities/resolution.rs
=============================

:File: ``domain/entities/resolution.rs``
:Lines of Code: 465
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Resolution``

Enumerations
------------

- ``ResolutionType``
- ``MajorityType``
- ``ResolutionStatus``

Functions
---------

- ``new()``
- ``record_vote_pour()``
- ``record_vote_contre()``
- ``record_abstention()``
- ``calculate_result()``
- ``close_voting()``
- ``total_votes()``
- ``pour_percentage()``
- ``contre_percentage()``
- ``abstention_percentage()``

Source Code
===========

See: ``backend/src/domain/entities/resolution.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


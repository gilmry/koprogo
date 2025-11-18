================================
domain/entities/shared_object.rs
================================

:File: ``domain/entities/shared_object.rs``
:Lines of Code: 805
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``SharedObject``

Enumerations
------------

- ``SharedObjectCategory``
- ``ObjectCondition``

Functions
---------

- ``new()``
- ``update()``
- ``mark_available()``
- ``mark_unavailable()``
- ``borrow()``
- ``return_object()``
- ``is_borrowed()``
- ``is_free()``
- ``is_overdue()``
- ``calculate_total_cost()``

*... and 1 more functions*

Source Code
===========

See: ``backend/src/domain/entities/shared_object.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


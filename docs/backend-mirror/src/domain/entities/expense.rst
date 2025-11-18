==========================
domain/entities/expense.rs
==========================

:File: ``domain/entities/expense.rs``
:Lines of Code: 1094
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Expense``

Enumerations
------------

- ``ExpenseCategory``
- ``PaymentStatus``
- ``ApprovalStatus``

Functions
---------

- ``new()``
- ``new_with_vat()``
- ``recalculate_vat()``
- ``submit_for_approval()``
- ``approve()``
- ``reject()``
- ``can_be_modified()``
- ``is_approved()``
- ``mark_as_paid()``
- ``mark_as_overdue()``

*... and 4 more functions*

Source Code
===========

See: ``backend/src/domain/entities/expense.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


===================================
domain/entities/payment_reminder.rs
===================================

:File: ``domain/entities/payment_reminder.rs``
:Lines of Code: 481
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``PaymentReminder``

Enumerations
------------

- ``ReminderLevel``
- ``ReminderStatus``
- ``DeliveryMethod``

Functions
---------

- ``days_after_due_date()``
- ``next_level()``
- ``tone()``
- ``new()``
- ``calculate_penalty()``
- ``mark_as_sent()``
- ``mark_as_opened()``
- ``mark_as_paid()``
- ``escalate()``
- ``cancel()``

*... and 3 more functions*

Source Code
===========

See: ``backend/src/domain/entities/payment_reminder.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


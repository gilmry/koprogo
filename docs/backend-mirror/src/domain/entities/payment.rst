==========================
domain/entities/payment.rs
==========================

:File: ``domain/entities/payment.rs``
:Lines of Code: 519
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Payment``

Enumerations
------------

- ``TransactionStatus``
- ``PaymentMethodType``

Functions
---------

- ``new()``
- ``mark_processing()``
- ``mark_requires_action()``
- ``mark_succeeded()``
- ``mark_failed()``
- ``mark_cancelled()``
- ``refund()``
- ``set_stripe_payment_intent_id()``
- ``set_stripe_customer_id()``
- ``set_payment_method_id()``

*... and 4 more functions*

Source Code
===========

See: ``backend/src/domain/entities/payment.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`


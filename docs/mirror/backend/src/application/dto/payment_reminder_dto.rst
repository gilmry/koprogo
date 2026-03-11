=======================================
application/dto/payment_reminder_dto.rs
=======================================

:Fichier: ``backend/src/application/dto/payment_reminder_dto.rs``
:Type: RUST
:Lignes de Code: 172
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **payment reminder**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``CreatePaymentReminderDto``
- ``PaymentReminderResponseDto``
- ``MarkReminderSentDto``
- ``EscalateReminderDto``
- ``CancelReminderDto``
- ``AddTrackingNumberDto``
- ``PaymentRecoveryStatsDto``
- ``ReminderLevelCountDto``
- ``ReminderStatusCountDto``
- ``OverdueExpenseDto``
- ``BulkCreateRemindersDto``
- ``BulkCreateRemindersResponseDto``

Fonctions
---------

- ``new()``

Code Source
===========

Voir: ``backend/src/application/dto/payment_reminder_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


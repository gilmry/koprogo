===================================
domain/entities/payment_reminder.rs
===================================

:Fichier: ``backend/src/domain/entities/payment_reminder.rs``
:Type: RUST
:Lignes de Code: 481
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **transaction de paiement**. Intégration Stripe avec gestion du lifecycle (Pending → Processing → Succeeded/Failed) et support remboursements.

API Publique
============

Structures
----------

- ``PaymentReminder``

Énumérations
------------

- ``ReminderLevel``
- ``ReminderStatus``
- ``DeliveryMethod``

Fonctions
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
- ``set_tracking_number()``
- ``needs_escalation()``
- ``recalculate_penalties()``

Code Source
===========

Voir: ``backend/src/domain/entities/payment_reminder.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


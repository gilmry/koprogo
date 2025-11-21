===================================================
application/use_cases/payment_reminder_use_cases.rs
===================================================

:Fichier: ``backend/src/application/use_cases/payment_reminder_use_cases.rs``
:Type: RUST
:Lignes de Code: 662
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **payment reminder**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``PaymentReminderUseCases``

Fonctions
---------

- ``new()``
- ``create_reminder()``
- ``get_reminder()``
- ``list_by_expense()``
- ``list_by_owner()``
- ``list_by_organization()``
- ``list_active_by_owner()``
- ``mark_as_sent()``
- ``mark_as_opened()``
- ``mark_as_paid()``
- ``cancel_reminder()``
- ``escalate_reminder()``
- ``add_tracking_number()``
- ``find_pending_reminders()``
- ``find_reminders_needing_escalation()``

*... et 6 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/payment_reminder_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


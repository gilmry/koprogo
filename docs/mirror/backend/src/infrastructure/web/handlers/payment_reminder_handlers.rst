========================================================
infrastructure/web/handlers/payment_reminder_handlers.rs
========================================================

:Fichier: ``backend/src/infrastructure/web/handlers/payment_reminder_handlers.rs``
:Type: RUST
:Lignes de Code: 653
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **payment reminder**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_reminder()``
- ``get_reminder()``
- ``list_by_expense()``
- ``list_by_owner()``
- ``list_active_by_owner()``
- ``list_by_organization()``
- ``mark_as_sent()``
- ``mark_as_opened()``
- ``mark_as_paid()``
- ``cancel_reminder()``
- ``escalate_reminder()``
- ``add_tracking_number()``
- ``get_recovery_stats()``
- ``find_overdue_without_reminders()``
- ``bulk_create_reminders()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/payment_reminder_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


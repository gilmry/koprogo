====================================================
infrastructure/web/handlers/notification_handlers.rs
====================================================

:Fichier: ``backend/src/infrastructure/web/handlers/notification_handlers.rs``
:Type: RUST
:Lignes de Code: 363
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **notification**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_notification()``
- ``get_notification()``
- ``list_my_notifications()``
- ``list_unread_notifications()``
- ``mark_notification_read()``
- ``mark_all_notifications_read()``
- ``delete_notification()``
- ``get_notification_stats()``
- ``get_user_preferences()``
- ``get_preference()``
- ``update_preference()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/notification_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


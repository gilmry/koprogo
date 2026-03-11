===============================================
application/use_cases/notification_use_cases.rs
===============================================

:Fichier: ``backend/src/application/use_cases/notification_use_cases.rs``
:Type: RUST
:Lignes de Code: 321
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **notification**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``NotificationUseCases``

Fonctions
---------

- ``new()``
- ``create_notification()``
- ``get_notification()``
- ``list_user_notifications()``
- ``list_unread_notifications()``
- ``mark_as_read()``
- ``mark_all_read()``
- ``delete_notification()``
- ``get_user_stats()``
- ``get_user_preferences()``
- ``get_preference()``
- ``update_preference()``
- ``get_pending_notifications()``
- ``get_failed_notifications()``
- ``mark_as_sent()``

*... et 3 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/notification_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


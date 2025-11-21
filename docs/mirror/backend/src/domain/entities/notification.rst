===============================
domain/entities/notification.rs
===============================

:Fichier: ``backend/src/domain/entities/notification.rs``
:Type: RUST
:Lignes de Code: 434
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **notification multi-canal**. Support Email, SMS, Push et In-App avec préférences utilisateur granulaires.

API Publique
============

Structures
----------

- ``Notification``
- ``NotificationPreference``

Énumérations
------------

- ``NotificationType``
- ``NotificationChannel``
- ``NotificationStatus``
- ``NotificationPriority``

Fonctions
---------

- ``new()``
- ``with_link()``
- ``with_metadata()``
- ``mark_sent()``
- ``mark_failed()``
- ``mark_read()``
- ``is_unread()``
- ``is_pending()``
- ``retry()``
- ``new()``
- ``set_channel_enabled()``
- ``is_channel_enabled()``

Code Source
===========

Voir: ``backend/src/domain/entities/notification.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


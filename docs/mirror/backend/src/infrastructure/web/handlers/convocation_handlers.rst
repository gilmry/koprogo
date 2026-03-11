===================================================
infrastructure/web/handlers/convocation_handlers.rs
===================================================

:Fichier: ``backend/src/infrastructure/web/handlers/convocation_handlers.rs``
:Type: RUST
:Lignes de Code: 416
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **convocation**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_convocation()``
- ``get_convocation()``
- ``get_convocation_by_meeting()``
- ``list_building_convocations()``
- ``list_organization_convocations()``
- ``delete_convocation()``
- ``schedule_convocation()``
- ``send_convocation()``
- ``cancel_convocation()``
- ``list_convocation_recipients()``
- ``get_convocation_tracking_summary()``
- ``mark_recipient_email_opened()``
- ``update_recipient_attendance()``
- ``set_recipient_proxy()``
- ``send_convocation_reminders()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/convocation_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


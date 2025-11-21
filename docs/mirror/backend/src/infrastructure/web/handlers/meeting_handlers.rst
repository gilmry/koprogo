===============================================
infrastructure/web/handlers/meeting_handlers.rs
===============================================

:Fichier: ``backend/src/infrastructure/web/handlers/meeting_handlers.rs``
:Type: RUST
:Lignes de Code: 530
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **meeting**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_meeting()``
- ``get_meeting()``
- ``list_meetings()``
- ``list_meetings_by_building()``
- ``update_meeting()``
- ``add_agenda_item()``
- ``complete_meeting()``
- ``cancel_meeting()``
- ``reschedule_meeting()``
- ``delete_meeting()``
- ``export_meeting_minutes_pdf()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/meeting_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


========================================================
infrastructure/web/handlers/resource_booking_handlers.rs
========================================================

:Fichier: ``backend/src/infrastructure/web/handlers/resource_booking_handlers.rs``
:Type: RUST
:Lignes de Code: 607
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **resource booking**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``UpcomingQuery``
- ``PastQuery``
- ``CheckConflictsQuery``

Fonctions
---------

- ``create_booking()``
- ``get_booking()``
- ``list_building_bookings()``
- ``list_by_resource_type()``
- ``list_by_resource()``
- ``list_my_bookings()``
- ``list_my_bookings_by_status()``
- ``list_building_bookings_by_status()``
- ``list_upcoming_bookings()``
- ``list_active_bookings()``
- ``list_past_bookings()``
- ``update_booking()``
- ``cancel_booking()``
- ``complete_booking()``
- ``mark_no_show()``

*... et 4 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/resource_booking_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


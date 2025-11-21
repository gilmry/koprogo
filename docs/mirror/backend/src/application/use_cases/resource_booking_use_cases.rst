===================================================
application/use_cases/resource_booking_use_cases.rs
===================================================

:Fichier: ``backend/src/application/use_cases/resource_booking_use_cases.rs``
:Type: RUST
:Lignes de Code: 422
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **resource booking**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``ResourceBookingUseCases``

Fonctions
---------

- ``new()``
- ``create_booking()``
- ``get_booking()``
- ``list_building_bookings()``
- ``list_by_resource_type()``
- ``list_by_resource()``
- ``list_user_bookings()``
- ``list_user_bookings_by_status()``
- ``list_building_bookings_by_status()``
- ``list_upcoming_bookings()``
- ``list_active_bookings()``
- ``list_past_bookings()``
- ``update_booking()``
- ``cancel_booking()``
- ``complete_booking()``

*... et 5 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/resource_booking_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


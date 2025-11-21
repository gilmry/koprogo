===================================
domain/entities/resource_booking.rs
===================================

:Fichier: ``backend/src/domain/entities/resource_booking.rs``
:Type: RUST
:Lignes de Code: 838
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **resource booking**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``ResourceBooking``

Énumérations
------------

- ``ResourceType``
- ``BookingStatus``
- ``RecurringPattern``

Fonctions
---------

- ``new()``
- ``cancel()``
- ``complete()``
- ``mark_no_show()``
- ``confirm()``
- ``update_details()``
- ``is_active()``
- ``is_past()``
- ``is_future()``
- ``duration_hours()``
- ``conflicts_with()``
- ``is_modifiable()``
- ``is_recurring()``

Code Source
===========

Voir: ``backend/src/domain/entities/resource_booking.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


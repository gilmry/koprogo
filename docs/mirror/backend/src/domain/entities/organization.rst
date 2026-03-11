===============================
domain/entities/organization.rs
===============================

:Fichier: ``backend/src/domain/entities/organization.rs``
:Type: RUST
:Lignes de Code: 260
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **organization**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``Organization``

Énumérations
------------

- ``SubscriptionPlan``

Fonctions
---------

- ``new()``
- ``upgrade_plan()``
- ``update_contact()``
- ``deactivate()``
- ``activate()``
- ``can_add_building()``
- ``can_add_user()``

Code Source
===========

Voir: ``backend/src/domain/entities/organization.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


=============================
domain/entities/unit_owner.rs
=============================

:Fichier: ``backend/src/domain/entities/unit_owner.rs``
:Type: RUST
:Lignes de Code: 358
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **unit owner**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``UnitOwner``

Fonctions
---------

- ``new()``
- ``new_with_start_date()``
- ``is_active()``
- ``end_ownership()``
- ``update_percentage()``
- ``set_primary_contact()``

Code Source
===========

Voir: ``backend/src/domain/entities/unit_owner.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


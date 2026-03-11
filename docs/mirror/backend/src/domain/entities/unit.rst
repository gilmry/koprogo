=======================
domain/entities/unit.rs
=======================

:Fichier: ``backend/src/domain/entities/unit.rs``
:Type: RUST
:Lignes de Code: 154
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **lot de copropriété** (appartement, cave, parking). Contient les informations du lot (numéro, étage, surface, type).

API Publique
============

Structures
----------

- ``Unit``

Énumérations
------------

- ``UnitType``

Fonctions
---------

- ``new()``
- ``validate_update()``
- ``assign_owner()``
- ``remove_owner()``

Code Source
===========

Voir: ``backend/src/domain/entities/unit.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


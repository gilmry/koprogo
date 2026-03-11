================================
domain/entities/shared_object.rs
================================

:Fichier: ``backend/src/domain/entities/shared_object.rs``
:Type: RUST
:Lignes de Code: 805
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **shared object**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``SharedObject``

Énumérations
------------

- ``SharedObjectCategory``
- ``ObjectCondition``

Fonctions
---------

- ``new()``
- ``update()``
- ``mark_available()``
- ``mark_unavailable()``
- ``borrow()``
- ``return_object()``
- ``is_borrowed()``
- ``is_free()``
- ``is_overdue()``
- ``calculate_total_cost()``
- ``days_overdue()``

Code Source
===========

Voir: ``backend/src/domain/entities/shared_object.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


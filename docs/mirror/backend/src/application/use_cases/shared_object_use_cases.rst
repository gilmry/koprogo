================================================
application/use_cases/shared_object_use_cases.rs
================================================

:Fichier: ``backend/src/application/use_cases/shared_object_use_cases.rs``
:Type: RUST
:Lignes de Code: 493
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **shared object**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``SharedObjectUseCases``

Fonctions
---------

- ``new()``
- ``create_shared_object()``
- ``get_shared_object()``
- ``list_building_objects()``
- ``list_available_objects()``
- ``list_borrowed_objects()``
- ``list_overdue_objects()``
- ``list_owner_objects()``
- ``list_user_borrowed_objects()``
- ``list_objects_by_category()``
- ``list_free_objects()``
- ``update_shared_object()``
- ``mark_object_available()``
- ``mark_object_unavailable()``
- ``borrow_object()``

*... et 3 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/shared_object_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


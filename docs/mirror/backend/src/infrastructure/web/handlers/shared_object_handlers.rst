=====================================================
infrastructure/web/handlers/shared_object_handlers.rs
=====================================================

:Fichier: ``backend/src/infrastructure/web/handlers/shared_object_handlers.rs``
:Type: RUST
:Lignes de Code: 388
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **shared object**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_shared_object()``
- ``get_shared_object()``
- ``list_building_objects()``
- ``list_available_objects()``
- ``list_borrowed_objects()``
- ``list_overdue_objects()``
- ``list_free_objects()``
- ``list_objects_by_category()``
- ``list_owner_objects()``
- ``list_my_borrowed_objects()``
- ``update_shared_object()``
- ``mark_object_available()``
- ``mark_object_unavailable()``
- ``borrow_object()``
- ``return_object()``

*... et 2 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/shared_object_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


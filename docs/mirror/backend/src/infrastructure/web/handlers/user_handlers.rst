============================================
infrastructure/web/handlers/user_handlers.rs
============================================

:Fichier: ``backend/src/infrastructure/web/handlers/user_handlers.rs``
:Type: RUST
:Lignes de Code: 844
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **user**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``RoleResponse``
- ``UserResponse``
- ``RoleAssignmentRequest``
- ``CreateUserRequest``
- ``UpdateUserRequest``

Fonctions
---------

- ``list_users()``
- ``create_user()``
- ``update_user()``
- ``activate_user()``
- ``deactivate_user()``
- ``delete_user()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/user_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


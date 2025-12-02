=======================================
application/use_cases/auth_use_cases.rs
=======================================

:Fichier: ``backend/src/application/use_cases/auth_use_cases.rs``
:Type: RUST
:Lignes de Code: 485
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **auth**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``AuthUseCases``

Fonctions
---------

- ``new()``
- ``login()``
- ``register()``
- ``switch_active_role()``
- ``get_user_by_id()``
- ``verify_token()``
- ``refresh_token()``
- ``revoke_all_refresh_tokens()``

Code Source
===========

Voir: ``backend/src/application/use_cases/auth_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


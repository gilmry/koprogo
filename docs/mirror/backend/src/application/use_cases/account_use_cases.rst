==========================================
application/use_cases/account_use_cases.rs
==========================================

:Fichier: ``backend/src/application/use_cases/account_use_cases.rs``
:Type: RUST
:Lignes de Code: 990
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **account**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``AccountUseCases``

Fonctions
---------

- ``new()``
- ``create_account()``
- ``get_account()``
- ``get_account_by_code()``
- ``list_accounts()``
- ``list_accounts_by_type()``
- ``list_child_accounts()``
- ``list_direct_use_accounts()``
- ``search_accounts()``
- ``update_account()``
- ``delete_account()``
- ``count_accounts()``
- ``seed_belgian_pcmn()``

Code Source
===========

Voir: ``backend/src/application/use_cases/account_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


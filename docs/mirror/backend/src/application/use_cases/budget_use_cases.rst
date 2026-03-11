=========================================
application/use_cases/budget_use_cases.rs
=========================================

:Fichier: ``backend/src/application/use_cases/budget_use_cases.rs``
:Type: RUST
:Lignes de Code: 270
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **budget**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``BudgetUseCases``

Fonctions
---------

- ``new()``
- ``create_budget()``
- ``get_budget()``
- ``get_by_building_and_fiscal_year()``
- ``get_active_budget()``
- ``list_by_building()``
- ``list_by_fiscal_year()``
- ``list_by_status()``
- ``list_paginated()``
- ``update_budget()``
- ``submit_for_approval()``
- ``approve_budget()``
- ``reject_budget()``
- ``archive_budget()``
- ``delete_budget()``

*... et 2 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/budget_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


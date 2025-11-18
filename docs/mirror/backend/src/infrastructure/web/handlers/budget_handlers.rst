==============================================
infrastructure/web/handlers/budget_handlers.rs
==============================================

:Fichier: ``backend/src/infrastructure/web/handlers/budget_handlers.rs``
:Type: RUST
:Lignes de Code: 468
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **budget**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_budget()``
- ``get_budget()``
- ``get_budget_by_building_and_fiscal_year()``
- ``get_active_budget()``
- ``list_budgets_by_building()``
- ``list_budgets_by_fiscal_year()``
- ``list_budgets_by_status()``
- ``list_budgets()``
- ``update_budget()``
- ``submit_budget()``
- ``approve_budget()``
- ``reject_budget()``
- ``archive_budget()``
- ``get_budget_stats()``
- ``get_budget_variance()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/budget_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


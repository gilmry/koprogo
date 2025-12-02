=============================================
infrastructure/web/handlers/stats_handlers.rs
=============================================

:Fichier: ``backend/src/infrastructure/web/handlers/stats_handlers.rs``
:Type: RUST
:Lignes de Code: 651
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **stats**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``DashboardStats``
- ``SeedDataStats``
- ``SyndicDashboardStats``
- ``NextMeetingInfo``
- ``UrgentTask``

Fonctions
---------

- ``get_dashboard_stats()``
- ``get_owner_stats()``
- ``get_syndic_stats()``
- ``get_syndic_urgent_tasks()``
- ``get_seed_data_stats()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/stats_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


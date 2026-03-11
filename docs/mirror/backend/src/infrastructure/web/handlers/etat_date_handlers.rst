=================================================
infrastructure/web/handlers/etat_date_handlers.rs
=================================================

:Fichier: ``backend/src/infrastructure/web/handlers/etat_date_handlers.rs``
:Type: RUST
:Lignes de Code: 400
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **etat date**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_etat_date()``
- ``get_etat_date()``
- ``get_by_reference_number()``
- ``list_etats_dates()``
- ``list_etats_dates_by_unit()``
- ``list_etats_dates_by_building()``
- ``mark_in_progress()``
- ``mark_generated()``
- ``mark_delivered()``
- ``update_financial_data()``
- ``update_additional_data()``
- ``list_overdue()``
- ``list_expired()``
- ``get_stats()``
- ``delete_etat_date()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/etat_date_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


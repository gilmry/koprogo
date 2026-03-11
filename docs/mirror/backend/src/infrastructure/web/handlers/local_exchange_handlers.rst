======================================================
infrastructure/web/handlers/local_exchange_handlers.rs
======================================================

:Fichier: ``backend/src/infrastructure/web/handlers/local_exchange_handlers.rs``
:Type: RUST
:Lignes de Code: 346
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **local exchange**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_exchange()``
- ``get_exchange()``
- ``list_building_exchanges()``
- ``list_available_exchanges()``
- ``list_owner_exchanges()``
- ``list_exchanges_by_type()``
- ``request_exchange()``
- ``start_exchange()``
- ``complete_exchange()``
- ``cancel_exchange()``
- ``rate_provider()``
- ``rate_requester()``
- ``delete_exchange()``
- ``get_credit_balance()``
- ``get_leaderboard()``

*... et 2 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/local_exchange_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


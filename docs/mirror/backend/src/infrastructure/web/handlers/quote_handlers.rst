=============================================
infrastructure/web/handlers/quote_handlers.rs
=============================================

:Fichier: ``backend/src/infrastructure/web/handlers/quote_handlers.rs``
:Type: RUST
:Lignes de Code: 322
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **quote**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_quote()``
- ``get_quote()``
- ``list_building_quotes()``
- ``list_contractor_quotes()``
- ``list_quotes_by_status()``
- ``submit_quote()``
- ``start_review()``
- ``accept_quote()``
- ``reject_quote()``
- ``withdraw_quote()``
- ``compare_quotes()``
- ``update_contractor_rating()``
- ``delete_quote()``
- ``count_building_quotes()``
- ``count_quotes_by_status()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/quote_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


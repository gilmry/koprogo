==============================================
infrastructure/web/handlers/notice_handlers.rs
==============================================

:Fichier: ``backend/src/infrastructure/web/handlers/notice_handlers.rs``
:Type: RUST
:Lignes de Code: 417
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **notice**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_notice()``
- ``get_notice()``
- ``list_building_notices()``
- ``list_published_notices()``
- ``list_pinned_notices()``
- ``list_notices_by_type()``
- ``list_notices_by_category()``
- ``list_notices_by_status()``
- ``list_author_notices()``
- ``update_notice()``
- ``publish_notice()``
- ``archive_notice()``
- ``pin_notice()``
- ``unpin_notice()``
- ``set_expiration()``

*... et 2 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/notice_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


================================================
infrastructure/web/handlers/document_handlers.rs
================================================

:Fichier: ``backend/src/infrastructure/web/handlers/document_handlers.rs``
:Type: RUST
:Lignes de Code: 311
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **document**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``UploadForm``

Fonctions
---------

- ``upload_document()``
- ``get_document()``
- ``list_documents()``
- ``download_document()``
- ``list_documents_by_building()``
- ``list_documents_by_meeting()``
- ``list_documents_by_expense()``
- ``link_document_to_meeting()``
- ``link_document_to_expense()``
- ``delete_document()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/document_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


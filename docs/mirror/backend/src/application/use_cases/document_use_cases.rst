===========================================
application/use_cases/document_use_cases.rs
===========================================

:Fichier: ``backend/src/application/use_cases/document_use_cases.rs``
:Type: RUST
:Lignes de Code: 420
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **document**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``DocumentUseCases``

Fonctions
---------

- ``new()``
- ``upload_document()``
- ``get_document()``
- ``download_document()``
- ``list_documents_by_building()``
- ``list_documents_by_meeting()``
- ``list_documents_by_expense()``
- ``list_documents_paginated()``
- ``link_to_meeting()``
- ``link_to_expense()``
- ``delete_document()``

Code Source
===========

Voir: ``backend/src/application/use_cases/document_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


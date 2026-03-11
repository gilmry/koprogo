=============================
application/dto/pagination.rs
=============================

:Fichier: ``backend/src/application/dto/pagination.rs``
:Type: RUST
:Lignes de Code: 271
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **pagination**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``PageRequest``
- ``PageResponse``
- ``PaginationMeta``

Énumérations
------------

- ``SortOrder``

Fonctions
---------

- ``offset()``
- ``limit()``
- ``validate()``
- ``validate_api()``
- ``to_sql()``
- ``new()``
- ``new()``

Code Source
===========

Voir: ``backend/src/application/dto/pagination.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


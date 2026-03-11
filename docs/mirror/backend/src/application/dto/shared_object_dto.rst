====================================
application/dto/shared_object_dto.rs
====================================

:Fichier: ``backend/src/application/dto/shared_object_dto.rs``
:Type: RUST
:Lignes de Code: 246
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **shared object**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``CreateSharedObjectDto``
- ``UpdateSharedObjectDto``
- ``BorrowObjectDto``
- ``SharedObjectResponseDto``
- ``SharedObjectSummaryDto``
- ``SharedObjectStatisticsDto``
- ``CategoryObjectCount``

Fonctions
---------

- ``from_shared_object()``
- ``from_shared_object()``

Code Source
===========

Voir: ``backend/src/application/dto/shared_object_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


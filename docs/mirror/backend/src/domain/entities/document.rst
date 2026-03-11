===========================
domain/entities/document.rs
===========================

:Fichier: ``backend/src/domain/entities/document.rs``
:Type: RUST
:Lignes de Code: 167
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **document**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``Document``

Énumérations
------------

- ``DocumentType``

Fonctions
---------

- ``new()``
- ``link_to_meeting()``
- ``link_to_expense()``
- ``file_size_mb()``

Code Source
===========

Voir: ``backend/src/domain/entities/document.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


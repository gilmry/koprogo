==============================
domain/entities/work_report.rs
==============================

:Fichier: ``backend/src/domain/entities/work_report.rs``
:Type: RUST
:Lignes de Code: 202
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **work report**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``WorkReport``

Énumérations
------------

- ``WorkType``
- ``WarrantyType``

Fonctions
---------

- ``new()``
- ``is_warranty_valid()``
- ``warranty_days_remaining()``
- ``add_photo()``
- ``add_document()``

Code Source
===========

Voir: ``backend/src/domain/entities/work_report.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


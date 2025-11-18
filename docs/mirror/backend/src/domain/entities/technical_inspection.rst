=======================================
domain/entities/technical_inspection.rs
=======================================

:Fichier: ``backend/src/domain/entities/technical_inspection.rs``
:Type: RUST
:Lignes de Code: 269
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **technical inspection**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``TechnicalInspection``

Énumérations
------------

- ``InspectionType``
- ``InspectionStatus``

Fonctions
---------

- ``frequency_days()``
- ``display_name()``
- ``new()``
- ``calculate_next_due_date()``
- ``is_overdue()``
- ``days_until_due()``
- ``mark_overdue()``
- ``add_report()``
- ``add_photo()``
- ``add_certificate()``

Code Source
===========

Voir: ``backend/src/domain/entities/technical_inspection.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


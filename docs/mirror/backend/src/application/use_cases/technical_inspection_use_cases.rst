=======================================================
application/use_cases/technical_inspection_use_cases.rs
=======================================================

:Fichier: ``backend/src/application/use_cases/technical_inspection_use_cases.rs``
:Type: RUST
:Lignes de Code: 369
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **technical inspection**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``TechnicalInspectionUseCases``

Fonctions
---------

- ``new()``
- ``create_technical_inspection()``
- ``get_technical_inspection()``
- ``list_technical_inspections_by_building()``
- ``list_technical_inspections_by_organization()``
- ``list_technical_inspections_paginated()``
- ``get_overdue_inspections()``
- ``get_upcoming_inspections()``
- ``get_inspections_by_type()``
- ``update_technical_inspection()``
- ``mark_as_completed()``
- ``add_report()``
- ``add_photo()``
- ``add_certificate()``
- ``delete_technical_inspection()``

Code Source
===========

Voir: ``backend/src/application/use_cases/technical_inspection_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


==============================================
application/use_cases/work_report_use_cases.rs
==============================================

:Fichier: ``backend/src/application/use_cases/work_report_use_cases.rs``
:Type: RUST
:Lignes de Code: 296
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **work report**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``WorkReportUseCases``

Fonctions
---------

- ``new()``
- ``create_work_report()``
- ``get_work_report()``
- ``list_work_reports_by_building()``
- ``list_work_reports_by_organization()``
- ``list_work_reports_paginated()``
- ``get_active_warranties()``
- ``get_expiring_warranties()``
- ``update_work_report()``
- ``add_photo()``
- ``add_document()``
- ``delete_work_report()``

Code Source
===========

Voir: ``backend/src/application/use_cases/work_report_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


==============================
domain/entities/gdpr_export.rs
==============================

:Fichier: ``backend/src/domain/entities/gdpr_export.rs``
:Type: RUST
:Lignes de Code: 327
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **gdpr export**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``GdprExport``
- ``UserData``
- ``OwnerData``
- ``RelatedData``
- ``UnitOwnershipData``
- ``ExpenseData``
- ``DocumentData``
- ``MeetingData``

Fonctions
---------

- ``new()``
- ``add_owner_profile()``
- ``add_unit_ownership()``
- ``add_expense()``
- ``add_document()``
- ``add_meeting()``
- ``is_anonymized()``
- ``total_items()``

Code Source
===========

Voir: ``backend/src/domain/entities/gdpr_export.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


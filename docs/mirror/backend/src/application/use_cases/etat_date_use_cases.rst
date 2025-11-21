============================================
application/use_cases/etat_date_use_cases.rs
============================================

:Fichier: ``backend/src/application/use_cases/etat_date_use_cases.rs``
:Type: RUST
:Lignes de Code: 263
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **etat date**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``EtatDateUseCases``

Fonctions
---------

- ``new()``
- ``create_etat_date()``
- ``get_etat_date()``
- ``get_by_reference_number()``
- ``list_by_unit()``
- ``list_by_building()``
- ``list_paginated()``
- ``mark_in_progress()``
- ``mark_generated()``
- ``mark_delivered()``
- ``update_financial_data()``
- ``update_additional_data()``
- ``list_overdue()``
- ``list_expired()``
- ``delete_etat_date()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/etat_date_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


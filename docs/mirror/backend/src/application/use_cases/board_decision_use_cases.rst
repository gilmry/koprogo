=================================================
application/use_cases/board_decision_use_cases.rs
=================================================

:Fichier: ``backend/src/application/use_cases/board_decision_use_cases.rs``
:Type: RUST
:Lignes de Code: 601
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **board decision**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``BoardDecisionUseCases``

Fonctions
---------

- ``new()``
- ``create_decision()``
- ``get_decision()``
- ``list_decisions_by_building()``
- ``list_decisions_by_status()``
- ``list_overdue_decisions()``
- ``update_decision_status()``
- ``add_notes()``
- ``complete_decision()``
- ``get_decision_stats()``

Code Source
===========

Voir: ``backend/src/application/use_cases/board_decision_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


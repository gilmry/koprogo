=================================
domain/entities/board_decision.rs
=================================

:Fichier: ``backend/src/domain/entities/board_decision.rs``
:Type: RUST
:Lignes de Code: 469
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **board decision**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``BoardDecision``

Énumérations
------------

- ``DecisionStatus``

Fonctions
---------

- ``new()``
- ``is_overdue()``
- ``update_status()``
- ``add_notes()``
- ``check_and_update_overdue_status()``

Code Source
===========

Voir: ``backend/src/domain/entities/board_decision.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


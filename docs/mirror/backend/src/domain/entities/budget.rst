=========================
domain/entities/budget.rs
=========================

:Fichier: ``backend/src/domain/entities/budget.rs``
:Type: RUST
:Lignes de Code: 410
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **budget annuel de copropriété**. Conformité loi belge avec catégories de charges et prévisions.

API Publique
============

Structures
----------

- ``Budget``

Énumérations
------------

- ``BudgetStatus``

Fonctions
---------

- ``new()``
- ``submit_for_approval()``
- ``approve()``
- ``reject()``
- ``archive()``
- ``update_amounts()``
- ``update_notes()``
- ``is_active()``
- ``is_editable()``

Code Source
===========

Voir: ``backend/src/domain/entities/budget.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


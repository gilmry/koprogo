=================================
domain/entities/call_for_funds.rs
=================================

:Fichier: ``backend/src/domain/entities/call_for_funds.rs``
:Type: RUST
:Lignes de Code: 264
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **call for funds**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``CallForFunds``

Énumérations
------------

- ``CallForFundsStatus``

Fonctions
---------

- ``new()``
- ``mark_as_sent()``
- ``mark_as_completed()``
- ``cancel()``
- ``is_overdue()``

Code Source
===========

Voir: ``backend/src/domain/entities/call_for_funds.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


===============================
domain/entities/board_member.rs
===============================

:Fichier: ``backend/src/domain/entities/board_member.rs``
:Type: RUST
:Lignes de Code: 453
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **board member**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``BoardMember``

Énumérations
------------

- ``BoardPosition``

Fonctions
---------

- ``new()``
- ``is_active()``
- ``days_remaining()``
- ``expires_soon()``
- ``extend_mandate()``

Code Source
===========

Voir: ``backend/src/domain/entities/board_member.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


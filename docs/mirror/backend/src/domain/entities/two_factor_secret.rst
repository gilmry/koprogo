====================================
domain/entities/two_factor_secret.rs
====================================

:Fichier: ``backend/src/domain/entities/two_factor_secret.rs``
:Type: RUST
:Lignes de Code: 320
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **two factor secret**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``TwoFactorSecret``

Fonctions
---------

- ``new()``
- ``with_backup_codes()``
- ``enable()``
- ``disable()``
- ``mark_used()``
- ``regenerate_backup_codes()``
- ``remove_backup_code()``
- ``backup_codes_low()``
- ``needs_reverification()``

Code Source
===========

Voir: ``backend/src/domain/entities/two_factor_secret.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


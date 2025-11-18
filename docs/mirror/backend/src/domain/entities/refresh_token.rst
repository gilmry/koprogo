================================
domain/entities/refresh_token.rs
================================

:Fichier: ``backend/src/domain/entities/refresh_token.rs``
:Type: RUST
:Lignes de Code: 93
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **refresh token**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``RefreshToken``

Fonctions
---------

- ``new()``
- ``is_expired()``
- ``is_valid()``
- ``revoke()``

Code Source
===========

Voir: ``backend/src/domain/entities/refresh_token.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


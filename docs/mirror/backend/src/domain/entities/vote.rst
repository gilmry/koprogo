=======================
domain/entities/vote.rs
=======================

:Fichier: ``backend/src/domain/entities/vote.rs``
:Type: RUST
:Lignes de Code: 265
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **vote individuel** sur une résolution. Support vote par procuration et traçabilité GDPR complète.

API Publique
============

Structures
----------

- ``Vote``

Énumérations
------------

- ``VoteChoice``

Fonctions
---------

- ``new()``
- ``is_proxy_vote()``
- ``effective_voter_id()``
- ``change_vote()``

Code Source
===========

Voir: ``backend/src/domain/entities/vote.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


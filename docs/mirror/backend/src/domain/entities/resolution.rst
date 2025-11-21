=============================
domain/entities/resolution.rs
=============================

:Fichier: ``backend/src/domain/entities/resolution.rs``
:Type: RUST
:Lignes de Code: 465
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **résolution d'assemblée générale**. Implémente les 3 types de majorité belges (Simple, Absolue, Qualifiée) et le système de tantièmes.

API Publique
============

Structures
----------

- ``Resolution``

Énumérations
------------

- ``ResolutionType``
- ``MajorityType``
- ``ResolutionStatus``

Fonctions
---------

- ``new()``
- ``record_vote_pour()``
- ``record_vote_contre()``
- ``record_abstention()``
- ``calculate_result()``
- ``close_voting()``
- ``total_votes()``
- ``pour_percentage()``
- ``contre_percentage()``
- ``abstention_percentage()``

Code Source
===========

Voir: ``backend/src/domain/entities/resolution.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


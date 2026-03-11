============================
domain/entities/challenge.rs
============================

:Fichier: ``backend/src/domain/entities/challenge.rs``
:Type: RUST
:Lignes de Code: 613
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **challenge de gamification**. Time-bound avec métriques cibles et récompenses en points (Individual/Team/Building).

API Publique
============

Structures
----------

- ``Challenge``
- ``ChallengeProgress``

Énumérations
------------

- ``ChallengeStatus``
- ``ChallengeType``

Fonctions
---------

- ``new()``
- ``activate()``
- ``complete()``
- ``cancel()``
- ``is_currently_active()``
- ``has_ended()``
- ``duration_days()``
- ``update()``
- ``update_title()``
- ``update_description()``
- ``update_icon()``
- ``update_start_date()``
- ``update_end_date()``
- ``update_target_value()``
- ``update_reward_points()``

*... et 4 autres fonctions*

Code Source
===========

Voir: ``backend/src/domain/entities/challenge.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


==============================
domain/entities/achievement.rs
==============================

:Fichier: ``backend/src/domain/entities/achievement.rs``
:Type: RUST
:Lignes de Code: 526
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **achievement de gamification**. 8 catégories, 5 tiers (Bronze → Diamond), support achievements secrets et répétables.

API Publique
============

Structures
----------

- ``Achievement``
- ``UserAchievement``

Énumérations
------------

- ``AchievementCategory``
- ``AchievementTier``

Fonctions
---------

- ``new()``
- ``update()``
- ``default_points_for_tier()``
- ``update_name()``
- ``update_description()``
- ``update_icon()``
- ``update_points_value()``
- ``update_requirements()``
- ``new()``
- ``increment_earned()``
- ``repeat_earn()``

Code Source
===========

Voir: ``backend/src/domain/entities/achievement.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


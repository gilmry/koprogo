===============================================
application/use_cases/gamification_use_cases.rs
===============================================

:Fichier: ``backend/src/application/use_cases/gamification_use_cases.rs``
:Type: RUST
:Lignes de Code: 677
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **gamification**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``AchievementUseCases``
- ``ChallengeUseCases``
- ``GamificationStatsUseCases``

Fonctions
---------

- ``new()``
- ``create_achievement()``
- ``get_achievement()``
- ``list_achievements()``
- ``list_achievements_by_category()``
- ``list_visible_achievements()``
- ``update_achievement()``
- ``delete_achievement()``
- ``award_achievement()``
- ``get_user_achievements()``
- ``get_recent_achievements()``
- ``new()``
- ``create_challenge()``
- ``get_challenge()``
- ``list_challenges()``

*... et 15 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/gamification_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


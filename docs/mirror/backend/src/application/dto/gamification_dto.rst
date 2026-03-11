===================================
application/dto/gamification_dto.rs
===================================

:Fichier: ``backend/src/application/dto/gamification_dto.rs``
:Type: RUST
:Lignes de Code: 385
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **gamification**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``CreateAchievementDto``
- ``UpdateAchievementDto``
- ``AchievementResponseDto``
- ``UserAchievementResponseDto``
- ``CreateChallengeDto``
- ``UpdateChallengeDto``
- ``ChallengeResponseDto``
- ``ChallengeProgressResponseDto``
- ``LeaderboardEntryDto``
- ``LeaderboardResponseDto``
- ``UserGamificationStatsDto``

Fonctions
---------

- ``from_entities()``
- ``from_entities()``

Code Source
===========

Voir: ``backend/src/application/dto/gamification_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


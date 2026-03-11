====================================================
infrastructure/web/handlers/gamification_handlers.rs
====================================================

:Fichier: ``backend/src/infrastructure/web/handlers/gamification_handlers.rs``
:Type: RUST
:Lignes de Code: 624
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **gamification**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``AwardAchievementRequest``
- ``IncrementProgressRequest``

Fonctions
---------

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
- ``create_challenge()``
- ``get_challenge()``
- ``list_challenges()``
- ``list_challenges_by_status()``
- ``list_building_challenges()``

*... et 12 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/gamification_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


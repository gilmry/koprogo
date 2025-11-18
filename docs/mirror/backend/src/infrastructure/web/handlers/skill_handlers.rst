=============================================
infrastructure/web/handlers/skill_handlers.rs
=============================================

:Fichier: ``backend/src/infrastructure/web/handlers/skill_handlers.rs``
:Type: RUST
:Lignes de Code: 320
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **skill**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_skill()``
- ``get_skill()``
- ``list_building_skills()``
- ``list_available_skills()``
- ``list_free_skills()``
- ``list_professional_skills()``
- ``list_skills_by_category()``
- ``list_skills_by_expertise()``
- ``list_owner_skills()``
- ``update_skill()``
- ``mark_skill_available()``
- ``mark_skill_unavailable()``
- ``delete_skill()``
- ``get_skill_statistics()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/skill_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


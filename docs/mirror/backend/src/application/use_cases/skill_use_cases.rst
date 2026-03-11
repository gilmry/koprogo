========================================
application/use_cases/skill_use_cases.rs
========================================

:Fichier: ``backend/src/application/use_cases/skill_use_cases.rs``
:Type: RUST
:Lignes de Code: 380
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **skill**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``SkillUseCases``

Fonctions
---------

- ``new()``
- ``create_skill()``
- ``get_skill()``
- ``list_building_skills()``
- ``list_available_skills()``
- ``list_owner_skills()``
- ``list_skills_by_category()``
- ``list_skills_by_expertise()``
- ``list_free_skills()``
- ``list_professional_skills()``
- ``update_skill()``
- ``mark_skill_available()``
- ``mark_skill_unavailable()``
- ``delete_skill()``
- ``get_skill_statistics()``

Code Source
===========

Voir: ``backend/src/application/use_cases/skill_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


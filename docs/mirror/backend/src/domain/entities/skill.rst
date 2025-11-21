========================
domain/entities/skill.rs
========================

:Fichier: ``backend/src/domain/entities/skill.rs``
:Type: RUST
:Lignes de Code: 629
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **skill**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``Skill``

Énumérations
------------

- ``SkillCategory``
- ``ExpertiseLevel``

Fonctions
---------

- ``new()``
- ``update()``
- ``mark_available()``
- ``mark_unavailable()``
- ``is_free()``
- ``is_professional()``

Code Source
===========

Voir: ``backend/src/domain/entities/skill.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


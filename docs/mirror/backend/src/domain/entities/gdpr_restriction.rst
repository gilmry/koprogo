===================================
domain/entities/gdpr_restriction.rs
===================================

:Fichier: ``backend/src/domain/entities/gdpr_restriction.rs``
:Type: RUST
:Lignes de Code: 216
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **gdpr restriction**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``GdprRestrictionRequest``

Énumérations
------------

- ``RestrictionStatus``
- ``RestrictionReason``

Fonctions
---------

- ``new()``
- ``activate()``
- ``lift()``
- ``reject()``
- ``is_active()``
- ``is_pending()``

Code Source
===========

Voir: ``backend/src/domain/entities/gdpr_restriction.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


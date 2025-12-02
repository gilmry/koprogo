=====================================
domain/entities/gdpr_rectification.rs
=====================================

:Fichier: ``backend/src/domain/entities/gdpr_rectification.rs``
:Type: RUST
:Lignes de Code: 175
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **gdpr rectification**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``GdprRectificationRequest``
- ``FieldChange``

Énumérations
------------

- ``RectificationStatus``

Fonctions
---------

- ``new()``
- ``approve()``
- ``reject()``
- ``mark_applied()``
- ``is_pending()``

Code Source
===========

Voir: ``backend/src/domain/entities/gdpr_rectification.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


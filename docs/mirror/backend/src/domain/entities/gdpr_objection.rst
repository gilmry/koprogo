=================================
domain/entities/gdpr_objection.rs
=================================

:Fichier: ``backend/src/domain/entities/gdpr_objection.rs``
:Type: RUST
:Lignes de Code: 252
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **gdpr objection**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``GdprObjectionRequest``
- ``ProcessingPurpose``

Énumérations
------------

- ``ObjectionStatus``
- ``ObjectionType``

Fonctions
---------

- ``new()``
- ``accept()``
- ``reject()``
- ``partial_accept()``
- ``is_marketing_objection()``
- ``is_pending()``
- ``get_accepted_purposes()``

Code Source
===========

Voir: ``backend/src/domain/entities/gdpr_objection.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


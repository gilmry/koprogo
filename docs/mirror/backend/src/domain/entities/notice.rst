=========================
domain/entities/notice.rs
=========================

:Fichier: ``backend/src/domain/entities/notice.rs``
:Type: RUST
:Lignes de Code: 915
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **notice**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``Notice``

Énumérations
------------

- ``NoticeType``
- ``NoticeCategory``
- ``NoticeStatus``

Fonctions
---------

- ``new()``
- ``publish()``
- ``archive()``
- ``expire()``
- ``pin()``
- ``unpin()``
- ``is_expired()``
- ``update_content()``
- ``set_expiration()``

Code Source
===========

Voir: ``backend/src/domain/entities/notice.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


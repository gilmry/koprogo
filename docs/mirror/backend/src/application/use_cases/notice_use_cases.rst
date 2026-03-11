=========================================
application/use_cases/notice_use_cases.rs
=========================================

:Fichier: ``backend/src/application/use_cases/notice_use_cases.rs``
:Type: RUST
:Lignes de Code: 476
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **notice**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``NoticeUseCases``
- ``NoticeStatistics``

Fonctions
---------

- ``new()``
- ``create_notice()``
- ``get_notice()``
- ``list_building_notices()``
- ``list_published_notices()``
- ``list_pinned_notices()``
- ``list_notices_by_type()``
- ``list_notices_by_category()``
- ``list_notices_by_status()``
- ``list_author_notices()``
- ``update_notice()``
- ``publish_notice()``
- ``archive_notice()``
- ``pin_notice()``
- ``unpin_notice()``

*... et 4 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/notice_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


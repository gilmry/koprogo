========================================
application/use_cases/quote_use_cases.rs
========================================

:Fichier: ``backend/src/application/use_cases/quote_use_cases.rs``
:Type: RUST
:Lignes de Code: 474
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **quote**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``QuoteUseCases``

Fonctions
---------

- ``new()``
- ``create_quote()``
- ``submit_quote()``
- ``start_review()``
- ``accept_quote()``
- ``reject_quote()``
- ``withdraw_quote()``
- ``compare_quotes()``
- ``get_quote()``
- ``list_by_building()``
- ``list_by_contractor()``
- ``list_by_status()``
- ``list_by_project_title()``
- ``update_contractor_rating()``
- ``mark_expired_quotes()``

*... et 3 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/quote_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


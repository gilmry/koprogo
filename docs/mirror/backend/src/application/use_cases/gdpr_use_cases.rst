=======================================
application/use_cases/gdpr_use_cases.rs
=======================================

:Fichier: ``backend/src/application/use_cases/gdpr_use_cases.rs``
:Type: RUST
:Lignes de Code: 604
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **gdpr**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``GdprUseCases``

Fonctions
---------

- ``new()``
- ``export_user_data()``
- ``erase_user_data()``
- ``can_erase_user()``
- ``rectify_user_data()``
- ``restrict_user_processing()``
- ``unrestrict_user_processing()``
- ``set_marketing_preference()``

Code Source
===========

Voir: ``backend/src/application/use_cases/gdpr_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


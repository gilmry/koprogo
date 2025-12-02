=============================================
application/use_cases/two_factor_use_cases.rs
=============================================

:Fichier: ``backend/src/application/use_cases/two_factor_use_cases.rs``
:Type: RUST
:Lignes de Code: 450
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **two factor**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``TwoFactorUseCases``

Fonctions
---------

- ``new()``
- ``setup_2fa()``
- ``enable_2fa()``
- ``verify_2fa()``
- ``disable_2fa()``
- ``regenerate_backup_codes()``
- ``get_2fa_status()``

Code Source
===========

Voir: ``backend/src/application/use_cases/two_factor_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


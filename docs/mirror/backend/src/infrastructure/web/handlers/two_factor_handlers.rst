==================================================
infrastructure/web/handlers/two_factor_handlers.rs
==================================================

:Fichier: ``backend/src/infrastructure/web/handlers/two_factor_handlers.rs``
:Type: RUST
:Lignes de Code: 430
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **two factor**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``setup_2fa()``
- ``enable_2fa()``
- ``verify_2fa()``
- ``disable_2fa()``
- ``regenerate_backup_codes()``
- ``get_2fa_status()``
- ``configure_routes()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/two_factor_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


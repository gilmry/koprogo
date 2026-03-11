====================================================
infrastructure/web/handlers/organization_handlers.rs
====================================================

:Fichier: ``backend/src/infrastructure/web/handlers/organization_handlers.rs``
:Type: RUST
:Lignes de Code: 451
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **organization**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``OrganizationResponse``
- ``CreateOrganizationRequest``
- ``UpdateOrganizationRequest``

Fonctions
---------

- ``list_organizations()``
- ``create_organization()``
- ``update_organization()``
- ``activate_organization()``
- ``suspend_organization()``
- ``delete_organization()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/organization_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


==================================================
infrastructure/web/handlers/unit_owner_handlers.rs
==================================================

:Fichier: ``backend/src/infrastructure/web/handlers/unit_owner_handlers.rs``
:Type: RUST
:Lignes de Code: 512
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **unit owner**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``add_owner_to_unit()``
- ``remove_owner_from_unit()``
- ``update_unit_owner()``
- ``get_unit_owners()``
- ``get_owner_units()``
- ``get_unit_ownership_history()``
- ``get_owner_ownership_history()``
- ``transfer_ownership()``
- ``get_total_ownership_percentage()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/unit_owner_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


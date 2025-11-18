======================================================
infrastructure/web/handlers/payment_method_handlers.rs
======================================================

:Fichier: ``backend/src/infrastructure/web/handlers/payment_method_handlers.rs``
:Type: RUST
:Lignes de Code: 425
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **payment method**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_payment_method()``
- ``get_payment_method()``
- ``get_payment_method_by_stripe_id()``
- ``list_owner_payment_methods()``
- ``list_active_owner_payment_methods()``
- ``get_default_payment_method()``
- ``list_organization_payment_methods()``
- ``list_payment_methods_by_type()``
- ``update_payment_method()``
- ``set_payment_method_as_default()``
- ``deactivate_payment_method()``
- ``reactivate_payment_method()``
- ``delete_payment_method()``
- ``count_active_payment_methods()``
- ``has_active_payment_methods()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/payment_method_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


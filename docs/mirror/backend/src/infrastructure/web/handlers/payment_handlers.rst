===============================================
infrastructure/web/handlers/payment_handlers.rs
===============================================

:Fichier: ``backend/src/infrastructure/web/handlers/payment_handlers.rs``
:Type: RUST
:Lignes de Code: 579
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **payment**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_payment()``
- ``get_payment()``
- ``get_payment_by_stripe_intent()``
- ``list_owner_payments()``
- ``list_building_payments()``
- ``list_expense_payments()``
- ``list_organization_payments()``
- ``list_payments_by_status()``
- ``list_pending_payments()``
- ``list_failed_payments()``
- ``mark_payment_processing()``
- ``mark_payment_requires_action()``
- ``mark_payment_succeeded()``
- ``mark_payment_failed()``
- ``mark_payment_cancelled()``

*... et 7 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/payment_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


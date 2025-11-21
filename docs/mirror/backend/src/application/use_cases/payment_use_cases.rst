==========================================
application/use_cases/payment_use_cases.rs
==========================================

:Fichier: ``backend/src/application/use_cases/payment_use_cases.rs``
:Type: RUST
:Lignes de Code: 374
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **payment**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``PaymentUseCases``

Fonctions
---------

- ``new()``
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
- ``mark_processing()``
- ``mark_requires_action()``
- ``mark_succeeded()``
- ``mark_failed()``

*... et 11 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/payment_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


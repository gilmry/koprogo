=================================================
application/use_cases/payment_method_use_cases.rs
=================================================

:Fichier: ``backend/src/application/use_cases/payment_method_use_cases.rs``
:Type: RUST
:Lignes de Code: 272
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **payment method**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``PaymentMethodUseCases``

Fonctions
---------

- ``new()``
- ``create_payment_method()``
- ``get_payment_method()``
- ``get_payment_method_by_stripe_id()``
- ``list_owner_payment_methods()``
- ``list_active_owner_payment_methods()``
- ``get_default_payment_method()``
- ``list_organization_payment_methods()``
- ``list_payment_methods_by_type()``
- ``update_payment_method()``
- ``set_as_default()``
- ``deactivate_payment_method()``
- ``reactivate_payment_method()``
- ``delete_payment_method()``
- ``count_active_payment_methods()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/payment_method_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


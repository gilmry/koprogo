==========================
domain/entities/payment.rs
==========================

:Fichier: ``backend/src/domain/entities/payment.rs``
:Type: RUST
:Lignes de Code: 519
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **transaction de paiement**. Intégration Stripe avec gestion du lifecycle (Pending → Processing → Succeeded/Failed) et support remboursements.

API Publique
============

Structures
----------

- ``Payment``

Énumérations
------------

- ``TransactionStatus``
- ``PaymentMethodType``

Fonctions
---------

- ``new()``
- ``mark_processing()``
- ``mark_requires_action()``
- ``mark_succeeded()``
- ``mark_failed()``
- ``mark_cancelled()``
- ``refund()``
- ``set_stripe_payment_intent_id()``
- ``set_stripe_customer_id()``
- ``set_payment_method_id()``
- ``set_metadata()``
- ``get_net_amount_cents()``
- ``is_final()``
- ``can_refund()``

Code Source
===========

Voir: ``backend/src/domain/entities/payment.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


=================================
domain/entities/payment_method.rs
=================================

:Fichier: ``backend/src/domain/entities/payment_method.rs``
:Type: RUST
:Lignes de Code: 322
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **moyen de paiement** (carte bancaire, SEPA, virement). Stockage sécurisé avec tokens Stripe (PCI-DSS compliant).

API Publique
============

Structures
----------

- ``PaymentMethod``

Énumérations
------------

- ``PaymentMethodType``

Fonctions
---------

- ``new()``
- ``set_default()``
- ``unset_default()``
- ``deactivate()``
- ``reactivate()``
- ``set_metadata()``
- ``set_expiry()``
- ``is_expired()``
- ``is_usable()``

Code Source
===========

Voir: ``backend/src/domain/entities/payment_method.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


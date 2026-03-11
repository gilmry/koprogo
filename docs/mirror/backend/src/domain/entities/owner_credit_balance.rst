=======================================
domain/entities/owner_credit_balance.rs
=======================================

:Fichier: ``backend/src/domain/entities/owner_credit_balance.rs``
:Type: RUST
:Lignes de Code: 329
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **copropriétaire**. Contient les données personnelles (GDPR) et coordonnées du propriétaire.

API Publique
============

Structures
----------

- ``OwnerCreditBalance``

Énumérations
------------

- ``CreditStatus``
- ``ParticipationLevel``

Fonctions
---------

- ``new()``
- ``earn_credits()``
- ``spend_credits()``
- ``increment_exchanges()``
- ``update_rating()``
- ``has_sufficient_credits()``
- ``credit_status()``
- ``is_new_member()``
- ``participation_level()``

Code Source
===========

Voir: ``backend/src/domain/entities/owner_credit_balance.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


=====================================
domain/entities/owner_contribution.rs
=====================================

:Fichier: ``backend/src/domain/entities/owner_contribution.rs``
:Type: RUST
:Lignes de Code: 274
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **copropriétaire**. Contient les données personnelles (GDPR) et coordonnées du propriétaire.

API Publique
============

Structures
----------

- ``OwnerContribution``

Énumérations
------------

- ``ContributionType``
- ``ContributionPaymentStatus``
- ``ContributionPaymentMethod``

Fonctions
---------

- ``new()``
- ``mark_as_paid()``
- ``is_paid()``
- ``is_overdue()``

Code Source
===========

Voir: ``backend/src/domain/entities/owner_contribution.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


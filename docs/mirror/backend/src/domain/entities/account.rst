==========================
domain/entities/account.rs
==========================

:Fichier: ``backend/src/domain/entities/account.rs``
:Type: RUST
:Lignes de Code: 537
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **compte comptable PCMN**. Implémentation complète du Plan Comptable Minimum Normalisé belge (AR 12/07/2012) avec hiérarchie 8 classes.

API Publique
============

Structures
----------

- ``Account``

Énumérations
------------

- ``AccountType``

Fonctions
---------

- ``from_code()``
- ``is_balance_sheet()``
- ``is_income_statement()``
- ``new()``
- ``get_class()``
- ``is_root()``
- ``update()``

Code Source
===========

Voir: ``backend/src/domain/entities/account.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


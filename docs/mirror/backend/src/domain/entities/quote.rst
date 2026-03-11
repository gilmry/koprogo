========================
domain/entities/quote.rs
========================

:Fichier: ``backend/src/domain/entities/quote.rs``
:Type: RUST
:Lignes de Code: 595
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **devis entrepreneur**. Conformité légale belge (3 devis obligatoires >5000€) avec scoring automatique (prix, délai, garantie, réputation).

API Publique
============

Structures
----------

- ``Quote``
- ``QuoteScore``

Énumérations
------------

- ``QuoteStatus``

Fonctions
---------

- ``to_sql()``
- ``from_sql()``
- ``new()``
- ``submit()``
- ``start_review()``
- ``accept()``
- ``reject()``
- ``withdraw()``
- ``is_expired()``
- ``mark_expired()``
- ``set_contractor_rating()``
- ``calculate_score()``

Code Source
===========

Voir: ``backend/src/domain/entities/quote.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


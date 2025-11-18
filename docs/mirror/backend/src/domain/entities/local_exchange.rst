=================================
domain/entities/local_exchange.rs
=================================

:Fichier: ``backend/src/domain/entities/local_exchange.rs``
:Type: RUST
:Lignes de Code: 579
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **échange SEL** (Système d'Échange Local). Currency temps (1h = 1 crédit) avec workflow complet (Offered → Requested → InProgress → Completed).

API Publique
============

Structures
----------

- ``LocalExchange``

Énumérations
------------

- ``ExchangeType``
- ``ExchangeStatus``

Fonctions
---------

- ``to_sql()``
- ``from_sql()``
- ``to_sql()``
- ``from_sql()``
- ``new()``
- ``request()``
- ``start()``
- ``complete()``
- ``cancel()``
- ``rate_provider()``
- ``rate_requester()``
- ``is_active()``
- ``has_mutual_ratings()``

Code Source
===========

Voir: ``backend/src/domain/entities/local_exchange.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


=================================================
application/use_cases/local_exchange_use_cases.rs
=================================================

:Fichier: ``backend/src/application/use_cases/local_exchange_use_cases.rs``
:Type: RUST
:Lignes de Code: 608
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **local exchange**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``LocalExchangeUseCases``

Fonctions
---------

- ``new()``
- ``create_exchange()``
- ``get_exchange()``
- ``list_building_exchanges()``
- ``list_available_exchanges()``
- ``list_owner_exchanges()``
- ``list_exchanges_by_type()``
- ``request_exchange()``
- ``start_exchange()``
- ``complete_exchange()``
- ``cancel_exchange()``
- ``rate_provider()``
- ``rate_requester()``
- ``delete_exchange()``
- ``get_credit_balance()``

*... et 3 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/local_exchange_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


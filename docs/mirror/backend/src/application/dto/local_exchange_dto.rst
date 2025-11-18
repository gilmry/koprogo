=====================================
application/dto/local_exchange_dto.rs
=====================================

:Fichier: ``backend/src/application/dto/local_exchange_dto.rs``
:Type: RUST
:Lignes de Code: 157
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **local exchange**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``CreateLocalExchangeDto``
- ``RequestExchangeDto``
- ``CompleteExchangeDto``
- ``CancelExchangeDto``
- ``RateExchangeDto``
- ``LocalExchangeResponseDto``
- ``OwnerCreditBalanceDto``
- ``SelStatisticsDto``
- ``OwnerExchangeSummaryDto``

Fonctions
---------

- ``from_entity()``
- ``from_entity()``

Code Source
===========

Voir: ``backend/src/application/dto/local_exchange_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


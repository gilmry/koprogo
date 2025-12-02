=====================================
application/ports/linky_api_client.rs
=====================================

:Fichier: ``backend/src/application/ports/linky_api_client.rs``
:Type: RUST
:Lignes de Code: 121
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Port (trait) définissant l'interface **linky api client**. Abstraction pour l'inversion de dépendance (Hexagonal Architecture), implémentée par la couche Infrastructure.

API Publique
============

Structures
----------

- ``OAuth2TokenResponse``
- ``ConsumptionDataPoint``
- ``PowerDataPoint``

Énumérations
------------

- ``LinkyApiError``

Traits
------

- ``LinkyApiClient``

Code Source
===========

Voir: ``backend/src/application/ports/linky_api_client.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


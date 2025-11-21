===========================
application/dto/auth_dto.rs
===========================

:Fichier: ``backend/src/application/dto/auth_dto.rs``
:Type: RUST
:Lignes de Code: 80
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **auth**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``LoginRequest``
- ``RegisterRequest``
- ``LoginResponse``
- ``RefreshTokenRequest``
- ``UserRoleSummary``
- ``UserResponse``
- ``SwitchRoleRequest``
- ``Claims``

Code Source
===========

Voir: ``backend/src/application/dto/auth_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


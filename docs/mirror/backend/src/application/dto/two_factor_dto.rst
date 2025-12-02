=================================
application/dto/two_factor_dto.rs
=================================

:Fichier: ``backend/src/application/dto/two_factor_dto.rs``
:Type: RUST
:Lignes de Code: 142
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **two factor**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``Setup2FAResponseDto``
- ``Enable2FADto``
- ``Enable2FAResponseDto``
- ``Verify2FADto``
- ``Verify2FAResponseDto``
- ``Disable2FADto``
- ``RegenerateBackupCodesDto``
- ``RegenerateBackupCodesResponseDto``
- ``TwoFactorStatusDto``

Code Source
===========

Voir: ``backend/src/application/dto/two_factor_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


================================
infrastructure/web/middleware.rs
================================

:Fichier: ``backend/src/infrastructure/web/middleware.rs``
:Type: RUST
:Lignes de Code: 398
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Module Rust **middleware**. Fait partie de la couche Infrastructure (Adaptateurs).

API Publique
============

Structures
----------

- ``AuthenticatedUser``
- ``OrganizationId``
- ``GdprRateLimitConfig``
- ``GdprRateLimitState``
- ``GdprRateLimit``
- ``GdprRateLimitMiddleware``

Fonctions
---------

- ``require_organization()``
- ``new()``
- ``check_rate_limit()``
- ``new()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/middleware.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


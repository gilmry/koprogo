========================================
infrastructure/web/login_rate_limiter.rs
========================================

:Fichier: ``backend/src/infrastructure/web/login_rate_limiter.rs``
:Type: RUST
:Lignes de Code: 279
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Module Rust **login rate limiter**. Fait partie de la couche Infrastructure (Adaptateurs).

API Publique
============

Structures
----------

- ``LoginRateLimiter``
- ``LoginRateLimiterMiddleware``

Fonctions
---------

- ``new()``
- ``check_rate_limit()``
- ``get_attempt_count()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/login_rate_limiter.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`


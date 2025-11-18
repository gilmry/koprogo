==========================================
Backend Source Code Documentation (Mirror)
==========================================

This documentation mirrors the entire backend source code structure,
providing detailed documentation for each file.

**Total Files**: 321 Rust source files

**Architecture**: Hexagonal (Ports & Adapters) + Domain-Driven Design (DDD)

Directory Structure
===================

.. toctree::
   :maxdepth: 2
   :caption: Source Code Mirror

   src/index

Layers
======

The backend follows a strict 3-layer architecture:

1. **Domain Layer** (``domain/``)
   - Pure business logic
   - No external dependencies
   - Entities with invariant validation
   - Domain services

2. **Application Layer** (``application/``)
   - Use cases (orchestration logic)
   - Ports (trait definitions)
   - DTOs (data transfer objects)
   - Services (application-level orchestration)

3. **Infrastructure Layer** (``infrastructure/``)
   - Database repositories (PostgreSQL)
   - Web handlers (Actix-web)
   - External integrations (Stripe, Linky, etc.)
   - Storage adapters

Quick Links
===========

- :doc:`/ARCHITECTURE` - Architecture documentation
- :doc:`/CLAUDE` - Project overview and commands
- :doc:`/NOUVELLES_FONCTIONNALITES_2025` - 2025 features documentation


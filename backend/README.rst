=================
KoproGo Backend
=================

:Stack: Rust + Actix-web 4.9 + PostgreSQL 15
:Architecture: Hexagonal (Ports & Adapters) + DDD

Structure du projet
===================

.. code-block:: text

   backend/
   |- src/
   |  |- domain/           # Logique metier pure (zero dependance externe)
   |  |  |- entities/      # Aggregats avec validation des invariants
   |  |  '- services/      # Services domaine
   |  |- application/      # Cas d'usage + interfaces
   |  |  |- dto/           # Data Transfer Objects (contrats API)
   |  |  |- ports/         # Traits (interfaces repository)
   |  |  '- use_cases/     # Orchestration logique metier
   |  '- infrastructure/   # Implementations concretes
   |     |- database/
   |     |  '- repositories/  # Implementations PostgreSQL des ports
   |     '- web/
   |        |- handlers/   # Handlers HTTP Actix-web
   |        '- routes.rs   # Configuration des routes API
   |- migrations/          # Migrations SQL (sqlx)
   |- tests/
   |  |- features/         # Scenarios BDD Gherkin (.feature)
   |  |- bdd*.rs           # Step definitions Cucumber
   |  |- integration/      # Tests integration (testcontainers)
   |  '- e2e/              # Tests E2E API
   '- benches/             # Benchmarks Criterion

Regles d'architecture
=====================

1. **Domain** : logique metier pure, aucune dependance externe
2. **Application** : definit les ports (traits), orchestre les use cases
3. **Infrastructure** : implemente les ports, handlers HTTP

La dependance va toujours de l'exterieur vers l'interieur :
``Infrastructure -> Application -> Domain``

Demarrage rapide
=================

.. code-block:: bash

   # Prerequis : Docker, Rust toolchain
   cp .env.example .env

   # Demarrer PostgreSQL
   make docker-up

   # Executer les migrations
   make migrate

   # Lancer le serveur (localhost:8080)
   cargo run

   # Ou avec auto-reload
   make dev

Tests
=====

.. code-block:: bash

   # Tests unitaires (domaine)
   cargo test --lib

   # Tests BDD (Cucumber/Gherkin)
   cargo test --test bdd
   cargo test --test bdd_governance
   cargo test --test bdd_community
   cargo test --test bdd_operations
   cargo test --test bdd_financial

   # Tests integration (testcontainers)
   cargo test --test integration

   # Tests E2E
   cargo test --test e2e

   # Benchmarks
   cargo bench

   # Couverture
   make coverage

Dependances principales
=======================

- ``actix-web`` 4.9 : Framework web
- ``sqlx`` 0.8 : Acces base avec verification compile-time
- ``tokio`` 1.41 : Runtime async
- ``uuid``, ``chrono`` : Types de donnees
- ``serde``, ``serde_json`` : Serialisation
- ``cucumber`` 0.21 : Tests BDD
- ``testcontainers`` 0.23 : Tests integration
- ``criterion`` 0.5 : Benchmarks

Cibles de performance
=====================

- Latence P99 : < 5ms
- Debit : > 100k req/s
- Memoire : < 128MB par instance
- Pool connexions : max 10 PostgreSQL

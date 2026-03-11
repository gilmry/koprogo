Documentation Backend
=====================

Vue d'ensemble du backend KoproGo, développé en Rust avec architecture hexagonale (Ports & Adapters) et Domain-Driven Design (DDD).

**Stack Technique** :

- **Langage** : Rust 1.83
- **Framework Web** : Actix-web 4.9
- **Base de Données** : PostgreSQL 15
- **ORM** : SQLx 0.8 (compile-time verified queries)
- **Runtime Async** : Tokio 1.41
- **Tests** : Cucumber (BDD), Criterion (benchmarks), Testcontainers

**Architecture** :

.. code-block:: text

   Domain (Core Métier)
     ↑ définit interfaces
   Application (Use Cases + Ports)
     ↑ implémente ports
   Infrastructure (Adapters: Web, Database)

Principes Architecturaux
-------------------------

Architecture Hexagonale (Ports & Adapters)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Le backend suit une stricte séparation en 3 couches :

.. code-block:: text

   ┌──────────────────────────────────────────────────┐
   │              DOMAIN (Core)                       │
   │  ┌────────────┐         ┌────────────┐          │
   │  │  Entities  │         │  Services  │          │
   │  └────────────┘         └────────────┘          │
   │    (Building, User, Expense, Owner...)           │
   └──────────────────────────────────────────────────┘
                      ↑ NO dependencies
   ┌──────────────────────────────────────────────────┐
   │          APPLICATION (Use Cases)                 │
   │  ┌────────────┐  ┌────────────┐  ┌────────────┐ │
   │  │    DTOs    │  │   Ports    │  │ Use Cases  │ │
   │  └────────────┘  └────────────┘  └────────────┘ │
   │      (API contracts) (Traits)  (Business Logic)  │
   └──────────────────────────────────────────────────┘
                      ↑ depends on Domain
   ┌──────────────────────────────────────────────────┐
   │       INFRASTRUCTURE (Adapters)                  │
   │  ┌────────────┐              ┌────────────┐     │
   │  │  Database  │              │    Web     │     │
   │  │(PostgreSQL)│              │ (Actix-web)│     │
   │  └────────────┘              └────────────┘     │
   │   (Repositories impl)       (HTTP handlers)     │
   └──────────────────────────────────────────────────┘
                      ↑ depends on Application

**Règles Strictes** :

- ❌ **Domain** ne dépend de PERSONNE (pure business logic)
- ❌ **Application** dépend UNIQUEMENT de Domain
- ✅ **Infrastructure** implémente les ports définis par Application

Domain-Driven Design (DDD)
^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Concepts DDD Implémentés** :

- **Entities** : Objets métier avec identité (Building, Owner, Unit)
- **Value Objects** : UUID, Timestamps, Email (à venir)
- **Aggregates** : Building (aggregate root), Units (entités liées)
- **Domain Services** : ExpenseCalculator, PCNExporter
- **Repositories** : Abstraction accès données (traits)
- **Use Cases** : Logique métier orchestrée

Organisation des Modules
-------------------------

.. toctree::
   :maxdepth: 2

   src/lib
   src/config
   src/domain/index
   src/application/index
   src/infrastructure/index
   tests/index
   benches/index

Structure Complète
------------------

.. code-block:: text

   backend/src/
   ├── lib.rs                          # Point d'entrée bibliothèque
   ├── main.rs                         # Point d'entrée serveur
   ├── config.rs                       # Configuration runtime
   ├── db.rs                           # Utilitaires DB
   │
   ├── domain/                         # Couche Domaine (Core)
   │   ├── entities/                   # Entités métier
   │   │   ├── user.rs                 # Utilisateurs (auth)
   │   │   ├── organization.rs         # Organisations (multi-tenant)
   │   │   ├── building.rs             # Immeubles
   │   │   ├── unit.rs                 # Lots
   │   │   ├── owner.rs                # Copropriétaires
   │   │   ├── expense.rs              # Charges
   │   │   ├── meeting.rs              # Assemblées générales
   │   │   ├── document.rs             # Documents
   │   │   └── refresh_token.rs        # Tokens JWT refresh
   │   └── services/                   # Services domaine
   │       ├── expense_calculator.rs   # Calculs de répartition charges
   │       ├── pcn_mapper.rs           # Mapping PCN (Précompte)
   │       └── pcn_exporter.rs         # Export PDF/Excel PCN
   │
   ├── application/                    # Couche Application (Use Cases)
   │   ├── dto/                        # Data Transfer Objects
   │   │   ├── auth_dto.rs             # DTOs authentification
   │   │   ├── building_dto.rs         # DTOs immeubles
   │   │   ├── unit_dto.rs             # DTOs lots
   │   │   ├── owner_dto.rs            # DTOs copropriétaires
   │   │   ├── expense_dto.rs          # DTOs charges
   │   │   ├── meeting_dto.rs          # DTOs assemblées
   │   │   ├── document_dto.rs         # DTOs documents
   │   │   ├── pcn_dto.rs              # DTOs PCN
   │   │   └── pagination.rs           # Pagination générique
   │   ├── ports/                      # Interfaces (Traits)
   │   │   ├── user_repository.rs
   │   │   ├── organization_repository.rs
   │   │   ├── building_repository.rs
   │   │   ├── unit_repository.rs
   │   │   ├── owner_repository.rs
   │   │   ├── expense_repository.rs
   │   │   ├── meeting_repository.rs
   │   │   └── document_repository.rs
   │   └── use_cases/                  # Logique métier
   │       ├── auth_use_cases.rs       # Authentification JWT
   │       ├── building_use_cases.rs   # CRUD immeubles
   │       ├── unit_use_cases.rs       # CRUD lots
   │       ├── owner_use_cases.rs      # CRUD copropriétaires
   │       └── expense_use_cases.rs    # CRUD charges
   │
   └── infrastructure/                 # Couche Infrastructure (Adapters)
       ├── database/                   # Adapter PostgreSQL
       │   ├── pool.rs                 # Pool connexions SQLx
       │   ├── seed.rs                 # Seed données test
       │   └── repositories/           # Implémentations repositories
       │       ├── user_repository_impl.rs
       │       ├── organization_repository_impl.rs
       │       ├── building_repository_impl.rs
       │       ├── unit_repository_impl.rs
       │       ├── owner_repository_impl.rs
       │       ├── expense_repository_impl.rs
       │       ├── meeting_repository_impl.rs
       │       └── document_repository_impl.rs
       └── web/                        # Adapter HTTP (Actix-web)
           ├── app_state.rs            # État partagé app
           ├── routes.rs               # Configuration routes
           ├── middleware/             # Middleware HTTP
           │   ├── auth.rs             # Auth JWT middleware
           │   └── cors.rs             # CORS middleware
           └── handlers/               # Handlers HTTP
               ├── auth_handlers.rs    # POST /auth/login, /auth/refresh
               ├── seed_handlers.rs    # POST /seed (dev only)
               ├── building_handlers.rs # CRUD /buildings
               ├── unit_handlers.rs    # CRUD /units
               ├── owner_handlers.rs   # CRUD /owners
               ├── expense_handlers.rs # CRUD /expenses
               ├── meeting_handlers.rs # CRUD /meetings
               ├── document_handlers.rs # CRUD /documents
               └── health.rs           # GET /health

Commandes Développement
------------------------

**Setup Environnement** :

.. code-block:: bash

   # Copier .env
   cp backend/.env.example backend/.env

   # Démarrer PostgreSQL
   make docker-up

   # Run migrations
   cd backend && sqlx migrate run

**Développement** :

.. code-block:: bash

   # Dev server avec auto-reload
   make dev                    # backend uniquement
   make dev-all                # backend + frontend + postgres

   # Build release
   cargo build --release

   # Format code
   cargo fmt

   # Lint
   cargo clippy -- -D warnings

**Tests** :

.. code-block:: bash

   # Tests unitaires (domain logic)
   cargo test --lib

   # Tests d'intégration (avec testcontainers)
   cargo test --test integration

   # Tests BDD (Cucumber/Gherkin)
   cargo test --test bdd

   # Tests E2E (full API)
   cargo test --test e2e

   # Benchmarks (Criterion)
   cargo bench

   # Coverage (tarpaulin)
   make coverage

   # Tous les tests
   make test

**Base de Données** :

.. code-block:: bash

   # Créer migration
   cd backend && sqlx migrate add <nom>

   # Appliquer migrations
   sqlx migrate run

   # Vérifier queries compile-time
   cargo sqlx prepare

   # Seed données test
   cargo run --bin seed

**Sécurité** :

.. code-block:: bash

   # Audit dépendances
   cargo audit
   make audit

Endpoints API
-------------

Base URL : ``http://localhost:8080/api/v1``

Authentification
^^^^^^^^^^^^^^^^

.. code-block:: text

   POST   /auth/login              # Login JWT (email + password)
   POST   /auth/refresh            # Refresh token
   GET    /auth/me                 # Profile utilisateur courant

Buildings (Immeubles)
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   GET    /buildings               # Liste paginée (JWT required)
   POST   /buildings               # Créer (JWT required, role: syndic)
   GET    /buildings/:id           # Détails immeuble
   PUT    /buildings/:id           # Modifier
   DELETE /buildings/:id           # Supprimer

Units (Lots)
^^^^^^^^^^^^

.. code-block:: text

   GET    /units                   # Liste paginée
   POST   /units                   # Créer lot
   GET    /units/:id               # Détails lot
   PUT    /units/:id               # Modifier
   DELETE /units/:id               # Supprimer
   GET    /buildings/:id/units     # Lots d'un immeuble
   PUT    /units/:id/assign-owner/:owner_id  # Assigner copropriétaire

Owners (Copropriétaires)
^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   GET    /owners                  # Liste paginée
   POST   /owners                  # Créer copropriétaire
   GET    /owners/:id              # Détails copropriétaire
   PUT    /owners/:id              # Modifier
   DELETE /owners/:id              # Supprimer

Expenses (Charges)
^^^^^^^^^^^^^^^^^^

.. code-block:: text

   GET    /expenses                # Liste paginée
   POST   /expenses                # Créer charge
   GET    /expenses/:id            # Détails charge
   PUT    /expenses/:id            # Modifier
   DELETE /expenses/:id            # Supprimer
   GET    /buildings/:id/expenses  # Charges d'un immeuble
   PUT    /expenses/:id/mark-paid  # Marquer comme payé

Meetings (Assemblées Générales)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   GET    /meetings                # Liste paginée
   POST   /meetings                # Créer assemblée
   GET    /meetings/:id            # Détails assemblée
   PUT    /meetings/:id            # Modifier
   DELETE /meetings/:id            # Supprimer

Documents
^^^^^^^^^

.. code-block:: text

   GET    /documents               # Liste paginée
   POST   /documents/upload        # Upload document
   GET    /documents/:id           # Télécharger document
   DELETE /documents/:id           # Supprimer document

PCN (Précompte de Charge Notariale)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   GET    /pcn/:building_id        # Calcul PCN immeuble
   GET    /pcn/export/pdf/:id      # Export PDF
   GET    /pcn/export/excel/:id    # Export Excel

Health & Seed
^^^^^^^^^^^^^

.. code-block:: text

   GET    /health                  # Health check
   POST   /seed                    # Seed données test (dev only)

Performance
-----------

**Objectifs de Performance** :

- **Latency P99** : < 5ms
- **Throughput** : > 100,000 req/s
- **Memory** : < 128MB par instance
- **Connection Pool** : Max 10 connexions PostgreSQL

**Optimisations** :

.. code-block:: toml

   # Cargo.toml
   [profile.release]
   opt-level = 3           # Optimisation maximale
   lto = true              # Link-Time Optimization
   codegen-units = 1       # Codegen units minimal

**Benchmarks** :

.. code-block:: bash

   cargo bench

   # Résultats typiques:
   # - Building CRUD: ~1-2ms P99
   # - Query pagination: ~3-4ms P99
   # - JWT validation: <0.1ms

Tests
-----

**Stratégie de Tests (Pyramid)** :

.. code-block:: text

                 ▲
                / \
               /E2E\         < 10% (tests end-to-end)
              /_____\
             /       \
            /  BDD    \      < 20% (tests comportementaux)
           /__________\
          /            \
         / Integration  \   < 30% (tests avec DB réelle)
        /________________\
       /                  \
      /    Unit Tests      \  < 40% (tests logique pure)
     /______________________\

**Coverage Cible** : 80%+ (domaine: 100%, application: 80%, infrastructure: 60%)

Sécurité
--------

**JWT Authentication** :

- Tokens signés HS256 (secret 256-bit)
- Expiration: 1h (access token), 7 jours (refresh token)
- Claims: user_id, role, organization_id

**GDPR Compliance** :

- Chiffrement données sensibles (email, téléphone) - à implémenter
- Droit à l'effacement (endpoint /users/:id/gdpr-delete)
- Logs anonymisés

**SQL Injection Prevention** :

- SQLx parameterized queries (compile-time verified)
- Pas de concaténation SQL manuelle

**CORS** :

.. code-block:: rust

   Cors::default()
       .allowed_origin("https://koprogo.com")
       .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
       .allowed_headers(vec![AUTHORIZATION, CONTENT_TYPE, ACCEPT_LANGUAGE])

Multi-Tenancy
-------------

Isolation données via ``organization_id`` :

.. code-block:: sql

   -- Toutes les requêtes filtrent par organization_id
   SELECT * FROM buildings
   WHERE organization_id = $1;  -- Extrait du JWT

**Row-Level Security (RLS)** PostgreSQL - à implémenter.

Migrations Base de Données
---------------------------

**Localisation** : ``backend/migrations/``

.. code-block:: bash

   # Créer migration
   sqlx migrate add create_buildings_table

   # Appliquer
   sqlx migrate run

   # Rollback (manuel, éditer migration)
   sqlx migrate revert

**Exemple Migration** :

.. code-block:: sql

   -- migrations/YYYYMMDDHHMMSS_create_buildings_table.sql
   CREATE TABLE buildings (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name VARCHAR(255) NOT NULL CHECK (length(name) > 0),
       address VARCHAR(255) NOT NULL,
       city VARCHAR(100) NOT NULL,
       postal_code VARCHAR(20) NOT NULL,
       country VARCHAR(100) NOT NULL,
       total_units INTEGER NOT NULL CHECK (total_units > 0),
       construction_year INTEGER,
       organization_id UUID REFERENCES organizations(id),
       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );

   CREATE INDEX idx_buildings_org ON buildings(organization_id);
   CREATE INDEX idx_buildings_city ON buildings(city);

Déploiement
-----------

**Production Build** :

.. code-block:: bash

   cargo build --release
   ./target/release/koprogo_api

**Docker** :

.. code-block:: dockerfile

   FROM rust:1.83 as builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release

   FROM debian:bookworm-slim
   COPY --from=builder /app/target/release/koprogo_api /usr/local/bin/
   CMD ["koprogo_api"]

**Variables d'Environnement** :

.. code-block:: bash

   DATABASE_URL=postgresql://user:pass@localhost:5432/koprogo_db
   SERVER_HOST=0.0.0.0
   SERVER_PORT=8080
   JWT_SECRET=<256-bit-secret>
   RUST_LOG=info

Extensions Futures
------------------

1. **WebSocket** : Notifications temps réel
2. **GraphQL** : Alternative REST (via Juniper)
3. **gRPC** : Communication inter-services
4. **Event Sourcing** : Audit trail complet
5. **CQRS** : Séparation lectures/écritures
6. **ScyllaDB** : Cache distribué
7. **DragonflyDB** : Cache Redis-compatible
8. **Kubernetes** : Orchestration containers

Références
----------

- CLAUDE.md : Guide développement complet
- Architecture Hexagonale : Alistair Cockburn
- Domain-Driven Design : Eric Evans
- Clean Architecture : Robert C. Martin
- Rust Book : https://doc.rust-lang.org/book/
- Actix-web Docs : https://actix.rs/docs/
- SQLx Docs : https://docs.rs/sqlx/

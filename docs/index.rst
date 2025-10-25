===================================
Documentation Technique Koprogo
===================================

.. contents:: Table des matières
   :depth: 3
   :local:

Introduction
============

Koprogo est une plateforme SaaS de gestion de copropriété développée avec une stack moderne:

- **Backend**: Rust avec Actix-web et PostgreSQL
- **Frontend**: Astro + Svelte en mode PWA
- **Architecture**: Hexagonale (Ports & Adapters)
- **Multi-tenancy**: Support organisationnel complet
- **Authentification**: JWT avec rôles hiérarchiques

Vue d'ensemble du projet
========================

Structure générale
------------------

.. code-block:: text

    koprogo/
    ├── backend/           # API Rust/Actix-web
    │   ├── src/
    │   │   ├── main.rs           # Point d'entrée serveur
    │   │   ├── lib.rs            # Modules publics
    │   │   ├── domain/           # Entités et logique métier
    │   │   ├── application/      # Use cases et DTOs
    │   │   └── infrastructure/   # Adapteurs (DB, Web)
    │   ├── migrations/    # Migrations SQL
    │   ├── tests/         # Tests BDD et E2E
    │   └── benches/       # Tests de charge
    │
    ├── frontend/          # Application Astro/Svelte
    │   ├── src/
    │   │   ├── pages/           # Pages Astro (routes)
    │   │   ├── components/      # Composants Svelte
    │   │   ├── lib/             # Utilitaires et stores
    │   │   └── layouts/         # Layouts Astro
    │   └── tests/e2e/    # Tests E2E Playwright
    │
    ├── docs/             # Documentation (ce dossier)
    ├── docker-compose.yml
    └── Makefile

Stack technique
---------------

Backend
~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Technologie
     - Utilisation
   * - **Rust** (edition 2021, nightly)
     - Langage backend avec performance et sécurité
   * - **Actix-web** 4.11
     - Framework web asynchrone haute performance
   * - **SQLx** 0.8.6
     - Client PostgreSQL async avec migrations et macros
   * - **PostgreSQL** 15
     - Base de données relationnelle
   * - **bcrypt** 0.15
     - Hachage de mots de passe (cost 12)
   * - **jsonwebtoken** 9.3
     - Authentification JWT
   * - **uuid** 1.11
     - Identifiants uniques (v4)
   * - **chrono** 0.4
     - Gestion des dates/timestamps
   * - **validator** 0.18
     - Validation déclarative
   * - **serde** 1.0
     - Sérialisation/désérialisation JSON

Frontend
~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Technologie
     - Utilisation
   * - **Astro** 5.x
     - Framework SSR/SSG pour pages et routing
   * - **Svelte** 5.x
     - Composants interactifs réactifs
   * - **TypeScript** 5.x
     - Typage statique
   * - **Vite** 6.x
     - Build tool et dev server
   * - **@vite-pwa/astro**
     - Support Progressive Web App
   * - **Workbox**
     - Service Worker et stratégies de cache
   * - **IndexedDB**
     - Base de données locale pour mode offline
   * - **Playwright**
     - Tests E2E avec vidéos

DevOps
~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Outil
     - Utilisation
   * - **Docker** / **docker-compose**
     - Conteneurisation et orchestration
   * - **GitHub Actions**
     - CI/CD avec workflows automatisés
   * - **Make**
     - Commandes de développement
   * - **SQLx CLI**
     - Gestion des migrations et query cache

Architecture Hexagonale
========================

Principes
---------

L'application backend suit l'architecture hexagonale (Ports & Adapters):

.. code-block:: text

    ┌─────────────────────────────────────────────────────────────┐
    │                    INFRASTRUCTURE                           │
    │                                                             │
    │   ┌──────────────┐                      ┌──────────────┐   │
    │   │     Web      │                      │   Database   │   │
    │   │  (Actix-web) │                      │  (PostgreSQL)│   │
    │   │   Handlers   │                      │ Repositories │   │
    │   └──────┬───────┘                      └───────┬──────┘   │
    │          │                                      │          │
    └──────────┼──────────────────────────────────────┼──────────┘
               │                                      │
               ▼                                      ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                    APPLICATION                              │
    │                                                             │
    │   ┌──────────────┐        ┌──────────────┐                 │
    │   │  Use Cases   │◄───────┤     DTOs     │                 │
    │   │              │        │              │                 │
    │   │ • AuthUseCases        │ • LoginRequest                 │
    │   │ • BuildingUseCases    │ • BuildingDto                  │
    │   │ • ...                 │ • ...                          │
    │   └──────┬───────┘        └──────────────┘                 │
    │          │                                                  │
    │          │ utilise                                          │
    │          ▼                                                  │
    └─────────────────────────────────────────────────────────────┘
               │
               ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                       DOMAIN                                │
    │                                                             │
    │   ┌──────────────┐        ┌──────────────┐                 │
    │   │   Entities   │        │   Services   │                 │
    │   │              │        │              │                 │
    │   │ • User               │ • ExpenseCalculator             │
    │   │ • Building           │                                 │
    │   │ • Unit               │                                 │
    │   │ • Owner              │                                 │
    │   │ • Expense            │                                 │
    │   └──────────────┘        └──────────────┘                 │
    │                                                             │
    │   📌 Cœur métier - Indépendant des frameworks              │
    └─────────────────────────────────────────────────────────────┘

Flux de dépendances
-------------------

.. code-block:: text

    Infrastructure ──depends on──> Application ──depends on──> Domain
         (Web)                       (Use Cases)              (Entities)
         (DB)

    ✅ Domain ne dépend de personne (pur métier)
    ✅ Application ne dépend que de Domain
    ✅ Infrastructure dépend de Application et Domain

Avantages
---------

1. **Testabilité**: Chaque couche testable indépendamment
2. **Maintenabilité**: Séparation claire des responsabilités
3. **Évolutivité**: Changement de framework/DB sans toucher au métier
4. **Business-centric**: La logique métier est au centre

Documentation Backend
=====================

Point d'entrée
--------------

.. toctree::
   :maxdepth: 1

   backend/src/main
   backend/src/lib
   backend/src/config

Couche Domaine
--------------

Entités métier
~~~~~~~~~~~~~~

.. toctree::
   :maxdepth: 1

   backend/src/domain/entities/user
   backend/src/domain/entities/building
   backend/src/domain/entities/unit (à documenter)
   backend/src/domain/entities/owner (à documenter)
   backend/src/domain/entities/expense (à documenter)
   backend/src/domain/entities/meeting (à documenter)
   backend/src/domain/entities/document (à documenter)
   backend/src/domain/entities/organization (à documenter)

Services de domaine
~~~~~~~~~~~~~~~~~~~

.. toctree::
   :maxdepth: 1

   backend/src/domain/services/expense_calculator (à documenter)

Couche Application
------------------

Use Cases
~~~~~~~~~

Les use cases orchestrent la logique métier:

- ``auth_use_cases.rs`` - Authentification, login, register
- ``building_use_cases.rs`` - CRUD immeubles
- ``unit_use_cases.rs`` - CRUD lots
- ``owner_use_cases.rs`` - CRUD propriétaires
- ``expense_use_cases.rs`` - CRUD charges

DTOs
~~~~

Data Transfer Objects pour les échanges API:

- ``auth_dto.rs`` - LoginRequest, RegisterRequest, LoginResponse, Claims
- ``building_dto.rs`` - BuildingDto, CreateBuildingRequest
- ``unit_dto.rs`` - UnitDto, CreateUnitRequest
- ``owner_dto.rs`` - OwnerDto, CreateOwnerRequest
- ``expense_dto.rs`` - ExpenseDto, CreateExpenseRequest

Ports (Interfaces)
~~~~~~~~~~~~~~~~~~

Traits définissant les contrats pour les repositories:

- ``user_repository.rs``
- ``organization_repository.rs``
- ``building_repository.rs``
- ``unit_repository.rs``
- ``owner_repository.rs``
- ``expense_repository.rs``
- ``meeting_repository.rs``
- ``document_repository.rs``

Couche Infrastructure
---------------------

Base de données
~~~~~~~~~~~~~~~

.. code-block:: text

    infrastructure/database/
    ├── mod.rs                    # Exports publics
    ├── pool.rs                   # Pool de connexions SQLx
    ├── seed.rs                   # Seeding de données (SuperAdmin, demo)
    └── repositories/
        ├── user_repository_impl.rs
        ├── organization_repository_impl.rs
        ├── building_repository_impl.rs
        ├── unit_repository_impl.rs
        ├── owner_repository_impl.rs
        ├── expense_repository_impl.rs
        ├── meeting_repository_impl.rs
        └── document_repository_impl.rs

Web (API REST)
~~~~~~~~~~~~~~

.. code-block:: text

    infrastructure/web/
    ├── mod.rs                    # Exports publics
    ├── app_state.rs              # État partagé de l'application
    ├── routes.rs                 # Configuration des routes
    └── handlers/
        ├── auth_handlers.rs      # POST /api/v1/auth/login, /register, GET /me
        ├── seed_handlers.rs      # POST /api/v1/seed/demo, /clear
        ├── building_handlers.rs  # CRUD /api/v1/buildings
        ├── unit_handlers.rs      # CRUD /api/v1/units
        ├── owner_handlers.rs     # CRUD /api/v1/owners
        ├── expense_handlers.rs   # CRUD /api/v1/expenses
        └── health.rs             # GET /health

Migrations SQL
--------------

Les migrations sont gérées par SQLx:

.. code-block:: bash

    backend/migrations/
    ├── 20240101_create_users.sql
    ├── 20240102_create_organizations.sql
    ├── 20240103_create_buildings.sql
    ├── 20240104_create_units.sql
    ├── 20240105_create_owners.sql
    ├── 20240106_create_expenses.sql
    └── ...

Exécuter les migrations:

.. code-block:: bash

    cd backend
    sqlx migrate run

Documentation Frontend
======================

Structure
---------

.. code-block:: text

    frontend/src/
    ├── pages/                  # Routes Astro (SSR/SSG)
    │   ├── index.astro        # Landing page
    │   ├── login.astro        # Page de connexion
    │   ├── admin/
    │   │   └── index.astro    # Dashboard SuperAdmin
    │   ├── syndic/
    │   │   └── index.astro    # Dashboard Syndic
    │   ├── accountant/
    │   │   └── index.astro    # Dashboard Comptable
    │   ├── owner/
    │   │   └── index.astro    # Dashboard Copropriétaire
    │   └── buildings/
    │       └── index.astro    # Liste des immeubles
    │
    ├── components/             # Composants Svelte
    │   ├── LoginForm.svelte
    │   ├── Navigation.svelte
    │   ├── SyncStatus.svelte
    │   ├── BuildingList.svelte
    │   └── dashboards/
    │       ├── AdminDashboard.svelte
    │       ├── SyndicDashboard.svelte
    │       ├── AccountantDashboard.svelte
    │       └── OwnerDashboard.svelte
    │
    ├── lib/                    # Utilitaires et configuration
    │   ├── config.ts          # Configuration API centralisée
    │   ├── types.ts           # Types TypeScript
    │   ├── db.ts              # Wrapper IndexedDB
    │   └── sync.ts            # Service de synchronisation
    │
    ├── stores/                 # Stores Svelte
    │   └── auth.ts            # Store d'authentification
    │
    └── layouts/
        └── Layout.astro       # Layout principal

Fonctionnalités clés
--------------------

PWA (Progressive Web App)
~~~~~~~~~~~~~~~~~~~~~~~~~

- Service Worker avec Workbox
- Manifest PWA pour installation
- Mode offline avec IndexedDB
- Synchronisation bidirectionnelle

Authentification
~~~~~~~~~~~~~~~~

- JWT avec refresh token
- Persistance localStorage + IndexedDB
- Redirections selon rôle
- Middleware de protection des routes

Multi-rôles
~~~~~~~~~~~

- SuperAdmin: accès plateforme complet
- Syndic: gestion complète immeubles
- Accountant: accès finances
- Owner: consultation limitée

Tests E2E
=========

Framework: Playwright avec enregistrement vidéo

.. code-block:: text

    frontend/tests/e2e/
    ├── config.ts              # Configuration des tests
    ├── auth.spec.ts           # Tests d'authentification (8 tests)
    ├── dashboards.spec.ts     # Tests des dashboards (8 tests)
    └── pwa-offline.spec.ts    # Tests PWA/offline (8 tests)

Total: 24 tests E2E

Exécuter les tests:

.. code-block:: bash

    cd frontend
    npm run test:e2e         # Mode headless avec vidéos
    npm run test:e2e:ui      # Mode UI interactif
    npm run test:e2e:debug   # Mode debug pas à pas

API REST
========

Authentification
----------------

.. code-block:: http

    POST /api/v1/auth/register
    Content-Type: application/json

    {
      "email": "user@example.com",
      "password": "password123",
      "first_name": "John",
      "last_name": "Doe",
      "role": "syndic"
    }

    Response 201:
    {
      "token": "eyJ...",
      "user": {
        "id": "uuid",
        "email": "user@example.com",
        "first_name": "John",
        "last_name": "Doe",
        "role": "syndic"
      }
    }

.. code-block:: http

    POST /api/v1/auth/login
    Content-Type: application/json

    {
      "email": "user@example.com",
      "password": "password123"
    }

    Response 200:
    {
      "token": "eyJ...",
      "user": { /* ... */ }
    }

.. code-block:: http

    GET /api/v1/auth/me
    Authorization: Bearer eyJ...

    Response 200:
    {
      "id": "uuid",
      "email": "user@example.com",
      "first_name": "John",
      "last_name": "Doe",
      "role": "syndic"
    }

Immeubles
---------

.. code-block:: http

    GET /api/v1/buildings
    Authorization: Bearer eyJ...

    Response 200:
    [
      {
        "id": "uuid",
        "name": "Résidence Les Jardins",
        "address": "123 Rue de la Paix",
        "city": "Paris",
        "postal_code": "75001",
        "country": "France",
        "total_units": 50
      }
    ]

Seeding
-------

.. code-block:: http

    POST /api/v1/seed/demo
    Authorization: Bearer eyJ... (SuperAdmin)

    Response 200:
    {
      "message": "Demo data created successfully",
      "users": [/* credentials */]
    }

    POST /api/v1/seed/clear
    Authorization: Bearer eyJ... (SuperAdmin)

    Response 200:
    {
      "message": "Demo data cleared successfully"
    }

Guides de développement
=======================

Installation
------------

.. code-block:: bash

    # Cloner le projet
    git clone https://github.com/your-org/koprogo.git
    cd koprogo

    # Installation complète
    make setup

    # Démarrer les services
    make dev            # Backend seul
    make dev-frontend   # Frontend seul (autre terminal)
    make dev-all        # Tout en Docker

Commandes Make
--------------

.. code-block:: bash

    make help           # Affiche toutes les commandes disponibles
    make setup          # Installation complète
    make dev            # Démarre backend
    make dev-frontend   # Démarre frontend
    make dev-all        # Démarre tout avec Docker
    make test           # Run all tests
    make test-e2e       # Tests E2E avec vidéos
    make clean          # Nettoyage

Workflow Git
------------

1. Créer une branche feature:

   .. code-block:: bash

       git checkout -b feature/my-feature

2. Développer et committer:

   .. code-block:: bash

       git add .
       git commit -m "feat: Add my feature"

3. Pousser et créer PR:

   .. code-block:: bash

       git push origin feature/my-feature
       gh pr create

CI/CD
-----

GitHub Actions avec 3 workflows:

1. **Backend CI** (``.github/workflows/backend-ci.yml``)
   - Tests unitaires
   - Tests BDD
   - Clippy (linter)
   - Format check

2. **Frontend CI** (``.github/workflows/frontend-ci.yml``)
   - Tests E2E avec vidéos
   - Build production
   - Lint TypeScript

3. **Full Stack CI** (``.github/workflows/full-ci.yml``)
   - Intégration complète
   - Tests end-to-end complets

Variables d'environnement
==========================

Backend (.env)
--------------

.. code-block:: bash

    DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_db
    JWT_SECRET=your-super-secret-key-256-bits-min
    SERVER_HOST=127.0.0.1
    SERVER_PORT=8080
    RUST_LOG=info

Frontend (.env)
---------------

.. code-block:: bash

    PUBLIC_API_URL=http://127.0.0.1:8080

Déploiement
===========

Docker Production
-----------------

.. code-block:: bash

    # Build des images
    docker-compose build

    # Démarrer en production
    docker-compose up -d

    # Vérifier les logs
    docker-compose logs -f api
    docker-compose logs -f frontend

Variables pour production
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # Backend
    DATABASE_URL=postgresql://user:pass@prod-db:5432/koprogo_prod
    JWT_SECRET=$(openssl rand -base64 32)
    SERVER_HOST=0.0.0.0
    SERVER_PORT=8080

    # Frontend
    PUBLIC_API_URL=https://api.koprogo.com

Sécurité
========

Bonnes pratiques implémentées
------------------------------

1. **Mots de passe**: Bcrypt avec cost factor 12
2. **JWT**: Tokens avec expiration 24h
3. **CORS**: Configuration restrictive en production
4. **SQL Injection**: Requêtes paramétrées SQLx
5. **XSS**: Échappement automatique Svelte
6. **Multi-tenant**: Isolation par organization_id
7. **Validation**: Côté serveur avec validator crate

SuperAdmin par défaut
----------------------

.. code-block:: text

    Email: admin@koprogo.com
    Password: admin123

    ⚠️ À CHANGER EN PRODUCTION!

Glossaire
=========

.. glossary::

   Building
      Immeuble en copropriété géré par un syndic

   Unit
      Lot dans un immeuble (appartement, parking, cave)

   Owner
      Copropriétaire possédant un ou plusieurs lots

   Expense
      Charge ou dépense de copropriété

   Organization
      Entité multi-tenant (cabinet de syndic)

   SuperAdmin
      Administrateur plateforme avec accès universel

   Syndic
      Gestionnaire de copropriété

   Accountant
      Comptable avec accès limité aux finances

Ressources
==========

Documentation externe
---------------------

- `Rust Book <https://doc.rust-lang.org/book/>`_
- `Actix-web <https://actix.rs/>`_
- `SQLx <https://github.com/launchbadge/sqlx>`_
- `Astro <https://astro.build/>`_
- `Svelte <https://svelte.dev/>`_
- `Playwright <https://playwright.dev/>`_

Liens projet
------------

- Repository: (à définir)
- Issues: (à définir)
- Wiki: (à définir)

Contributeurs
=============

(Liste à compléter)

Licence
=======

(À définir)

Statut de la documentation
==========================

.. list-table::
   :header-rows: 1
   :widths: 50 20 30

   * - Section
     - Statut
     - Dernière MAJ
   * - Backend - Point d'entrée (main.rs, lib.rs, config.rs)
     - ✅ Complet
     - 2025-10-22
   * - Backend - Entités (User, Building)
     - ✅ Complet
     - 2025-10-22
   * - Backend - Autres entités
     - 🚧 À faire
     - -
   * - Backend - Use Cases
     - 🚧 À faire
     - -
   * - Backend - Repositories
     - 🚧 À faire
     - -
   * - Backend - Handlers
     - 🚧 À faire
     - -
   * - Frontend - Pages
     - 🚧 À faire
     - -
   * - Frontend - Composants
     - 🚧 À faire
     - -
   * - Frontend - Lib & Stores
     - 🚧 À faire
     - -
   * - Configuration & DevOps
     - 🚧 À faire
     - -

**Légende:**

- ✅ Complet - Documentation détaillée avec exemples
- 🚧 À faire - Section à documenter
- ⚠️ Partiel - Documentation incomplète

---

Guides et Documentation Détaillée
==================================

.. toctree::
   :maxdepth: 2
   :caption: Documentation Projet

   README
   changelog

.. toctree::
   :maxdepth: 2
   :caption: Business & Roadmap

   BUSINESS_PLAN_BOOTSTRAP
   INFRASTRUCTURE_ROADMAP

.. toctree::
   :maxdepth: 2
   :caption: Guides de Déploiement

   VPS_DEPLOYMENT
   DEPLOY_GITOPS

.. toctree::
   :maxdepth: 2
   :caption: Guides de Développement

   MAKEFILE_GUIDE
   E2E_TESTING_GUIDE
   PERFORMANCE_TESTING
   PERFORMANCE_REPORT

.. toctree::
   :maxdepth: 1
   :caption: Archives

   archive/BUSINESS_PLAN
   archive/MARKET_ANALYSIS
   archive/ROADMAP
   archive/ANALYSIS
   archive/SESSION_SUMMARY
   archive/NEW_ISSUES
   archive/PRIORITIES_TABLE
   archive/ISSUE_004_COMPLETION_GUIDE
   archive/load-tests-troubleshooting/PANIC_FIXES
   archive/load-tests-troubleshooting/IMPLEMENTATION_SUMMARY
   archive/load-tests-troubleshooting/TROUBLESHOOTING_401
   archive/load-tests-troubleshooting/CHANGELOG_RATE_LIMITING
   archive/root-md/DEPLOYMENT_VPS
   archive/root-md/infrastructure

.. toctree::
   :maxdepth: 2
   :caption: Entités du Domaine

   backend/src/domain/entities/building
   backend/src/domain/entities/user
   backend/src/domain/entities/organization
   backend/src/domain/entities/unit
   backend/src/domain/entities/owner
   backend/src/domain/entities/expense
   backend/src/domain/entities/meeting
   backend/src/domain/entities/document

---

*Cette documentation est générée et maintenue pour le projet Koprogo.*
*Dernière mise à jour: 2025-10-25*

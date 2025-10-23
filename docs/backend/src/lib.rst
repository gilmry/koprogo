====================
backend/src/lib.rs
====================

Description
===========

Fichier racine de la bibliothèque ``koprogo_api``. Ce fichier déclare les modules publics qui composent l'architecture hexagonale de l'application. Il sert de point d'entrée pour l'exposition des modules à l'extérieur de la crate.

Responsabilités
===============

1. **Déclaration des modules publics**
   - Exposition des trois couches de l'architecture hexagonale
   - Organisation de la structure du projet

2. **Encapsulation**
   - Contrôle de la visibilité des modules
   - Permet aux autres fichiers d'importer via ``koprogo_api::*``

Modules
=======

application
-----------

**Module:** ``pub mod application``

**Description:**

Couche application de l'architecture hexagonale. Contient la logique métier de l'application organisée selon le pattern Use Cases.

**Contenu:**

- ``use_cases/`` - Cas d'usage métier (AuthUseCases, BuildingUseCases, etc.)
- ``dto/`` - Data Transfer Objects pour les échanges de données
- ``ports/`` - Interfaces (traits) pour les repositories

**Rôle:**

Cette couche orchestre les opérations métier en utilisant les entités du domaine et les repositories de l'infrastructure. Elle est indépendante de la base de données et du framework web.

domain
------

**Module:** ``pub mod domain``

**Description:**

Couche domaine de l'architecture hexagonale. Contient les entités métier et la logique métier pure.

**Contenu:**

- ``entities/`` - Entités métier (User, Building, Unit, Owner, Expense, etc.)
- ``services/`` - Services de domaine (ExpenseCalculator)

**Rôle:**

Cette couche représente le cœur métier de l'application. Elle est complètement indépendante des frameworks, de la base de données et des protocoles de communication. Les entités contiennent leur propre validation métier.

infrastructure
--------------

**Module:** ``pub mod infrastructure``

**Description:**

Couche infrastructure de l'architecture hexagonale. Contient les implémentations techniques des adaptateurs externes.

**Contenu:**

- ``database/`` - Implémentation des repositories PostgreSQL avec SQLx
- ``web/`` - Configuration Actix-web, handlers HTTP, routes, middleware

**Rôle:**

Cette couche implémente les détails techniques de l'application:

- Accès à la base de données PostgreSQL
- Exposition des endpoints HTTP REST
- Sérialisation/désérialisation JSON
- Gestion des erreurs HTTP

Architecture hexagonale
=======================

Le fichier organise l'application selon le pattern **Hexagonal Architecture** (Ports & Adapters):

.. code-block:: text

    ┌──────────────────────────────────────────────────────────────┐
    │                         lib.rs                               │
    │                   (Module Declarations)                      │
    └──────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │   DOMAIN    │  │ APPLICATION │  │INFRASTRUCTURE│
    │  (Entities) │  │(Use Cases)  │  │  (Adapters) │
    └─────────────┘  └─────────────┘  └─────────────┘
         Core              │                  │
        Métier        Orchestration    Implémentation
                                          Technique

Flux de dépendances
===================

Les dépendances suivent la règle de l'architecture hexagonale:

.. code-block:: text

    Infrastructure ──> Application ──> Domain
         (Web)          (Use Cases)   (Entities)
         (DB)

    ❌ Domain ne dépend de personne
    ❌ Application dépend uniquement de Domain
    ❌ Infrastructure dépend de Application et Domain

**Principe:** Les détails techniques (web, DB) dépendent du métier, jamais l'inverse.

Utilisation
===========

**Dans main.rs:**

.. code-block:: rust

    use koprogo_api::application::use_cases::*;
    use koprogo_api::infrastructure::database::*;
    use koprogo_api::infrastructure::web::{configure_routes, AppState};

**Dans les tests:**

.. code-block:: rust

    use koprogo_api::domain::entities::Building;
    use koprogo_api::application::dto::BuildingDto;

**Dans d'autres crates:**

.. code-block:: rust

    // Cargo.toml
    [dependencies]
    koprogo_api = { path = "../backend" }

    // Dans le code
    use koprogo_api::domain::entities::User;

Structure complète du projet
=============================

.. code-block:: text

    src/
    ├── lib.rs                    (ce fichier)
    ├── main.rs                   (point d'entrée serveur)
    ├── config.rs                 (configuration)
    ├── db.rs                     (utilitaires DB)
    │
    ├── domain/
    │   ├── mod.rs
    │   ├── entities/
    │   │   ├── user.rs
    │   │   ├── organization.rs
    │   │   ├── building.rs
    │   │   ├── unit.rs
    │   │   ├── owner.rs
    │   │   ├── expense.rs
    │   │   ├── meeting.rs
    │   │   └── document.rs
    │   └── services/
    │       └── expense_calculator.rs
    │
    ├── application/
    │   ├── mod.rs
    │   ├── dto/
    │   │   ├── auth_dto.rs
    │   │   ├── building_dto.rs
    │   │   ├── unit_dto.rs
    │   │   ├── owner_dto.rs
    │   │   └── expense_dto.rs
    │   ├── ports/
    │   │   ├── user_repository.rs
    │   │   ├── organization_repository.rs
    │   │   ├── building_repository.rs
    │   │   ├── unit_repository.rs
    │   │   ├── owner_repository.rs
    │   │   ├── expense_repository.rs
    │   │   ├── meeting_repository.rs
    │   │   └── document_repository.rs
    │   └── use_cases/
    │       ├── auth_use_cases.rs
    │       ├── building_use_cases.rs
    │       ├── unit_use_cases.rs
    │       ├── owner_use_cases.rs
    │       └── expense_use_cases.rs
    │
    └── infrastructure/
        ├── mod.rs
        ├── database/
        │   ├── mod.rs
        │   ├── pool.rs
        │   ├── seed.rs
        │   └── repositories/
        │       ├── user_repository_impl.rs
        │       ├── organization_repository_impl.rs
        │       ├── building_repository_impl.rs
        │       ├── unit_repository_impl.rs
        │       ├── owner_repository_impl.rs
        │       ├── expense_repository_impl.rs
        │       ├── meeting_repository_impl.rs
        │       └── document_repository_impl.rs
        └── web/
            ├── mod.rs
            ├── app_state.rs
            ├── routes.rs
            └── handlers/
                ├── auth_handlers.rs
                ├── seed_handlers.rs
                ├── building_handlers.rs
                ├── unit_handlers.rs
                ├── owner_handlers.rs
                ├── expense_handlers.rs
                └── health.rs

Avantages de cette architecture
================================

1. **Testabilité**
   - Chaque couche peut être testée indépendamment
   - Mock des repositories facilité par les traits (ports)

2. **Maintenabilité**
   - Séparation claire des responsabilités
   - Changement de base de données sans toucher au métier

3. **Évolutivité**
   - Ajout de nouveaux use cases sans modifier l'infrastructure
   - Ajout de nouveaux adapters (GraphQL, gRPC) facilement

4. **Indépendance des frameworks**
   - Logique métier isolée d'Actix-web et SQLx
   - Migration vers un autre framework facilitée

Références
==========

- Clean Architecture (Robert C. Martin)
- Hexagonal Architecture (Alistair Cockburn)
- Domain-Driven Design (Eric Evans)

Fichiers associés
=================

- ``backend/src/main.rs`` - Point d'entrée principal
- ``backend/src/domain/mod.rs`` - Module domaine
- ``backend/src/application/mod.rs`` - Module application
- ``backend/src/infrastructure/mod.rs`` - Module infrastructure

=======================
backend/src/main.rs
=======================

Description
===========

Point d'entrée principal de l'application backend Koprogo API. Ce fichier configure et démarre le serveur HTTP Actix-web avec toutes les dépendances nécessaires pour l'application de gestion de copropriété.

Responsabilités
===============

1. **Initialisation de l'environnement**
   - Chargement des variables d'environnement depuis ``.env``
   - Configuration du logger pour le suivi des événements

2. **Configuration de la base de données**
   - Création du pool de connexions PostgreSQL
   - Exécution automatique des migrations SQLx au démarrage
   - Initialisation du compte SuperAdmin avec credentials par défaut

3. **Architecture hexagonale - Injection de dépendances**
   - Initialisation des repositories (couche infrastructure)
   - Création des use cases (couche application)
   - Configuration de l'état de l'application (AppState)

4. **Configuration du serveur HTTP**
   - Configuration CORS pour les requêtes cross-origin
   - Middleware de logging des requêtes HTTP
   - Enregistrement des routes API

Fonctions
=========

main()
------

**Signature:**

.. code-block:: rust

    #[actix_web::main]
    async fn main() -> std::io::Result<()>

**Description:**

Fonction principale asynchrone qui initialise et démarre l'application backend.

**Comportement:**

1. **Chargement de la configuration:**

   - ``DATABASE_URL`` (obligatoire) - URL de connexion PostgreSQL
   - ``JWT_SECRET`` (optionnel, défaut: "super-secret-key-change-in-production")
   - ``SERVER_HOST`` (optionnel, défaut: "127.0.0.1")
   - ``SERVER_PORT`` (optionnel, défaut: "8080")

2. **Initialisation base de données:**

   - Création du pool avec ``create_pool(&database_url)``
   - Exécution migrations avec ``sqlx::migrate!("./migrations").run(&pool)``
   - Seeding du compte SuperAdmin (email: admin@koprogo.com, password: admin123)

3. **Initialisation des repositories (Arc pour thread-safety):**

   - ``PostgresUserRepository`` - Gestion des utilisateurs
   - ``PostgresBuildingRepository`` - Gestion des immeubles
   - ``PostgresUnitRepository`` - Gestion des lots
   - ``PostgresOwnerRepository`` - Gestion des copropriétaires
   - ``PostgresExpenseRepository`` - Gestion des charges

4. **Création des use cases:**

   - ``AuthUseCases::new(user_repo, jwt_secret)`` - Authentification JWT
   - ``BuildingUseCases::new(building_repo)`` - Logique métier immeubles
   - ``UnitUseCases::new(unit_repo)`` - Logique métier lots
   - ``OwnerUseCases::new(owner_repo)`` - Logique métier copropriétaires
   - ``ExpenseUseCases::new(expense_repo)`` - Logique métier charges

5. **Configuration du serveur Actix-web:**

   - CORS permissif (allow_any_origin) pour le développement
   - Logger middleware pour tracer les requêtes HTTP
   - Binding sur ``server_host:server_port``

**Variables d'environnement:**

.. list-table::
   :header-rows: 1

   * - Variable
     - Obligatoire
     - Valeur par défaut
     - Description
   * - DATABASE_URL
     - Oui
     - -
     - URL PostgreSQL (ex: postgres://user:pass@localhost/koprogo)
   * - JWT_SECRET
     - Non
     - super-secret-key-change-in-production
     - Clé secrète pour signer les tokens JWT
   * - SERVER_HOST
     - Non
     - 127.0.0.1
     - Adresse IP d'écoute du serveur
   * - SERVER_PORT
     - Non
     - 8080
     - Port d'écoute du serveur HTTP

**Retour:**

- ``Ok(())`` si le serveur démarre et s'exécute avec succès
- ``Err(std::io::Error)`` en cas d'erreur I/O (port déjà utilisé, etc.)

**Exemple de logs au démarrage:**

.. code-block:: text

    [2025-10-22T10:00:00Z INFO] Seeding superadmin account...
    [2025-10-22T10:00:00Z INFO] SuperAdmin account ready: admin@koprogo.com
    [2025-10-22T10:00:00Z INFO] Starting server at 127.0.0.1:8080

Architecture
============

Le fichier implémente le pattern **Hexagonal Architecture (Ports & Adapters)**:

.. code-block:: text

    ┌─────────────────────────────────────────────────────────────┐
    │                        main.rs                              │
    │                    (Composition Root)                       │
    └─────────────────────────────────────────────────────────────┘
                              │
                              ├──> Infrastructure Layer
                              │    ├─ PostgresUserRepository
                              │    ├─ PostgresBuildingRepository
                              │    ├─ PostgresUnitRepository
                              │    ├─ PostgresOwnerRepository
                              │    └─ PostgresExpenseRepository
                              │
                              ├──> Application Layer
                              │    ├─ AuthUseCases
                              │    ├─ BuildingUseCases
                              │    ├─ UnitUseCases
                              │    ├─ OwnerUseCases
                              │    └─ ExpenseUseCases
                              │
                              └──> Web Layer
                                   ├─ AppState (shared state)
                                   ├─ CORS middleware
                                   ├─ Logger middleware
                                   └─ Routes configuration

Dépendances
===========

Crates externes:

- ``actix-web`` - Framework web asynchrone
- ``actix-cors`` - Middleware CORS
- ``dotenv`` - Chargement variables d'environnement
- ``env_logger`` - Logging configurable
- ``sqlx`` - Client PostgreSQL avec migrations

Modules internes:

- ``koprogo_api::application::use_cases`` - Cas d'usage métier
- ``koprogo_api::infrastructure::database`` - Couche accès données
- ``koprogo_api::infrastructure::web`` - Configuration web et routes

Notes de sécurité
=================

.. warning::

   **Configuration CORS en développement:**

   Le serveur utilise ``allow_any_origin()`` qui accepte les requêtes de n'importe quelle origine.

   **En production**, il faut remplacer par:

   .. code-block:: rust

       let cors = Cors::default()
           .allowed_origin("https://app.koprogo.com")
           .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
           .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
           .max_age(3600);

.. warning::

   **JWT_SECRET en production:**

   La valeur par défaut "super-secret-key-change-in-production" N'EST PAS sécurisée.

   Utilisez une chaîne aléatoire forte de minimum 256 bits:

   .. code-block:: bash

       openssl rand -base64 32

Fichier associé
===============

- ``backend/src/lib.rs`` - Déclaration des modules publics
- ``backend/src/config.rs`` - Configuration structurée (alternative non utilisée)
- ``docker-compose.yml`` - Configuration Docker avec variables d'environnement
- ``.env`` - Fichier de variables d'environnement local

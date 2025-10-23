========================
backend/src/config.rs
========================

Description
===========

Module de configuration structurée de l'application backend. Fournit une struct ``Config`` qui centralise tous les paramètres de configuration chargés depuis les variables d'environnement.

.. note::

   Ce fichier définit une structure de configuration, mais **n'est actuellement pas utilisé** dans ``main.rs``.
   Le fichier ``main.rs`` charge les variables d'environnement directement.

Responsabilités
===============

1. **Centralisation de la configuration**
   - Regroupe tous les paramètres dans une seule structure
   - Type-safety avec Rust

2. **Chargement depuis l'environnement**
   - Lecture des variables d'environnement
   - Valeurs par défaut pour le développement
   - Validation des types (parsing)

Structures
==========

Config
------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone)]
    pub struct Config {
        pub database_url: String,
        pub jwt_secret: String,
        pub server_host: String,
        pub server_port: u16,
    }

**Description:**

Structure contenant toute la configuration de l'application.

**Champs:**

.. list-table::
   :header-rows: 1
   :widths: 20 15 65

   * - Champ
     - Type
     - Description
   * - ``database_url``
     - ``String``
     - URL de connexion à PostgreSQL (format: ``postgres://user:pass@host:port/database``)
   * - ``jwt_secret``
     - ``String``
     - Clé secrète pour signer les tokens JWT (minimum 256 bits recommandés)
   * - ``server_host``
     - ``String``
     - Adresse IP d'écoute du serveur HTTP (ex: "127.0.0.1", "0.0.0.0")
   * - ``server_port``
     - ``u16``
     - Port d'écoute du serveur HTTP (ex: 8080, 3000)

**Traits dérivés:**

- ``Debug`` - Permet l'affichage avec ``{:?}`` pour le debugging
- ``Clone`` - Permet de cloner la configuration si nécessaire

Méthodes
========

from_env()
----------

**Signature:**

.. code-block:: rust

    pub fn from_env() -> Self

**Description:**

Constructeur qui charge la configuration depuis les variables d'environnement.

**Variables d'environnement lues:**

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Variable
     - Obligatoire
     - Comportement
   * - ``DATABASE_URL``
     - ✅ Oui
     - **Panic si absente** avec message: "DATABASE_URL must be set"
   * - ``JWT_SECRET``
     - ✅ Oui
     - **Panic si absente** avec message: "JWT_SECRET must be set"
   * - ``SERVER_HOST``
     - ❌ Non
     - Défaut: ``"127.0.0.1"`` (localhost)
   * - ``SERVER_PORT``
     - ❌ Non
     - Défaut: ``"8080"``, **panic si non-numérique**

**Comportement:**

1. Lit chaque variable d'environnement
2. Pour les variables obligatoires: panic si absentes
3. Pour les variables optionnelles: utilise valeur par défaut
4. Parse ``SERVER_PORT`` en ``u16``, panic si invalide

**Retour:**

Instance de ``Config`` avec tous les champs initialisés.

**Exemple d'utilisation:**

.. code-block:: rust

    use koprogo_api::config::Config;
    use dotenv::dotenv;

    #[actix_web::main]
    async fn main() {
        dotenv().ok();

        // Charge la configuration
        let config = Config::from_env();

        println!("Server will start on {}:{}", config.server_host, config.server_port);
        println!("Database: {}", config.database_url);
    }

**Cas d'erreur:**

.. code-block:: rust

    // ❌ DATABASE_URL absente
    // Panic: "DATABASE_URL must be set"

    // ❌ JWT_SECRET absente
    // Panic: "JWT_SECRET must be set"

    // ❌ SERVER_PORT non numérique (ex: "abc")
    // Panic: "SERVER_PORT must be a valid number"

Avantages de cette approche
============================

1. **Type Safety**

   .. code-block:: rust

       // ✅ Le port est toujours un u16 valide
       let port: u16 = config.server_port;

       // ❌ Pas de parsing manuel partout
       // let port = env::var("SERVER_PORT").parse::<u16>().unwrap();

2. **Validation centralisée**

   - Toutes les erreurs de configuration détectées au démarrage
   - Pas de surprise en cours d'exécution

3. **Documentation des dépendances**

   - Les champs de la struct documentent les variables nécessaires
   - Facile de voir toute la configuration d'un coup d'œil

4. **Facilité de test**

   .. code-block:: rust

       #[cfg(test)]
       mod tests {
           use super::*;

           fn test_config() -> Config {
               Config {
                   database_url: "postgres://test:test@localhost/test_db".to_string(),
                   jwt_secret: "test-secret-key".to_string(),
                   server_host: "127.0.0.1".to_string(),
                   server_port: 8080,
               }
           }
       }

Utilisation actuelle vs. potentielle
=====================================

Approche actuelle (main.rs)
----------------------------

.. code-block:: rust

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-key-change-in-production".to_string());
    let server_host = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");

Approche avec Config
--------------------

.. code-block:: rust

    use koprogo_api::config::Config;

    let config = Config::from_env();

    let pool = create_pool(&config.database_url).await?;
    let auth_use_cases = AuthUseCases::new(user_repo, config.jwt_secret);

    HttpServer::new(move || { /* ... */ })
        .bind((config.server_host.as_str(), config.server_port))?
        .run()
        .await

Recommandation
==============

.. tip::

   **Pour améliorer le code:**

   1. Utiliser ``Config::from_env()`` dans ``main.rs`` au lieu de variables séparées
   2. Passer ``&Config`` ou ``Arc<Config>`` aux composants qui en ont besoin
   3. Facilite l'ajout de nouvelles variables de configuration

Améliorations possibles
=======================

1. **Support fichiers de configuration**

   .. code-block:: rust

       impl Config {
           pub fn from_file(path: &str) -> Result<Self, ConfigError> {
               // Charger depuis config.toml ou config.yaml
           }
       }

2. **Validation métier**

   .. code-block:: rust

       impl Config {
           pub fn validate(&self) -> Result<(), ConfigError> {
               if self.jwt_secret.len() < 32 {
                   return Err(ConfigError::JwtSecretTooShort);
               }
               if self.server_port < 1024 {
                   return Err(ConfigError::PrivilegedPort);
               }
               Ok(())
           }
       }

3. **Environnements multiples**

   .. code-block:: rust

       #[derive(Debug, Clone)]
       pub enum Environment {
           Development,
           Staging,
           Production,
       }

       impl Config {
           pub fn environment(&self) -> Environment {
               match env::var("ENV").as_deref() {
                   Ok("production") => Environment::Production,
                   Ok("staging") => Environment::Staging,
                   _ => Environment::Development,
               }
           }
       }

Fichier .env exemple
====================

.. code-block:: bash

    # Database
    DATABASE_URL=postgres://koprogo:koprogo123@localhost:5432/koprogo_db

    # JWT Secret (générer avec: openssl rand -base64 32)
    JWT_SECRET=votre-cle-secrete-super-forte-ici

    # Server
    SERVER_HOST=127.0.0.1
    SERVER_PORT=8080

    # Environment
    ENV=development

Dépendances
===========

- ``std::env`` - Accès aux variables d'environnement

Fichiers associés
=================

- ``backend/src/main.rs`` - Utilise les variables d'environnement directement
- ``backend/.env`` - Fichier de variables d'environnement local
- ``docker-compose.yml`` - Variables d'environnement pour Docker

Tests - Stratégie de Tests
===========================

Stratégie de tests complète suivant la pyramide des tests.

Pyramide des Tests
------------------

.. code-block:: text

                 ▲
                / \
               /E2E\         < 10% (tests end-to-end complets)
              /_____\
             /       \
            /  BDD    \      < 20% (tests comportementaux Gherkin)
           /__________\
          /            \
         / Integration  \   < 30% (tests avec DB réelle)
        /________________\
       /                  \
      /    Unit Tests      \  < 40% (tests logique pure)
     /______________________\

**Objectif Coverage** : 80%+ global (100% domaine, 80% application, 60% infrastructure)

Structure
---------

.. code-block:: text

   backend/tests/
   ├── integration/           # Tests d'intégration (testcontainers)
   │   ├── building_tests.rs
   │   ├── unit_tests.rs
   │   ├── owner_tests.rs
   │   └── expense_tests.rs
   ├── bdd.rs                 # Tests BDD (Cucumber)
   ├── features/              # Fichiers Gherkin
   │   ├── buildings.feature
   │   ├── units.feature
   │   ├── owners.feature
   │   └── expenses.feature
   └── e2e/                   # Tests end-to-end
       ├── api_tests.rs
       └── auth_tests.rs

Tests Unitaires
---------------

**Localisation** : Modules ``#[cfg(test)]`` dans chaque fichier source.

**Cible** : Logique domaine pure (entities, services).

**Exemple** :

.. code-block:: rust

   // backend/src/domain/entities/building.rs
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_create_building_success() {
           let building = Building::new(
               "Test Building".to_string(),
               "123 Main St".to_string(),
               "Paris".to_string(),
               "75001".to_string(),
               "France".to_string(),
               10,
               Some(2020),
           );

           assert!(building.is_ok());
           let building = building.unwrap();
           assert_eq!(building.name, "Test Building");
           assert_eq!(building.total_units, 10);
       }

       #[test]
       fn test_create_building_empty_name_fails() {
           let building = Building::new(
               "".to_string(),  // Invalide
               "123 Main St".to_string(),
               "Paris".to_string(),
               "75001".to_string(),
               "France".to_string(),
               10,
               None,
           );

           assert!(building.is_err());
           assert_eq!(
               building.unwrap_err(),
               "Building name cannot be empty"
           );
       }

       #[test]
       fn test_update_building_info() {
           let mut building = Building::new(
               "Old Name".to_string(),
               "Old Address".to_string(),
               "Old City".to_string(),
               "00000".to_string(),
               "France".to_string(),
               20,
               None,
           ).unwrap();

           let old_updated_at = building.updated_at;

           building.update_info(
               "New Name".to_string(),
               "New Address".to_string(),
               "New City".to_string(),
               "75001".to_string(),
           );

           assert_eq!(building.name, "New Name");
           assert!(building.updated_at > old_updated_at);
       }
   }

**Commande** :

.. code-block:: bash

   # Tous les tests unitaires
   cargo test --lib

   # Tests d'une entité spécifique
   cargo test --lib domain::entities::building

   # Tests avec output verbose
   cargo test --lib -- --nocapture

Tests d'Intégration
-------------------

**Localisation** : ``backend/tests/integration/``

**Cible** : Use cases + Repositories avec base de données réelle.

**Testcontainers** : PostgreSQL éphémère pour isolation complète.

**Exemple** :

.. code-block:: rust

   // backend/tests/integration/building_tests.rs
   use testcontainers::{clients::Cli, Container};
   use testcontainers_modules::postgres::Postgres;
   use sqlx::PgPool;

   async fn setup_test_db(
       docker: &Cli
   ) -> (Container<'_, Postgres>, PgPool) {
       // Démarrer PostgreSQL testcontainer
       let postgres = docker.run(Postgres::default());

       let connection_string = format!(
           "postgres://postgres:postgres@127.0.0.1:{}/postgres",
           postgres.get_host_port_ipv4(5432)
       );

       // Créer pool
       let pool = PgPool::connect(&connection_string)
           .await
           .expect("Failed to connect to test database");

       // Run migrations
       sqlx::migrate!()
           .run(&pool)
           .await
           .expect("Failed to run migrations");

       (postgres, pool)
   }

   #[tokio::test]
   async fn test_create_and_find_building() {
       let docker = Cli::default();
       let (_container, pool) = setup_test_db(&docker).await;

       // Créer repository
       let repo = PostgresBuildingRepository::new(pool.clone());

       // Créer building
       let building = Building::new(
           "Integration Test Building".to_string(),
           "123 Integration St".to_string(),
           "Paris".to_string(),
           "75001".to_string(),
           "France".to_string(),
           15,
           None,
       ).unwrap();

       // Sauvegarder
       let saved_building = repo.create(&building)
           .await
           .expect("Failed to create building");

       assert_eq!(saved_building.id, building.id);

       // Retrouver par ID
       let found_building = repo.find_by_id(building.id)
           .await
           .expect("Failed to find building")
           .expect("Building not found");

       assert_eq!(found_building.name, "Integration Test Building");
   }

   #[tokio::test]
   async fn test_update_building() {
       let docker = Cli::default();
       let (_container, pool) = setup_test_db(&docker).await;

       let repo = PostgresBuildingRepository::new(pool);

       // Créer
       let mut building = Building::new(
           "Original Name".to_string(),
           "Original Address".to_string(),
           "Paris".to_string(),
           "75001".to_string(),
           "France".to_string(),
           10,
           None,
       ).unwrap();

       let saved_building = repo.create(&building).await.unwrap();

       // Mettre à jour
       building.update_info(
           "Updated Name".to_string(),
           "Updated Address".to_string(),
           "Lyon".to_string(),
           "69001".to_string(),
       );

       let updated_building = repo.update(&building).await.unwrap();

       assert_eq!(updated_building.name, "Updated Name");
       assert_eq!(updated_building.city, "Lyon");
   }

   #[tokio::test]
   async fn test_delete_building() {
       let docker = Cli::default();
       let (_container, pool) = setup_test_db(&docker).await;

       let repo = PostgresBuildingRepository::new(pool);

       // Créer
       let building = Building::new(
           "To Delete".to_string(),
           "Address".to_string(),
           "City".to_string(),
           "12345".to_string(),
           "Country".to_string(),
           5,
           None,
       ).unwrap();

       let saved_building = repo.create(&building).await.unwrap();

       // Supprimer
       repo.delete(saved_building.id).await.unwrap();

       // Vérifier suppression
       let found = repo.find_by_id(saved_building.id).await.unwrap();
       assert!(found.is_none());
   }

**Commande** :

.. code-block:: bash

   # Tous les tests d'intégration
   cargo test --test integration

   # Test spécifique
   cargo test --test integration test_create_and_find_building

Tests BDD (Cucumber)
--------------------

**Localisation** : ``backend/tests/features/*.feature`` + ``backend/tests/bdd.rs``

**Cible** : Comportements utilisateur (Gherkin → Rust steps).

**Exemple Feature** :

.. code-block:: gherkin

   # backend/tests/features/buildings.feature
   Feature: Gestion des immeubles
     En tant que syndic
     Je veux gérer les immeubles de copropriété
     Afin de suivre mon portefeuille

     Scenario: Créer un nouvel immeuble
       Given je suis un syndic authentifié
       When je crée un immeuble avec les données suivantes:
         | name            | Résidence Les Jardins      |
         | address         | 15 Rue de la Paix          |
         | city            | Paris                      |
         | postal_code     | 75001                      |
         | country         | France                     |
         | total_units     | 45                         |
         | construction_year | 1990                     |
       Then l'immeuble est créé avec succès
       And l'immeuble contient 45 lots

     Scenario: Lister les immeubles d'un syndic
       Given je suis un syndic authentifié
       And j'ai créé 3 immeubles
       When je demande la liste de mes immeubles
       Then je reçois une liste de 3 immeubles

     Scenario: Modifier un immeuble existant
       Given je suis un syndic authentifié
       And j'ai créé un immeuble nommé "Ancien Nom"
       When je modifie le nom en "Nouveau Nom"
       Then l'immeuble a le nom "Nouveau Nom"

     Scenario: Supprimer un immeuble
       Given je suis un syndic authentifié
       And j'ai créé un immeuble
       When je supprime cet immeuble
       Then l'immeuble n'existe plus

     Scenario: Échouer à créer un immeuble avec nom vide
       Given je suis un syndic authentifié
       When je tente de créer un immeuble avec un nom vide
       Then je reçois une erreur "Building name cannot be empty"

**Implémentation Steps** :

.. code-block:: rust

   // backend/tests/bdd.rs
   use cucumber::{given, when, then, World};

   #[derive(Debug, Default, World)]
   pub struct BuildingWorld {
       auth_token: Option<String>,
       buildings: Vec<Building>,
       last_error: Option<String>,
       last_building: Option<Building>,
   }

   #[given("je suis un syndic authentifié")]
   async fn given_authenticated_syndic(world: &mut BuildingWorld) {
       // Mock JWT token
       world.auth_token = Some("mock-jwt-token".to_string());
   }

   #[when(regex = r"je crée un immeuble avec les données suivantes:")]
   async fn when_create_building(
       world: &mut BuildingWorld,
       step: &Step
   ) {
       let table = step.table.as_ref().unwrap();
       let data: HashMap<String, String> = table
           .rows
           .iter()
           .map(|row| (row[0].clone(), row[1].clone()))
           .collect();

       let building = Building::new(
           data["name"].clone(),
           data["address"].clone(),
           data["city"].clone(),
           data["postal_code"].clone(),
           data["country"].clone(),
           data["total_units"].parse().unwrap(),
           data.get("construction_year").and_then(|y| y.parse().ok()),
       );

       match building {
           Ok(b) => {
               world.last_building = Some(b.clone());
               world.buildings.push(b);
           }
           Err(e) => {
               world.last_error = Some(e);
           }
       }
   }

   #[then("l'immeuble est créé avec succès")]
   async fn then_building_created(world: &mut BuildingWorld) {
       assert!(world.last_building.is_some());
       assert!(world.last_error.is_none());
   }

   #[then(regex = r"l'immeuble contient (\d+) lots")]
   async fn then_building_has_units(world: &mut BuildingWorld, units: usize) {
       let building = world.last_building.as_ref().unwrap();
       assert_eq!(building.total_units as usize, units);
   }

**Commande** :

.. code-block:: bash

   # Tous les tests BDD
   cargo test --test bdd

   # Feature spécifique
   cargo test --test bdd -- buildings.feature

   # Avec output détaillé
   cargo test --test bdd -- --nocapture

Tests E2E (End-to-End)
----------------------

**Localisation** : ``backend/tests/e2e/``

**Cible** : API complète (HTTP requests → DB → responses).

**Exemple** :

.. code-block:: rust

   // backend/tests/e2e/api_tests.rs
   use actix_web::{test, App};

   #[actix_web::test]
   async fn test_complete_building_workflow() {
       // Setup test app
       let app = test::init_service(
           App::new()
               .app_data(web::Data::new(test_app_state()))
               .configure(configure_routes)
       ).await;

       // 1. Login
       let login_req = test::TestRequest::post()
           .uri("/api/v1/auth/login")
           .set_json(&json!({
               "email": "test@example.com",
               "password": "password123"
           }))
           .to_request();

       let login_resp = test::call_service(&app, login_req).await;
       assert_eq!(login_resp.status(), 200);

       let login_body: serde_json::Value = test::read_body_json(login_resp).await;
       let token = login_body["token"].as_str().unwrap();

       // 2. Create building
       let create_req = test::TestRequest::post()
           .uri("/api/v1/buildings")
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .set_json(&json!({
               "name": "E2E Test Building",
               "address": "123 E2E St",
               "city": "Paris",
               "postal_code": "75001",
               "country": "France",
               "total_units": 20
           }))
           .to_request();

       let create_resp = test::call_service(&app, create_req).await;
       assert_eq!(create_resp.status(), 201);

       let create_body: serde_json::Value = test::read_body_json(create_resp).await;
       let building_id = create_body["id"].as_str().unwrap();

       // 3. Get building
       let get_req = test::TestRequest::get()
           .uri(&format!("/api/v1/buildings/{}", building_id))
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .to_request();

       let get_resp = test::call_service(&app, get_req).await;
       assert_eq!(get_resp.status(), 200);

       let get_body: serde_json::Value = test::read_body_json(get_resp).await;
       assert_eq!(get_body["name"], "E2E Test Building");

       // 4. Update building
       let update_req = test::TestRequest::put()
           .uri(&format!("/api/v1/buildings/{}", building_id))
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .set_json(&json!({
               "name": "Updated E2E Building",
               "address": "123 E2E St",
               "city": "Lyon",
               "postal_code": "69001",
               "country": "France",
               "total_units": 20
           }))
           .to_request();

       let update_resp = test::call_service(&app, update_req).await;
       assert_eq!(update_resp.status(), 200);

       // 5. Delete building
       let delete_req = test::TestRequest::delete()
           .uri(&format!("/api/v1/buildings/{}", building_id))
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .to_request();

       let delete_resp = test::call_service(&app, delete_req).await;
       assert_eq!(delete_resp.status(), 204);

       // 6. Verify deletion
       let verify_req = test::TestRequest::get()
           .uri(&format!("/api/v1/buildings/{}", building_id))
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .to_request();

       let verify_resp = test::call_service(&app, verify_req).await;
       assert_eq!(verify_resp.status(), 404);
   }

**Commande** :

.. code-block:: bash

   # Tous les tests E2E
   cargo test --test e2e

Coverage
--------

**Tarpaulin** : Génération rapports coverage.

.. code-block:: bash

   # Générer coverage
   cargo tarpaulin --out Html --output-dir coverage

   # Output: coverage/index.html
   # Ouvrir dans navigateur
   xdg-open coverage/index.html

**Cible** :

- Domain : 100%
- Application : 80%+
- Infrastructure : 60%+
- Global : 80%+

CI/CD Tests
-----------

**GitHub Actions** : Exécution automatique tous les tests.

.. code-block:: yaml

   # .github/workflows/test.yml
   name: Tests

   on: [push, pull_request]

   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3

         - name: Install Rust
           uses: actions-rs/toolchain@v1
           with:
             toolchain: stable

         - name: Start PostgreSQL
           run: |
             docker-compose up -d postgres

         - name: Run migrations
           run: |
             cd backend && sqlx migrate run

         - name: Unit tests
           run: cargo test --lib

         - name: Integration tests
           run: cargo test --test integration

         - name: BDD tests
           run: cargo test --test bdd

         - name: E2E tests
           run: cargo test --test e2e

         - name: Coverage
           run: cargo tarpaulin --out Xml

         - name: Upload coverage
           uses: codecov/codecov-action@v3

Commandes Pratiques
-------------------

.. code-block:: bash

   # Tous les tests
   make test
   # ou
   cargo test

   # Tests rapides (skip integration/e2e)
   cargo test --lib

   # Tests avec output
   cargo test -- --nocapture

   # Tests parallèles
   cargo test -- --test-threads=4

   # Test spécifique
   cargo test test_create_building_success

   # Watch mode (auto-rerun)
   cargo watch -x test

Références
----------

- Rust Testing : https://doc.rust-lang.org/book/ch11-00-testing.html
- Cucumber Rust : https://cucumber-rs.github.io/cucumber/
- Testcontainers : https://docs.rs/testcontainers/
- Tarpaulin : https://github.com/xd009642/tarpaulin

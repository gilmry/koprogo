Infrastructure - Couche Infrastructure (Adapters)
==================================================

La couche Infrastructure implémente les détails techniques : accès base de données, exposition HTTP, gestion fichiers.

**Principe** : Infrastructure dépend de Application et Domain. Implémente les ports (traits) définis par Application.

Structure
---------

.. code-block:: text

   infrastructure/
   ├── database/                    # Adapter PostgreSQL (SQLx)
   │   ├── mod.rs
   │   ├── pool.rs                  # Connection pool
   │   ├── seed.rs                  # Seed données test
   │   └── repositories/            # Implémentations repositories
   │       ├── user_repository_impl.rs
   │       ├── organization_repository_impl.rs
   │       ├── building_repository_impl.rs
   │       ├── unit_repository_impl.rs
   │       ├── owner_repository_impl.rs
   │       ├── expense_repository_impl.rs
   │       ├── meeting_repository_impl.rs
   │       └── document_repository_impl.rs
   └── web/                         # Adapter HTTP (Actix-web)
       ├── mod.rs
       ├── app_state.rs             # État partagé application
       ├── routes.rs                # Configuration routes
       ├── middleware/              # Middleware HTTP
       │   ├── auth.rs              # JWT authentication
       │   └── cors.rs              # CORS policy
       └── handlers/                # Handlers HTTP
           ├── auth_handlers.rs
           ├── seed_handlers.rs
           ├── building_handlers.rs
           ├── unit_handlers.rs
           ├── owner_handlers.rs
           ├── expense_handlers.rs
           ├── meeting_handlers.rs
           ├── document_handlers.rs
           └── health.rs

Database (Adapter PostgreSQL)
------------------------------

**SQLx** : ORM avec vérification compile-time des requêtes.

Connection Pool
^^^^^^^^^^^^^^^

.. code-block:: rust

   use sqlx::postgres::PgPoolOptions;
   use sqlx::PgPool;

   pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
       PgPoolOptions::new()
           .max_connections(10)
           .connect(database_url)
           .await
   }

**Configuration** :

- Max connections : 10 (performance optimale pour charge typique)
- Timeout : 30s par défaut
- Health check : Ping automatique connexions

Repository Implementation
^^^^^^^^^^^^^^^^^^^^^^^^^

**Pattern** : Implémentation du trait port.

.. code-block:: rust

   pub struct PostgresBuildingRepository {
       pool: PgPool,
   }

   impl PostgresBuildingRepository {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
   }

   #[async_trait]
   impl BuildingRepository for PostgresBuildingRepository {
       async fn create(&self, building: &Building) -> Result<Building, String> {
           sqlx::query_as!(
               Building,
               r#"
               INSERT INTO buildings (
                   id, name, address, city, postal_code, country,
                   total_units, construction_year, organization_id,
                   created_at, updated_at
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *
               "#,
               building.id,
               building.name,
               building.address,
               building.city,
               building.postal_code,
               building.country,
               building.total_units,
               building.construction_year,
               building.organization_id,
               building.created_at,
               building.updated_at
           )
           .fetch_one(&self.pool)
           .await
           .map_err(|e| e.to_string())
       }

       async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String> {
           sqlx::query_as!(
               Building,
               r#"
               SELECT * FROM buildings WHERE id = $1
               "#,
               id
           )
           .fetch_optional(&self.pool)
           .await
           .map_err(|e| e.to_string())
       }

       async fn find_all_paginated(
           &self,
           organization_id: Uuid,
           page: i64,
           per_page: i64
       ) -> Result<PageResponse<Building>, String> {
           let offset = (page - 1) * per_page;

           // Compter total
           let total_items: i64 = sqlx::query_scalar!(
               "SELECT COUNT(*) FROM buildings WHERE organization_id = $1",
               organization_id
           )
           .fetch_one(&self.pool)
           .await
           .map_err(|e| e.to_string())?
           .unwrap_or(0);

           // Récupérer données paginées
           let buildings = sqlx::query_as!(
               Building,
               r#"
               SELECT * FROM buildings
               WHERE organization_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3
               "#,
               organization_id,
               per_page,
               offset
           )
           .fetch_all(&self.pool)
           .await
           .map_err(|e| e.to_string())?;

           let total_pages = (total_items + per_page - 1) / per_page;

           Ok(PageResponse {
               data: buildings,
               pagination: PaginationMeta {
                   current_page: page,
                   per_page,
                   total_items,
                   total_pages,
                   has_next: page < total_pages,
                   has_previous: page > 1,
               },
           })
       }

       async fn update(&self, building: &Building) -> Result<Building, String> {
           sqlx::query_as!(
               Building,
               r#"
               UPDATE buildings
               SET name = $2, address = $3, city = $4, postal_code = $5,
                   updated_at = $6
               WHERE id = $1
               RETURNING *
               "#,
               building.id,
               building.name,
               building.address,
               building.city,
               building.postal_code,
               building.updated_at
           )
           .fetch_one(&self.pool)
           .await
           .map_err(|e| e.to_string())
       }

       async fn delete(&self, id: Uuid) -> Result<(), String> {
           sqlx::query!("DELETE FROM buildings WHERE id = $1", id)
               .execute(&self.pool)
               .await
               .map_err(|e| e.to_string())?;

           Ok(())
       }
   }

**Avantages SQLx** :

- ✅ Vérification compile-time (``sqlx::query_as!``)
- ✅ Type-safe (mapping automatique Rust ↔ PostgreSQL)
- ✅ Performance (prepared statements)
- ✅ Prévention SQL injection (parameterized queries)

Seed Data
^^^^^^^^^

Génération données de test pour développement.

.. code-block:: rust

   pub async fn seed_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
       // Créer organisations
       let org = create_organization(pool, "Test Syndic").await?;

       // Créer users
       let user = create_user(pool, org.id, "admin@test.com", UserRole::Syndic).await?;

       // Créer buildings
       for i in 1..=5 {
           let building = create_building(
               pool,
               org.id,
               &format!("Résidence Test {}", i)
           ).await?;

           // Créer units pour chaque building
           for j in 1..=10 {
               create_unit(pool, building.id, &format!("A-{}", j)).await?;
           }
       }

       Ok(())
   }

Web (Adapter HTTP)
------------------

**Actix-web** : Framework web performant et type-safe.

App State
^^^^^^^^^

État partagé entre handlers (pool DB, use cases).

.. code-block:: rust

   pub struct AppState {
       pub building_use_cases: Arc<BuildingUseCases>,
       pub unit_use_cases: Arc<UnitUseCases>,
       pub owner_use_cases: Arc<OwnerUseCases>,
       pub expense_use_cases: Arc<ExpenseUseCases>,
       pub auth_use_cases: Arc<AuthUseCases>,
       pub jwt_secret: String,
   }

Routes Configuration
^^^^^^^^^^^^^^^^^^^^

.. code-block:: rust

   pub fn configure_routes(cfg: &mut web::ServiceConfig) {
       cfg
           .service(
               web::scope("/api/v1")
                   // Health
                   .route("/health", web::get().to(health_check))

                   // Auth
                   .service(
                       web::scope("/auth")
                           .route("/login", web::post().to(login))
                           .route("/refresh", web::post().to(refresh_token))
                           .route("/me", web::get().to(get_current_user))
                   )

                   // Buildings (protected)
                   .service(
                       web::scope("/buildings")
                           .wrap(AuthMiddleware)  // JWT required
                           .route("", web::get().to(list_buildings))
                           .route("", web::post().to(create_building))
                           .route("/{id}", web::get().to(get_building))
                           .route("/{id}", web::put().to(update_building))
                           .route("/{id}", web::delete().to(delete_building))
                           .route("/{id}/units", web::get().to(list_building_units))
                           .route("/{id}/expenses", web::get().to(list_building_expenses))
                   )

                   // Units
                   .service(
                       web::scope("/units")
                           .wrap(AuthMiddleware)
                           .route("", web::get().to(list_units))
                           .route("", web::post().to(create_unit))
                           .route("/{id}", web::get().to(get_unit))
                           .route("/{id}", web::put().to(update_unit))
                           .route("/{id}", web::delete().to(delete_unit))
                           .route("/{id}/assign-owner/{owner_id}", web::put().to(assign_owner))
                   )

                   // Owners, Expenses, Meetings, Documents...
           );
   }

HTTP Handlers
^^^^^^^^^^^^^

Traitement requêtes HTTP et réponses.

.. code-block:: rust

   #[derive(Deserialize)]
   pub struct PaginationParams {
       pub page: Option<i64>,
       pub per_page: Option<i64>,
   }

   pub async fn list_buildings(
       state: web::Data<AppState>,
       query: web::Query<PaginationParams>,
       user: AuthenticatedUser,  // Extrait du JWT middleware
   ) -> Result<HttpResponse, actix_web::Error> {
       let page = query.page.unwrap_or(1);
       let per_page = query.per_page.unwrap_or(20).min(100);  // Max 100

       let result = state.building_use_cases
           .list_buildings_paginated(user.organization_id, page, per_page)
           .await
           .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

       Ok(HttpResponse::Ok().json(result))
   }

   pub async fn create_building(
       state: web::Data<AppState>,
       dto: web::Json<BuildingDto>,
       user: AuthenticatedUser,
   ) -> Result<HttpResponse, actix_web::Error> {
       // Vérifier permissions (role = syndic)
       if user.role != UserRole::Syndic {
           return Err(actix_web::error::ErrorForbidden("Insufficient permissions"));
       }

       let result = state.building_use_cases
           .create_building(user.organization_id, dto.into_inner())
           .await
           .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

       Ok(HttpResponse::Created().json(result))
   }

   pub async fn get_building(
       state: web::Data<AppState>,
       path: web::Path<Uuid>,
       user: AuthenticatedUser,
   ) -> Result<HttpResponse, actix_web::Error> {
       let building_id = path.into_inner();

       let result = state.building_use_cases
           .get_building(building_id)
           .await
           .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

       match result {
           Some(building) => Ok(HttpResponse::Ok().json(building)),
           None => Err(actix_web::error::ErrorNotFound("Building not found")),
       }
   }

   pub async fn update_building(
       state: web::Data<AppState>,
       path: web::Path<Uuid>,
       dto: web::Json<BuildingDto>,
       user: AuthenticatedUser,
   ) -> Result<HttpResponse, actix_web::Error> {
       let building_id = path.into_inner();

       let result = state.building_use_cases
           .update_building(building_id, dto.into_inner())
           .await
           .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

       Ok(HttpResponse::Ok().json(result))
   }

   pub async fn delete_building(
       state: web::Data<AppState>,
       path: web::Path<Uuid>,
       user: AuthenticatedUser,
   ) -> Result<HttpResponse, actix_web::Error> {
       let building_id = path.into_inner();

       state.building_use_cases
           .delete_building(building_id)
           .await
           .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

       Ok(HttpResponse::NoContent().finish())
   }

Middleware
----------

Auth Middleware (JWT)
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: rust

   pub struct AuthMiddleware;

   impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
   where
       S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
       S::Future: 'static,
       B: 'static,
   {
       type Response = ServiceResponse<B>;
       type Error = Error;
       type Transform = AuthMiddlewareService<S>;
       type InitError = ();
       type Future = Ready<Result<Self::Transform, Self::InitError>>;

       fn new_transform(&self, service: S) -> Self::Future {
           ready(Ok(AuthMiddlewareService { service }))
       }
   }

   pub async fn extract_jwt_claims(req: &HttpRequest) -> Result<Claims, actix_web::Error> {
       let auth_header = req.headers()
           .get("Authorization")
           .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing Authorization header"))?
           .to_str()
           .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid Authorization header"))?;

       if !auth_header.starts_with("Bearer ") {
           return Err(actix_web::error::ErrorUnauthorized("Invalid Authorization format"));
       }

       let token = &auth_header[7..];

       decode_jwt_token(token, &jwt_secret)
           .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))
   }

CORS Middleware
^^^^^^^^^^^^^^^

.. code-block:: rust

   use actix_cors::Cors;
   use actix_web::http::header;

   pub fn configure_cors() -> Cors {
       Cors::default()
           .allowed_origin("https://koprogo.com")
           .allowed_origin("http://localhost:3000")  // Dev
           .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
           .allowed_headers(vec![
               header::AUTHORIZATION,
               header::ACCEPT,
               header::CONTENT_TYPE,
               HeaderName::from_static("accept-language"),
           ])
           .max_age(3600)
   }

Error Handling
--------------

Conversion erreurs métier → HTTP status.

.. code-block:: rust

   impl From<AppError> for actix_web::Error {
       fn from(error: AppError) -> Self {
           match error {
               AppError::NotFound(msg) => actix_web::error::ErrorNotFound(msg),
               AppError::ValidationError(msg) => actix_web::error::ErrorBadRequest(msg),
               AppError::Unauthorized(msg) => actix_web::error::ErrorUnauthorized(msg),
               AppError::InternalError(msg) => actix_web::error::ErrorInternalServerError(msg),
           }
       }
   }

Tests Infrastructure
--------------------

**Tests d'Intégration** avec Testcontainers :

.. code-block:: rust

   #[tokio::test]
   async fn test_create_building_endpoint() {
       // Démarrer PostgreSQL testcontainer
       let postgres = PostgresContainer::default();
       let pool = create_test_pool(&postgres).await;

       // Run migrations
       sqlx::migrate!().run(&pool).await.unwrap();

       // Créer app
       let app = test::init_service(
           App::new()
               .app_data(web::Data::new(AppState { ... }))
               .configure(configure_routes)
       ).await;

       // Test POST /buildings
       let req = test::TestRequest::post()
           .uri("/api/v1/buildings")
           .set_json(&building_dto)
           .insert_header(("Authorization", format!("Bearer {}", token)))
           .to_request();

       let resp = test::call_service(&app, req).await;
       assert_eq!(resp.status(), 201);
   }

Dépendances
-----------

.. code-block:: toml

   [dependencies]
   # Web framework
   actix-web = "4.9"
   actix-cors = "0.7"

   # Database
   sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

   # Async
   tokio = { version = "1.41", features = ["full"] }
   async-trait = "0.1"

   # Tests
   testcontainers = "0.23"

Références
----------

- Actix-web Docs : https://actix.rs/docs/
- SQLx Docs : https://docs.rs/sqlx/
- Testcontainers Docs : https://docs.rs/testcontainers/

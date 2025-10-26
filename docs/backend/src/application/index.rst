Application - Couche Application (Use Cases)
=============================================

La couche Application orchestre la logique métier en utilisant les entités du Domain et les repositories de l'Infrastructure.

**Principe** : Application dépend UNIQUEMENT de Domain. Infrastructure implémente les ports définis ici.

Structure
---------

.. code-block:: text

   application/
   ├── dto/               # Data Transfer Objects (API contracts)
   │   ├── auth_dto.rs
   │   ├── building_dto.rs
   │   ├── unit_dto.rs
   │   ├── owner_dto.rs
   │   ├── expense_dto.rs
   │   ├── meeting_dto.rs
   │   ├── document_dto.rs
   │   ├── pcn_dto.rs
   │   └── pagination.rs
   ├── ports/             # Interfaces (Traits) repositories
   │   ├── user_repository.rs
   │   ├── organization_repository.rs
   │   ├── building_repository.rs
   │   ├── unit_repository.rs
   │   ├── owner_repository.rs
   │   ├── expense_repository.rs
   │   ├── meeting_repository.rs
   │   └── document_repository.rs
   └── use_cases/         # Orchestration logique métier
       ├── auth_use_cases.rs
       ├── building_use_cases.rs
       ├── unit_use_cases.rs
       ├── owner_use_cases.rs
       └── expense_use_cases.rs

DTOs (Data Transfer Objects)
-----------------------------

Contrats de données pour l'API REST.

**Responsabilités** :

- Sérialisation/désérialisation JSON
- Validation des entrées utilisateur
- Transformation Domain ↔ DTO

**Pattern** :

.. code-block:: rust

   #[derive(Debug, Serialize, Deserialize)]
   pub struct BuildingDto {
       pub id: Option<Uuid>,
       pub name: String,
       pub address: String,
       pub city: String,
       pub postal_code: String,
       pub country: String,
       pub total_units: i32,
       pub construction_year: Option<i32>,
   }

   impl From<Building> for BuildingDto {
       fn from(building: Building) -> Self {
           Self {
               id: Some(building.id),
               name: building.name,
               address: building.address,
               city: building.city,
               postal_code: building.postal_code,
               country: building.country,
               total_units: building.total_units,
               construction_year: building.construction_year,
           }
       }
   }

**Pagination** :

.. code-block:: rust

   #[derive(Debug, Serialize)]
   pub struct PageResponse<T> {
       pub data: Vec<T>,
       pub pagination: PaginationMeta,
   }

   #[derive(Debug, Serialize)]
   pub struct PaginationMeta {
       pub current_page: i64,
       pub per_page: i64,
       pub total_items: i64,
       pub total_pages: i64,
       pub has_next: bool,
       pub has_previous: bool,
   }

Ports (Interfaces Repositories)
--------------------------------

Traits définissant les opérations de persistance.

**Pattern Repository** :

.. code-block:: rust

   #[async_trait]
   pub trait BuildingRepository: Send + Sync {
       async fn create(&self, building: &Building) -> Result<Building, String>;
       async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
       async fn find_all_paginated(
           &self,
           organization_id: Uuid,
           page: i64,
           per_page: i64
       ) -> Result<PageResponse<Building>, String>;
       async fn update(&self, building: &Building) -> Result<Building, String>;
       async fn delete(&self, id: Uuid) -> Result<(), String>;
   }

**Pourquoi Traits ?** :

- ✅ Testabilité : Mock repositories pour tests
- ✅ Flexibilité : Changer implémentation (PostgreSQL → ScyllaDB)
- ✅ Inversion de dépendance : Application ne connaît pas l'implémentation

Use Cases (Cas d'Usage)
------------------------

Orchestration de la logique métier.

**Responsabilités** :

- Validation données entrée
- Orchestration appels Domain + Repositories
- Transformation Domain ↔ DTO
- Gestion erreurs métier

**Pattern** :

.. code-block:: rust

   pub struct BuildingUseCases {
       repository: Arc<dyn BuildingRepository>,
   }

   impl BuildingUseCases {
       pub fn new(repository: Arc<dyn BuildingRepository>) -> Self {
           Self { repository }
       }

       pub async fn create_building(
           &self,
           organization_id: Uuid,
           dto: BuildingDto
       ) -> Result<BuildingDto, String> {
           // 1. Créer entité domaine (avec validation)
           let building = Building::new(
               dto.name,
               dto.address,
               dto.city,
               dto.postal_code,
               dto.country,
               dto.total_units,
               dto.construction_year,
           )?;

           // 2. Persister via repository
           let saved_building = self.repository.create(&building).await?;

           // 3. Retourner DTO
           Ok(BuildingDto::from(saved_building))
       }

       pub async fn get_building(
           &self,
           id: Uuid
       ) -> Result<Option<BuildingDto>, String> {
           let building = self.repository.find_by_id(id).await?;
           Ok(building.map(BuildingDto::from))
       }

       pub async fn list_buildings_paginated(
           &self,
           organization_id: Uuid,
           page: i64,
           per_page: i64
       ) -> Result<PageResponse<BuildingDto>, String> {
           let page_response = self.repository
               .find_all_paginated(organization_id, page, per_page)
               .await?;

           Ok(PageResponse {
               data: page_response.data
                   .into_iter()
                   .map(BuildingDto::from)
                   .collect(),
               pagination: page_response.pagination,
           })
       }

       pub async fn update_building(
           &self,
           id: Uuid,
           dto: BuildingDto
       ) -> Result<BuildingDto, String> {
           // 1. Récupérer entité existante
           let mut building = self.repository
               .find_by_id(id)
               .await?
               .ok_or_else(|| "Building not found".to_string())?;

           // 2. Mettre à jour via méthode domaine
           building.update_info(
               dto.name,
               dto.address,
               dto.city,
               dto.postal_code,
           );

           // 3. Persister
           let updated_building = self.repository.update(&building).await?;

           Ok(BuildingDto::from(updated_building))
       }

       pub async fn delete_building(&self, id: Uuid) -> Result<(), String> {
           self.repository.delete(id).await
       }
   }

Authentification Use Case
--------------------------

**AuthUseCases** :

.. code-block:: rust

   pub struct AuthUseCases {
       user_repository: Arc<dyn UserRepository>,
       jwt_secret: String,
   }

   impl AuthUseCases {
       pub async fn login(
           &self,
           email: String,
           password: String
       ) -> Result<LoginResponseDto, String> {
           // 1. Trouver utilisateur
           let user = self.user_repository
               .find_by_email(&email)
               .await?
               .ok_or_else(|| "Invalid credentials".to_string())?;

           // 2. Vérifier mot de passe (bcrypt)
           if !verify_password(&password, &user.password_hash) {
               return Err("Invalid credentials".to_string());
           }

           // 3. Générer JWT token
           let token = generate_jwt_token(&user, &self.jwt_secret)?;

           Ok(LoginResponseDto {
               token,
               user: UserDto::from(user),
           })
       }

       pub async fn refresh_token(
           &self,
           refresh_token: String
       ) -> Result<TokenDto, String> {
           // Logique refresh token
       }
   }

Gestion Erreurs
---------------

**Types d'Erreurs** :

.. code-block:: rust

   pub enum AppError {
       NotFound(String),
       ValidationError(String),
       Unauthorized(String),
       InternalError(String),
   }

   impl From<AppError> for String {
       fn from(error: AppError) -> Self {
           match error {
               AppError::NotFound(msg) => msg,
               AppError::ValidationError(msg) => msg,
               AppError::Unauthorized(msg) => msg,
               AppError::InternalError(msg) => msg,
           }
       }
   }

Tests Use Cases
---------------

**Mock Repositories** :

.. code-block:: rust

   #[cfg(test)]
   mod tests {
       use super::*;
       use mockall::predicate::*;
       use mockall::mock;

       mock! {
           BuildingRepo {}

           #[async_trait]
           impl BuildingRepository for BuildingRepo {
               async fn create(&self, building: &Building) -> Result<Building, String>;
               async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
               // ...
           }
       }

       #[tokio::test]
       async fn test_create_building_success() {
           let mut mock_repo = MockBuildingRepo::new();

           mock_repo
               .expect_create()
               .times(1)
               .returning(|b| Ok(b.clone()));

           let use_cases = BuildingUseCases::new(Arc::new(mock_repo));

           let dto = BuildingDto {
               id: None,
               name: "Test Building".to_string(),
               address: "123 Main St".to_string(),
               city: "Paris".to_string(),
               postal_code: "75001".to_string(),
               country: "France".to_string(),
               total_units: 10,
               construction_year: None,
           };

           let organization_id = Uuid::new_v4();
           let result = use_cases.create_building(organization_id, dto).await;

           assert!(result.is_ok());
       }
   }

Flux de Données
---------------

.. code-block:: text

   HTTP Request (JSON)
        ↓
   Handler (Infrastructure/Web)
        ↓
   DTO (Application)
        ↓
   Use Case (Application)
        ↓
   Entity (Domain) ← Validation métier
        ↓
   Repository (Port trait)
        ↓
   Repository Impl (Infrastructure/Database)
        ↓
   PostgreSQL
        ↓
   Entity (Domain)
        ↓
   DTO (Application)
        ↓
   JSON Response

Dépendances
-----------

.. code-block:: toml

   [dependencies]
   # Domain
   uuid = { version = "1.11", features = ["v4", "serde"] }
   chrono = { version = "0.4", features = ["serde"] }
   serde = { version = "1.0", features = ["derive"] }

   # Async
   async-trait = "0.1"
   tokio = { version = "1.41", features = ["full"] }

   # Tests
   mockall = "0.13"  # Mocking repositories

Références
----------

- Clean Architecture (Robert C. Martin)
- Use Case Driven Development
- Repository Pattern (Martin Fowler)

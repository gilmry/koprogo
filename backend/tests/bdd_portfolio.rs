//! BDD Portfolio — Story 2.1 (Slice 2 Refonte UX multi-rôle ACP).
//!
//! Harness Cucumber dédié à `tests/features/portfolio.feature` (4-cat).
//! Spinne un Postgres testcontainer, exerce `PortfolioUseCases` via les
//! step definitions ci-dessous.
//!
//! Inspiration : `backend/tests/bdd_acp.rs` (Story 1.1).
//!
//! On utilise des `User` réels (Postgres) + des `Building` réels (insertion
//! brute via sqlx pour éviter la dépendance au `PostgresBuildingRepository`
//! qui réfère encore `organization_id` — la story 1.3 du refacto building
//! traitera ça séparément).

use async_trait::async_trait;
use cucumber::{given, then, when, World};
use koprogo_api::application::dto::{
    AddBuildingDto, BuildingFilters, CreatePortfolioDto, PageRequest, SharePortfolioDto,
};
use koprogo_api::application::error::AppError;
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{PortfolioCaller, PortfolioUseCases};
use koprogo_api::domain::entities::{Building, BuildingMetrics};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresPortfolioRepository, PostgresUserRepository,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(World)]
#[world(init = Self::new)]
pub struct PortfolioWorld {
    use_cases: Option<Arc<PortfolioUseCases>>,
    pool: Option<sqlx::PgPool>,
    _container: Option<ContainerAsync<Postgres>>,
    users: HashMap<String, Uuid>,
    portfolios: HashMap<String, Uuid>,
    buildings: HashMap<String, Uuid>,
    last_portfolio_id: Option<Uuid>,
    last_caller: Option<PortfolioCaller>,
    last_error: Option<AppError>,
    last_list: Option<Vec<koprogo_api::application::dto::PortfolioResponseDto>>,
    last_buildings: Option<Vec<koprogo_api::application::dto::PortfolioBuildingResponseDto>>,
}

impl std::fmt::Debug for PortfolioWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PortfolioWorld")
            .field("users", &self.users)
            .field("portfolios", &self.portfolios)
            .field("buildings", &self.buildings)
            .field("last_error", &self.last_error)
            .finish()
    }
}

impl PortfolioWorld {
    async fn new() -> Self {
        Self {
            use_cases: None,
            pool: None,
            _container: None,
            users: HashMap::new(),
            portfolios: HashMap::new(),
            buildings: HashMap::new(),
            last_portfolio_id: None,
            last_caller: None,
            last_error: None,
            last_list: None,
            last_buildings: None,
        }
    }

    async fn setup(&mut self) {
        let mut attempts = 0;
        let container = loop {
            match Postgres::default().start().await {
                Ok(c) => break c,
                Err(e) if attempts < 3 => {
                    attempts += 1;
                    eprintln!("postgres start retry {}: {}", attempts, e);
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => panic!("postgres start failed: {}", e),
            }
        };
        let host_port = container.get_host_port_ipv4(5432).await.expect("port");
        let host = container.get_host().await.expect("host");
        let connection_string = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            host, host_port
        );
        let pool = create_pool(&connection_string).await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        let portfolio_repo = Arc::new(PostgresPortfolioRepository::new(pool.clone()));
        // BuildingRepository stub : `PostgresBuildingRepository` actuel
        // référence `organization_id` (column DROP par migration 040000).
        // Story 1.3 du refacto building s'en charge ; ici on n'a besoin que
        // de `find_by_id` pour AC @negative. On utilise un adapter local
        // qui parle SQL minimaliste (uniquement les colonnes communes
        // pré/post migration).
        let building_repo = Arc::new(SqlxBuildingExistenceRepo::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let uc = PortfolioUseCases::new(portfolio_repo, building_repo, user_repo);
        self.use_cases = Some(Arc::new(uc));
        self.pool = Some(pool);
        self._container = Some(container);
    }

    async fn ensure_setup(&mut self) {
        if self.use_cases.is_none() {
            self.setup().await;
        }
    }

    /// Insert a user directly via sqlx (avoid full UserRepository plumbing).
    async fn insert_user(&self, email: &str) -> Uuid {
        let pool = self.pool.as_ref().expect("pool initialised");
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(email)
        .bind("hash-not-used-in-portfolio-tests")
        .bind("First")
        .bind("Last")
        .bind("syndic")
        .execute(pool)
        .await
        .expect("insert user");
        id
    }

    /// Insert a building directly via sqlx (bypass building_repository_impl
    /// which still depends on organization_id while migration is in-flight).
    /// We create a throwaway organization to satisfy a possible FK.
    async fn insert_building(&self, name: &str) -> Uuid {
        let pool = self.pool.as_ref().expect("pool initialised");
        let org_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO organizations
              (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'starter', 1, 3, true, NOW(), NOW())
            ON CONFLICT (slug) DO NOTHING
            "#,
        )
        .bind(org_id)
        .bind(format!("Org for {}", name))
        .bind(format!("org-{}", Uuid::new_v4().simple()))
        .bind(format!("org-{}@example.test", Uuid::new_v4().simple()))
        .execute(pool)
        .await
        .expect("insert org");

        // Pour rester compatible que la migration 040000 ait été appliquée
        // ou pas, on insère uniquement les colonnes communes ; les colonnes
        // additionnelles (acp_id, organization_id) sont rattachées via ALTER
        // selon les migrations existantes — on essaie d'abord avec acp_id
        // (post-migration), fallback sans contrainte sinon.
        let bid = Uuid::new_v4();
        // Try with acp_id (post Story 1.2/1.4 NOT NULL state).
        let acp_id = Uuid::new_v4();
        let _ = sqlx::query(
            r#"
            INSERT INTO acps (
                id, organization_id, name, slug, legal_status,
                address_street, address_postal_code, address_city,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, 'copropriete_belge', 'Rue X 1', '1000', 'Bruxelles', NOW(), NOW())
            "#,
        )
        .bind(acp_id)
        .bind(org_id)
        .bind(format!("ACP for {}", name))
        .bind(format!("acp-{}", Uuid::new_v4().simple()))
        .execute(pool)
        .await;

        // Insertion building "minimaliste" : on tente le shape post-040000
        // (acp_id NOT NULL, no organization_id) puis fallback sur l'ancien
        // shape pre-040000 si la colonne organization_id existe encore.
        let res = sqlx::query(
            r#"
            INSERT INTO buildings (id, acp_id, name, address, city, postal_code, country, total_units, created_at, updated_at)
            VALUES ($1, $2, $3, 'Rue X 1', 'Bruxelles', '1000', 'BE', 10, NOW(), NOW())
            "#,
        )
        .bind(bid)
        .bind(acp_id)
        .bind(name)
        .execute(pool)
        .await;
        if res.is_ok() {
            return bid;
        }
        // Fallback : ancien shape (organization_id encore présent, acp_id
        // existe ou pas).
        let res2 = sqlx::query(
            r#"
            INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, created_at, updated_at)
            VALUES ($1, $2, $3, 'Rue X 1', 'Bruxelles', '1000', 'BE', 10, NOW(), NOW())
            "#,
        )
        .bind(bid)
        .bind(org_id)
        .bind(name)
        .execute(pool)
        .await;
        res2.expect("insert building (fallback shape)");
        bid
    }
}

// ============================================================================
// Background / Given
// ============================================================================

#[given("a portfolio management system")]
async fn given_system(world: &mut PortfolioWorld) {
    world.ensure_setup().await;
}

#[given(regex = r#"^an existing user "([^"]+)"$"#)]
async fn given_user(world: &mut PortfolioWorld, email: String) {
    world.ensure_setup().await;
    let id = world.insert_user(&email).await;
    world.users.insert(email, id);
}

#[given(regex = r#"^an existing building "([^"]+)"$"#)]
async fn given_building(world: &mut PortfolioWorld, name: String) {
    world.ensure_setup().await;
    let id = world.insert_building(&name).await;
    world.buildings.insert(name, id);
}

#[given(regex = r#"^a portfolio "([^"]+)" owned by "([^"]+)"$"#)]
async fn given_portfolio(world: &mut PortfolioWorld, portfolio_name: String, owner_email: String) {
    world.ensure_setup().await;
    let user_id = *world.users.get(&owner_email).expect("user must exist");
    let uc = world.use_cases.as_ref().unwrap().clone();
    let resp = uc
        .create_portfolio(
            &PortfolioCaller { user_id },
            CreatePortfolioDto {
                name: portfolio_name.clone(),
                description: None,
            },
        )
        .await
        .expect("create portfolio");
    let pid = Uuid::parse_str(&resp.id).unwrap();
    world.portfolios.insert(portfolio_name, pid);
    world.last_portfolio_id = Some(pid);
    world.last_caller = Some(PortfolioCaller { user_id });
}

#[given(regex = r#"^the portfolio is shared with "([^"]+)"$"#)]
async fn given_portfolio_shared(world: &mut PortfolioWorld, email: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let caller = world.last_caller.expect("caller set");
    let target_id = *world.users.get(&email).expect("user must exist");
    let uc = world.use_cases.as_ref().unwrap().clone();
    uc.share_with(
        &caller,
        pid,
        SharePortfolioDto {
            shared_with_user_id: target_id.to_string(),
            can_edit: false,
        },
    )
    .await
    .expect("share");
}

// ============================================================================
// When
// ============================================================================

#[when(regex = r#"^that user creates a portfolio named "([^"]*)"$"#)]
async fn when_user_creates(world: &mut PortfolioWorld, name: String) {
    let user_id = *world
        .users
        .values()
        .next()
        .expect("at least one user given");
    let caller = PortfolioCaller { user_id };
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .create_portfolio(
            &caller,
            CreatePortfolioDto {
                name,
                description: None,
            },
        )
        .await
    {
        Ok(r) => {
            let pid = Uuid::parse_str(&r.id).unwrap();
            world.portfolios.insert(r.name.clone(), pid);
            world.last_portfolio_id = Some(pid);
            world.last_caller = Some(caller);
            world.last_error = None;
        }
        Err(e) => {
            world.last_error = Some(e);
        }
    }
}

#[when(regex = r#"^that user adds "([^"]+)" as favorite to portfolio$"#)]
async fn when_user_adds_favorite(world: &mut PortfolioWorld, building_name: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let caller = world.last_caller.expect("caller set");
    let bid = *world
        .buildings
        .get(&building_name)
        .expect("building exists");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .add_building(
            &caller,
            pid,
            AddBuildingDto {
                building_id: bid.to_string(),
                is_favorite: true,
            },
        )
        .await
    {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

#[when(regex = r#"^that user adds "([^"]+)" as normal to portfolio$"#)]
async fn when_user_adds_normal(world: &mut PortfolioWorld, building_name: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let caller = world.last_caller.expect("caller set");
    let bid = *world
        .buildings
        .get(&building_name)
        .expect("building exists");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .add_building(
            &caller,
            pid,
            AddBuildingDto {
                building_id: bid.to_string(),
                is_favorite: false,
            },
        )
        .await
    {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

#[when("that user adds an unknown building to portfolio")]
async fn when_user_adds_unknown(world: &mut PortfolioWorld) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let caller = world.last_caller.expect("caller set");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .add_building(
            &caller,
            pid,
            AddBuildingDto {
                building_id: Uuid::new_v4().to_string(),
                is_favorite: false,
            },
        )
        .await
    {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

#[when("that user lists the buildings of portfolio")]
async fn when_user_lists_buildings(world: &mut PortfolioWorld) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let caller = world.last_caller.expect("caller set");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc.list_buildings(&caller, pid).await {
        Ok(list) => {
            world.last_buildings = Some(list);
            world.last_error = None;
        }
        Err(e) => world.last_error = Some(e),
    }
}

#[when("that user lists portfolios")]
async fn when_user_lists_portfolios(world: &mut PortfolioWorld) {
    let user_id = *world
        .users
        .values()
        .next()
        .expect("at least one user given");
    let caller = PortfolioCaller { user_id };
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc.list_portfolios(&caller).await {
        Ok(list) => {
            world.last_list = Some(list);
            world.last_error = None;
        }
        Err(e) => world.last_error = Some(e),
    }
}

#[when(regex = r#"^user "([^"]+)" tries to get that portfolio$"#)]
async fn when_user_tries_get(world: &mut PortfolioWorld, email: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let user_id = *world.users.get(&email).expect("user exists");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc.get_portfolio(&PortfolioCaller { user_id }, pid).await {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

#[when(regex = r#"^user "([^"]+)" tries to add "([^"]+)" to portfolio$"#)]
async fn when_user_tries_add(world: &mut PortfolioWorld, email: String, building_name: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    let user_id = *world.users.get(&email).expect("user exists");
    let bid = *world
        .buildings
        .get(&building_name)
        .expect("building exists");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .add_building(
            &PortfolioCaller { user_id },
            pid,
            AddBuildingDto {
                building_id: bid.to_string(),
                is_favorite: false,
            },
        )
        .await
    {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

#[when("that user gets a portfolio by an unknown id")]
async fn when_user_get_unknown(world: &mut PortfolioWorld) {
    let user_id = *world
        .users
        .values()
        .next()
        .expect("at least one user given");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc
        .get_portfolio(&PortfolioCaller { user_id }, Uuid::new_v4())
        .await
    {
        Ok(_) => world.last_error = None,
        Err(e) => world.last_error = Some(e),
    }
}

// ============================================================================
// Then
// ============================================================================

#[then("the portfolio should be persisted successfully")]
async fn then_persisted(world: &mut PortfolioWorld) {
    assert!(
        world.last_error.is_none(),
        "expected no error, got {:?}",
        world.last_error
    );
    assert!(world.last_portfolio_id.is_some());
}

#[then(regex = r#"^the portfolio name should be "([^"]+)"$"#)]
async fn then_name(world: &mut PortfolioWorld, expected: String) {
    let pid = world.last_portfolio_id.expect("portfolio set");
    assert!(world
        .portfolios
        .iter()
        .any(|(n, id)| *id == pid && *n == expected));
}

#[then(regex = r#"^the listing should contain (\d+) buildings$"#)]
async fn then_listing_count(world: &mut PortfolioWorld, n: usize) {
    let list = world.last_buildings.as_ref().unwrap();
    assert_eq!(
        list.len(),
        n,
        "expected {} buildings, got {}",
        n,
        list.len()
    );
}

#[then("the first building of the listing should be favorite")]
async fn then_first_is_favorite(world: &mut PortfolioWorld) {
    let list = world.last_buildings.as_ref().unwrap();
    assert!(
        !list.is_empty(),
        "listing is empty — cannot assert first is favorite"
    );
    assert!(list[0].is_favorite, "first building should be favorite");
}

#[then(regex = r#"^the portfolio list should contain (\d+) portfolios$"#)]
async fn then_portfolio_list_count(world: &mut PortfolioWorld, n: usize) {
    let list = world.last_list.as_ref().unwrap();
    assert_eq!(
        list.len(),
        n,
        "expected {} portfolios, got {}",
        n,
        list.len()
    );
}

#[then("the operation should fail with a forbidden error")]
async fn then_forbidden(world: &mut PortfolioWorld) {
    match world.last_error.as_ref() {
        Some(AppError::Forbidden(_)) => {}
        other => panic!("expected Forbidden, got {:?}", other),
    }
}

#[then("the operation should fail with a validation error")]
async fn then_validation(world: &mut PortfolioWorld) {
    match world.last_error.as_ref() {
        Some(AppError::Validation(_)) => {}
        other => panic!("expected Validation, got {:?}", other),
    }
}

#[then("the operation should fail with a not found error")]
async fn then_not_found(world: &mut PortfolioWorld) {
    match world.last_error.as_ref() {
        Some(AppError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

// ============================================================================
// SqlxBuildingExistenceRepo : adapter local pour ce harness BDD seulement.
// Le `PostgresBuildingRepository` actuel utilise du SQL qui réfère encore
// `organization_id` (dropped par 040000) — incompatible tant que Story 1.3
// (refacto building → acp_id) n'est pas fait. Pour ne pas bloquer Story 2.1
// sur une dette de slice 1, on implémente le strict minimum (`find_by_id`)
// avec un SELECT sur les colonnes communes pré/post migration.
// ============================================================================

struct SqlxBuildingExistenceRepo {
    pool: sqlx::PgPool,
}

impl SqlxBuildingExistenceRepo {
    fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BuildingRepository for SqlxBuildingExistenceRepo {
    async fn create(&self, _: &Building) -> Result<Building, String> {
        Err("not implemented for portfolio BDD harness".to_string())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String> {
        // Strict minimum : on vérifie l'existence + on retourne un Building
        // squelettique (les use-cases Portfolio n'utilisent QUE l'absence /
        // présence — aucun champ n'est lu).
        let row = sqlx::query("SELECT id, name FROM buildings WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(|r| {
            use sqlx::Row;
            Building {
                id: r.get("id"),
                organization_id: Uuid::nil(), // unused for portfolio AC
                name: r.get("name"),
                address: String::new(),
                city: String::new(),
                postal_code: String::new(),
                country: String::new(),
                total_units: 1,
                total_tantiemes: 1000,
                construction_year: None,
                syndic_name: None,
                syndic_email: None,
                syndic_phone: None,
                syndic_address: None,
                syndic_office_hours: None,
                syndic_emergency_contact: None,
                slug: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }))
    }

    async fn find_all(&self) -> Result<Vec<Building>, String> {
        Ok(Vec::new())
    }

    async fn find_all_paginated(
        &self,
        _page_request: &PageRequest,
        _filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String> {
        Ok((Vec::new(), 0))
    }

    async fn update(&self, _: &Building) -> Result<Building, String> {
        Err("not implemented for portfolio BDD harness".to_string())
    }

    async fn delete(&self, _: Uuid) -> Result<bool, String> {
        Ok(false)
    }

    async fn find_by_slug(&self, _: &str) -> Result<Option<Building>, String> {
        Ok(None)
    }

    async fn find_by_id_with_metrics(
        &self,
        _: Uuid,
    ) -> Result<Option<(Building, BuildingMetrics)>, String> {
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    use cucumber::writer::Stats as _;
    let writer = PortfolioWorld::cucumber()
        .run("tests/features/portfolio.feature")
        .await;
    if writer.execution_has_failed() {
        std::process::exit(1);
    }
}

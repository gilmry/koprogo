// BDD tests for Governance domain: resolutions, convocations, quotes, organizations, two_factor, public_syndic
// Step definitions will be implemented in Phase 2 (Tier 1 core)

use cucumber::{given, World};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{
    AuthUseCases, ConvocationUseCases, QuoteUseCases, ResolutionUseCases,
};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresConvocationRecipientRepository,
    PostgresConvocationRepository, PostgresMeetingRepository, PostgresOwnerRepository,
    PostgresQuoteRepository, PostgresRefreshTokenRepository, PostgresResolutionRepository,
    PostgresUserRepository, PostgresUserRoleRepository, PostgresVoteRepository,
};
use koprogo_api::infrastructure::pool::DbPool;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(World)]
#[world(init = Self::new)]
#[allow(dead_code)]
pub struct GovernanceWorld {
    // Infrastructure
    _container: Option<ContainerAsync<Postgres>>,
    pool: Option<DbPool>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,
    meeting_id: Option<Uuid>,

    // Use cases
    resolution_use_cases: Option<Arc<ResolutionUseCases>>,
    convocation_use_cases: Option<Arc<ConvocationUseCases>>,
    quote_use_cases: Option<Arc<QuoteUseCases>>,
    auth_use_cases: Option<Arc<AuthUseCases>>,

    // State
    last_result: Option<Result<String, String>>,
    last_resolution_id: Option<Uuid>,
    last_convocation_id: Option<Uuid>,
    last_quote_id: Option<Uuid>,

    // Owners for voting
    owner_alice_id: Option<Uuid>,
    owner_bob_id: Option<Uuid>,
    owner_charlie_id: Option<Uuid>,
}

impl std::fmt::Debug for GovernanceWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernanceWorld")
            .field("org_id", &self.org_id)
            .field("building_id", &self.building_id)
            .finish()
    }
}

impl GovernanceWorld {
    async fn new() -> Self {
        Self {
            _container: None,
            pool: None,
            org_id: None,
            building_id: None,
            meeting_id: None,
            resolution_use_cases: None,
            convocation_use_cases: None,
            quote_use_cases: None,
            auth_use_cases: None,
            last_result: None,
            last_resolution_id: None,
            last_convocation_id: None,
            last_quote_id: None,
            owner_alice_id: None,
            owner_bob_id: None,
            owner_charlie_id: None,
        }
    }

    async fn setup_database(&mut self) {
        let mut attempts = 0;
        let postgres_container = loop {
            match Postgres::default().start().await {
                Ok(container) => break container,
                Err(e) if attempts < 3 => {
                    attempts += 1;
                    eprintln!(
                        "Postgres container start failed (attempt {}): {}. Retrying...",
                        attempts, e
                    );
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => panic!("Failed to start postgres container: {}", e),
            }
        };

        let host_port = postgres_container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");

        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let pool = create_pool(&connection_string)
            .await
            .expect("Failed to create pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        self.pool = Some(pool.clone());

        // Create organization
        let org_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
               VALUES ($1, 'Governance BDD Org', 'gov-bdd', 'gov@bdd.com', 'starter', 10, 10, true, NOW(), NOW())"#
        )
        .bind(org_id)
        .execute(&pool)
        .await
        .expect("insert org");

        // Create building
        let building_repo: Arc<dyn BuildingRepository> =
            Arc::new(PostgresBuildingRepository::new(pool.clone()));
        {
            use koprogo_api::domain::entities::Building;
            let b = Building::new(
                org_id,
                "Residence Governance".to_string(),
                "1 Rue du Parlement".to_string(),
                "Bruxelles".to_string(),
                "1000".to_string(),
                "Belgique".to_string(),
                10,
                1000,
                Some(2000),
            )
            .unwrap();
            building_repo.create(&b).await.expect("create building");
            self.building_id = Some(b.id);
        }

        // Setup repositories
        let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
        let resolution_repo = Arc::new(PostgresResolutionRepository::new(pool.clone()));
        let vote_repo = Arc::new(PostgresVoteRepository::new(pool.clone()));
        let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
        let quote_repo = Arc::new(PostgresQuoteRepository::new(pool.clone()));
        let convocation_repo = Arc::new(PostgresConvocationRepository::new(pool.clone()));
        let convocation_recipient_repo =
            Arc::new(PostgresConvocationRecipientRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
        let user_role_repo: Arc<dyn koprogo_api::application::ports::UserRoleRepository> =
            Arc::new(PostgresUserRoleRepository::new(pool.clone()));

        // Setup use cases (2 args: resolution_repo, vote_repo)
        let resolution_use_cases = ResolutionUseCases::new(resolution_repo, vote_repo);
        // QuoteUseCases: 1 arg
        let quote_use_cases = QuoteUseCases::new(quote_repo);
        // ConvocationUseCases: 5 args (convocation, recipient, owner, building, meeting)
        let convocation_use_cases = ConvocationUseCases::new(
            convocation_repo,
            convocation_recipient_repo,
            owner_repo,
            building_repo,
            meeting_repo,
        );
        let auth_use_cases = AuthUseCases::new(
            user_repo,
            refresh_repo,
            user_role_repo,
            "test-secret-key-governance".to_string(),
        );

        self.resolution_use_cases = Some(Arc::new(resolution_use_cases));
        self.quote_use_cases = Some(Arc::new(quote_use_cases));
        self.convocation_use_cases = Some(Arc::new(convocation_use_cases));
        self.auth_use_cases = Some(Arc::new(auth_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
    }
}

// === COMMON GIVEN STEPS ===

#[given("the system is initialized")]
async fn given_system_initialized(world: &mut GovernanceWorld) {
    world.setup_database().await;
}

#[given(regex = r#"^an organization "([^"]*)" exists with id "([^"]*)"$"#)]
async fn given_org_exists(_world: &mut GovernanceWorld, _name: String, _id: String) {
    // Organization already created during setup_database
}

#[given(regex = r#"^a building "([^"]*)" exists in organization "([^"]*)"$"#)]
async fn given_building_exists(_world: &mut GovernanceWorld, _name: String, _org_id: String) {
    // Building already created during setup_database
}

#[given(regex = r#"^a meeting "([^"]*)" exists for the building$"#)]
async fn given_meeting_exists(world: &mut GovernanceWorld, title: String) {
    let pool = world.pool.as_ref().unwrap();
    let meeting_id = Uuid::new_v4();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();

    sqlx::query(
        r#"INSERT INTO meetings (id, organization_id, building_id, meeting_type, title, scheduled_date, location, status, created_at, updated_at)
           VALUES ($1, $2, $3, 'Ordinary', $4, NOW() + interval '30 days', 'Salle AG', 'Planned', NOW(), NOW())"#,
    )
    .bind(meeting_id)
    .bind(org_id)
    .bind(building_id)
    .bind(&title)
    .execute(pool)
    .await
    .expect("insert meeting");

    world.meeting_id = Some(meeting_id);
}

#[given(regex = r#"^an owner "([^"]*)" with (\d+) voting power \(tantiemes\) exists$"#)]
async fn given_owner_with_voting_power(
    world: &mut GovernanceWorld,
    name: String,
    _voting_power: i32,
) {
    let pool = world.pool.as_ref().unwrap();
    let owner_id = Uuid::new_v4();
    let org_id = world.org_id.unwrap();

    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, created_at, updated_at)
           VALUES ($1, $2, $3, 'BDD', $4, '+32123456789', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(&name)
    .bind(format!("{}@bdd-gov.be", name.to_lowercase()))
    .execute(pool)
    .await
    .expect("insert owner");

    match name.as_str() {
        "Alice" => world.owner_alice_id = Some(owner_id),
        "Bob" => world.owner_bob_id = Some(owner_id),
        "Charlie" => world.owner_charlie_id = Some(owner_id),
        _ => {}
    }
}

#[tokio::main]
async fn main() {
    GovernanceWorld::cucumber()
        .run_and_exit("tests/features/resolutions.feature")
        .await;
}

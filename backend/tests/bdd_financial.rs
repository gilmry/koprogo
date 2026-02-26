// BDD tests for Financial domain: payments, payment_methods, journal_entries,
// call_for_funds, owner_contributions, charge_distribution, dashboard
// Step definitions will be implemented in Phase 2-4

use cucumber::{given, World};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{
    CallForFundsUseCases, JournalEntryUseCases, OwnerContributionUseCases, PaymentMethodUseCases,
    PaymentUseCases,
};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresCallForFundsRepository,
    PostgresJournalEntryRepository, PostgresOwnerContributionRepository,
    PostgresPaymentMethodRepository, PostgresPaymentRepository, PostgresUnitOwnerRepository,
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
pub struct FinancialWorld {
    _container: Option<ContainerAsync<Postgres>>,
    pool: Option<DbPool>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,

    payment_use_cases: Option<Arc<PaymentUseCases>>,
    payment_method_use_cases: Option<Arc<PaymentMethodUseCases>>,
    journal_entry_use_cases: Option<Arc<JournalEntryUseCases>>,
    call_for_funds_use_cases: Option<Arc<CallForFundsUseCases>>,
    owner_contribution_use_cases: Option<Arc<OwnerContributionUseCases>>,

    last_result: Option<Result<String, String>>,
    last_payment_id: Option<Uuid>,
    last_owner_id: Option<Uuid>,
}

impl std::fmt::Debug for FinancialWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FinancialWorld")
            .field("org_id", &self.org_id)
            .finish()
    }
}

impl FinancialWorld {
    async fn new() -> Self {
        Self {
            _container: None,
            pool: None,
            org_id: None,
            building_id: None,
            payment_use_cases: None,
            payment_method_use_cases: None,
            journal_entry_use_cases: None,
            call_for_funds_use_cases: None,
            owner_contribution_use_cases: None,
            last_result: None,
            last_payment_id: None,
            last_owner_id: None,
        }
    }

    async fn setup_database(&mut self) {
        let mut attempts = 0;
        let postgres_container = loop {
            match Postgres::default().start().await {
                Ok(container) => break container,
                Err(e) if attempts < 3 => {
                    attempts += 1;
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

        let org_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
               VALUES ($1, 'Financial BDD Org', 'fin-bdd', 'fin@bdd.com', 'starter', 10, 10, true, NOW(), NOW())"#
        )
        .bind(org_id)
        .execute(&pool)
        .await
        .expect("insert org");

        let building_repo: Arc<dyn BuildingRepository> =
            Arc::new(PostgresBuildingRepository::new(pool.clone()));
        {
            use koprogo_api::domain::entities::Building;
            let b = Building::new(
                org_id,
                "Residence Financiere".to_string(),
                "1 Rue de la Bourse".to_string(),
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

        let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
        let payment_method_repo = Arc::new(PostgresPaymentMethodRepository::new(pool.clone()));
        let journal_entry_repo = Arc::new(PostgresJournalEntryRepository::new(pool.clone()));
        let call_for_funds_repo = Arc::new(PostgresCallForFundsRepository::new(pool.clone()));
        let owner_contribution_repo =
            Arc::new(PostgresOwnerContributionRepository::new(pool.clone()));
        let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));

        let payment_use_cases = PaymentUseCases::new(payment_repo, payment_method_repo.clone());
        let payment_method_use_cases = PaymentMethodUseCases::new(payment_method_repo);
        let journal_entry_use_cases = JournalEntryUseCases::new(journal_entry_repo);
        let call_for_funds_use_cases = CallForFundsUseCases::new(
            call_for_funds_repo,
            owner_contribution_repo.clone(),
            unit_owner_repo,
        );
        let owner_contribution_use_cases = OwnerContributionUseCases::new(owner_contribution_repo);

        self.payment_use_cases = Some(Arc::new(payment_use_cases));
        self.payment_method_use_cases = Some(Arc::new(payment_method_use_cases));
        self.journal_entry_use_cases = Some(Arc::new(journal_entry_use_cases));
        self.call_for_funds_use_cases = Some(Arc::new(call_for_funds_use_cases));
        self.owner_contribution_use_cases = Some(Arc::new(owner_contribution_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
    }
}

#[given("the system is initialized")]
async fn given_system_initialized(world: &mut FinancialWorld) {
    world.setup_database().await;
}

#[tokio::main]
async fn main() {
    FinancialWorld::cucumber()
        .run_and_exit("tests/features/payments.feature")
        .await;
}

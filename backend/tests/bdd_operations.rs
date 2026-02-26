// BDD tests for Operations domain: tickets, notifications, work_reports,
// technical_inspections, iot, energy_campaigns
// Step definitions will be implemented in Phase 2-4

use cucumber::{given, World};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{
    EnergyCampaignUseCases, IoTUseCases, NotificationUseCases, TechnicalInspectionUseCases,
    TicketUseCases, WorkReportUseCases,
};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresEnergyBillUploadRepository,
    PostgresEnergyCampaignRepository, PostgresIoTRepository,
    PostgresNotificationPreferenceRepository, PostgresNotificationRepository,
    PostgresTechnicalInspectionRepository, PostgresTicketRepository, PostgresWorkReportRepository,
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
pub struct OperationsWorld {
    _container: Option<ContainerAsync<Postgres>>,
    pool: Option<DbPool>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,

    ticket_use_cases: Option<Arc<TicketUseCases>>,
    notification_use_cases: Option<Arc<NotificationUseCases>>,
    work_report_use_cases: Option<Arc<WorkReportUseCases>>,
    technical_inspection_use_cases: Option<Arc<TechnicalInspectionUseCases>>,
    iot_use_cases: Option<Arc<IoTUseCases>>,
    energy_campaign_use_cases: Option<Arc<EnergyCampaignUseCases>>,

    last_result: Option<Result<String, String>>,
    last_ticket_id: Option<Uuid>,
    last_notification_id: Option<Uuid>,
    last_user_id: Option<Uuid>,
}

impl std::fmt::Debug for OperationsWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationsWorld")
            .field("org_id", &self.org_id)
            .finish()
    }
}

impl OperationsWorld {
    async fn new() -> Self {
        Self {
            _container: None,
            pool: None,
            org_id: None,
            building_id: None,
            ticket_use_cases: None,
            notification_use_cases: None,
            work_report_use_cases: None,
            technical_inspection_use_cases: None,
            iot_use_cases: None,
            energy_campaign_use_cases: None,
            last_result: None,
            last_ticket_id: None,
            last_notification_id: None,
            last_user_id: None,
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
               VALUES ($1, 'Operations BDD Org', 'ops-bdd', 'ops@bdd.com', 'starter', 10, 10, true, NOW(), NOW())"#
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
                "Residence Operations".to_string(),
                "1 Rue des Travaux".to_string(),
                "Liege".to_string(),
                "4000".to_string(),
                "Belgique".to_string(),
                10,
                1000,
                Some(2000),
            )
            .unwrap();
            building_repo.create(&b).await.expect("create building");
            self.building_id = Some(b.id);
        }

        let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
        let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
        let notification_pref_repo =
            Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
        let work_report_repo = Arc::new(PostgresWorkReportRepository::new(pool.clone()));
        let inspection_repo = Arc::new(PostgresTechnicalInspectionRepository::new(pool.clone()));
        let iot_repo = Arc::new(PostgresIoTRepository::new(pool.clone()));
        let energy_campaign_repo = Arc::new(PostgresEnergyCampaignRepository::new(pool.clone()));
        let energy_bill_repo = Arc::new(PostgresEnergyBillUploadRepository::new(pool.clone()));

        let ticket_use_cases = TicketUseCases::new(ticket_repo);
        let notification_use_cases =
            NotificationUseCases::new(notification_repo, notification_pref_repo);
        let work_report_use_cases = WorkReportUseCases::new(work_report_repo);
        let technical_inspection_use_cases = TechnicalInspectionUseCases::new(inspection_repo);
        let iot_use_cases = IoTUseCases::new(iot_repo);
        // EnergyCampaignUseCases: campaign_repo + bill_upload_repo + building_repo
        let energy_campaign_use_cases =
            EnergyCampaignUseCases::new(energy_campaign_repo, energy_bill_repo, building_repo);

        self.ticket_use_cases = Some(Arc::new(ticket_use_cases));
        self.notification_use_cases = Some(Arc::new(notification_use_cases));
        self.work_report_use_cases = Some(Arc::new(work_report_use_cases));
        self.technical_inspection_use_cases = Some(Arc::new(technical_inspection_use_cases));
        self.iot_use_cases = Some(Arc::new(iot_use_cases));
        self.energy_campaign_use_cases = Some(Arc::new(energy_campaign_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
    }
}

#[given("the system is initialized")]
async fn given_system_initialized(world: &mut OperationsWorld) {
    world.setup_database().await;
}

#[tokio::main]
async fn main() {
    OperationsWorld::cucumber()
        .run_and_exit("tests/features/tickets.feature")
        .await;
}

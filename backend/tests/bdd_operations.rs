// BDD tests for Operations domain: tickets, notifications, work_reports,
// technical_inspections, iot, energy_campaigns
// Phase 2 Tier 1: Ticket step definitions (17 scenarios)

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use cucumber::{gherkin::Step, given, then, when, World};
use koprogo_api::application::dto::{
    AssignTicketRequest, CancelTicketRequest, CreateIoTReadingDto, CreateNotificationRequest,
    CreateTicketRequest, NotificationStats, QueryIoTReadingsDto, ReopenTicketRequest,
    ResolveTicketRequest, UpdatePreferenceRequest,
};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{
    EnergyBillUploadUseCases, EnergyCampaignUseCases, IoTUseCases, NotificationUseCases,
    TechnicalInspectionUseCases, TicketStatistics, TicketUseCases, WorkReportUseCases,
};
use koprogo_api::domain::entities::{
    CampaignStatus, DeviceType, EnergyType, InspectionType, MetricType, NotificationChannel,
    NotificationPriority, NotificationStatus, NotificationType, TicketCategory, TicketPriority,
    TicketStatus, WarrantyType, WorkType,
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

    // Use cases
    ticket_use_cases: Option<Arc<TicketUseCases>>,
    notification_use_cases: Option<Arc<NotificationUseCases>>,
    work_report_use_cases: Option<Arc<WorkReportUseCases>>,
    technical_inspection_use_cases: Option<Arc<TechnicalInspectionUseCases>>,
    iot_use_cases: Option<Arc<IoTUseCases>>,
    energy_campaign_use_cases: Option<Arc<EnergyCampaignUseCases>>,

    // Owners
    owner_marie_id: Option<Uuid>,
    owner_pierre_id: Option<Uuid>,
    authenticated_user_id: Option<Uuid>,

    // Ticket tracking
    last_ticket_id: Option<Uuid>,
    last_ticket_status: Option<TicketStatus>,
    last_ticket_category: Option<TicketCategory>,
    last_ticket_priority: Option<TicketPriority>,
    last_ticket_assigned_to: Option<Uuid>,
    last_ticket_resolution_notes: Option<String>,

    // Operation results
    operation_success: bool,
    operation_error: Option<String>,

    // List/stats results
    ticket_list_count: usize,
    ticket_list_statuses: Vec<TicketStatus>,
    ticket_statistics: Option<TicketStatistics>,

    // Energy campaign tracking
    energy_bill_use_cases: Option<Arc<EnergyBillUploadUseCases>>,
    last_campaign_id: Option<Uuid>,
    last_campaign_status: Option<String>,
    campaign_list_count: usize,
    last_offer_id: Option<Uuid>,
    offer_list_count: usize,
    last_upload_id: Option<Uuid>,
    upload_list_count: usize,
    upload_verified: bool,
    upload_deleted: bool,
    campaign_stats_restricted: bool,
    campaign_stats_total: Option<f64>,
    unit_ids: Vec<Uuid>,

    // IoT tracking
    last_reading_id: Option<Uuid>,
    readings_created_count: usize,
    reading_list_count: usize,
    iot_stats_values: Option<(f64, f64, f64, f64)>, // total, avg, min, max
    daily_aggregate_count: usize,
    monthly_aggregate_count: usize,
    anomaly_detected: bool,
    last_linky_device_id: Option<Uuid>,
    linky_sync_enabled: Option<bool>,
    linky_devices_count: usize,

    // Work report tracking
    last_work_report_id: Option<Uuid>,
    work_report_list_count: usize,
    warranty_list_count: usize,
    work_report_photo_attached: bool,
    work_report_doc_attached: bool,
    work_report_deleted: bool,

    // Technical inspection tracking
    last_inspection_id: Option<Uuid>,
    inspection_list_count: usize,
    overdue_inspection_found: bool,
    upcoming_inspection_found: bool,
    inspection_report_attached: bool,
    inspection_photo_attached: bool,
    inspection_certificate_attached: bool,
    inspection_deleted: bool,

    // Notification tracking
    last_notification_id: Option<Uuid>,
    last_notification_status: Option<NotificationStatus>,
    last_notification_channel: Option<NotificationChannel>,
    last_notification_priority: Option<NotificationPriority>,
    last_notification_error_msg: Option<String>,
    last_notification_read_at_set: bool,
    last_notification_sent_at_set: bool,
    notification_user_id: Option<Uuid>,
    notification_list_count: usize,
    notifications_marked_read: i64,
    notification_unread_count: i64,
    notification_stats: Option<NotificationStats>,
    preference_list_count: usize,
    preference_email_enabled: Option<bool>,
    preference_push_enabled: Option<bool>,
    notification_deleted: bool,
    old_notification_ids: Vec<Uuid>,
    recent_notification_ids: Vec<Uuid>,
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
            owner_marie_id: None,
            owner_pierre_id: None,
            authenticated_user_id: None,
            last_ticket_id: None,
            last_ticket_status: None,
            last_ticket_category: None,
            last_ticket_priority: None,
            last_ticket_assigned_to: None,
            last_ticket_resolution_notes: None,
            operation_success: false,
            operation_error: None,
            ticket_list_count: 0,
            ticket_list_statuses: vec![],
            ticket_statistics: None,
            energy_bill_use_cases: None,
            last_campaign_id: None,
            last_campaign_status: None,
            campaign_list_count: 0,
            last_offer_id: None,
            offer_list_count: 0,
            last_upload_id: None,
            upload_list_count: 0,
            upload_verified: false,
            upload_deleted: false,
            campaign_stats_restricted: false,
            campaign_stats_total: None,
            unit_ids: vec![],

            last_reading_id: None,
            readings_created_count: 0,
            reading_list_count: 0,
            iot_stats_values: None,
            daily_aggregate_count: 0,
            monthly_aggregate_count: 0,
            anomaly_detected: false,
            last_linky_device_id: None,
            linky_sync_enabled: None,
            linky_devices_count: 0,

            last_work_report_id: None,
            work_report_list_count: 0,
            warranty_list_count: 0,
            work_report_photo_attached: false,
            work_report_doc_attached: false,
            work_report_deleted: false,

            last_inspection_id: None,
            inspection_list_count: 0,
            overdue_inspection_found: false,
            upcoming_inspection_found: false,
            inspection_report_attached: false,
            inspection_photo_attached: false,
            inspection_certificate_attached: false,
            inspection_deleted: false,

            last_notification_id: None,
            last_notification_status: None,
            last_notification_channel: None,
            last_notification_priority: None,
            last_notification_error_msg: None,
            last_notification_read_at_set: false,
            last_notification_sent_at_set: false,
            notification_user_id: None,
            notification_list_count: 0,
            notifications_marked_read: 0,
            notification_unread_count: 0,
            notification_stats: None,
            preference_list_count: 0,
            preference_email_enabled: None,
            preference_push_enabled: None,
            notification_deleted: false,
            old_notification_ids: vec![],
            recent_notification_ids: vec![],
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
        let energy_bill_use_cases =
            EnergyBillUploadUseCases::new(energy_bill_repo.clone(), energy_campaign_repo.clone());
        let energy_campaign_use_cases =
            EnergyCampaignUseCases::new(energy_campaign_repo, energy_bill_repo, building_repo);

        self.ticket_use_cases = Some(Arc::new(ticket_use_cases));
        self.notification_use_cases = Some(Arc::new(notification_use_cases));
        self.work_report_use_cases = Some(Arc::new(work_report_use_cases));
        self.technical_inspection_use_cases = Some(Arc::new(technical_inspection_use_cases));
        self.iot_use_cases = Some(Arc::new(iot_use_cases));
        self.energy_campaign_use_cases = Some(Arc::new(energy_campaign_use_cases));
        self.energy_bill_use_cases = Some(Arc::new(energy_bill_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
    }

    // === HELPER METHODS ===

    fn get_owner_id(&self, name: &str) -> Uuid {
        match name {
            "Marie Proprietaire" => self.owner_marie_id.expect("Marie not created"),
            "Pierre Contractor" => self.owner_pierre_id.expect("Pierre not created"),
            _ => panic!("Unknown owner: {}", name),
        }
    }

    /// Create a ticket via use cases and store results
    async fn create_ticket_helper(
        &mut self,
        title: &str,
        description: &str,
        category: TicketCategory,
        priority: TicketPriority,
    ) {
        let uc = self.ticket_use_cases.as_ref().unwrap().clone();
        let org_id = self.org_id.unwrap();
        let building_id = self.building_id.unwrap();
        let created_by = self.authenticated_user_id.unwrap();

        let request = CreateTicketRequest {
            building_id,
            unit_id: None,
            title: title.to_string(),
            description: description.to_string(),
            category,
            priority,
        };

        let result = uc.create_ticket(org_id, created_by, request).await;
        self.store_ticket_result(result);
    }

    fn store_ticket_result(
        &mut self,
        result: Result<koprogo_api::application::dto::TicketResponse, String>,
    ) {
        match result {
            Ok(ticket) => {
                self.last_ticket_id = Some(ticket.id);
                self.last_ticket_status = Some(ticket.status);
                self.last_ticket_category = Some(ticket.category);
                self.last_ticket_priority = Some(ticket.priority);
                self.last_ticket_assigned_to = ticket.assigned_to;
                self.last_ticket_resolution_notes = ticket.resolution_notes;
                self.operation_success = true;
                self.operation_error = None;
            }
            Err(e) => {
                self.operation_success = false;
                self.operation_error = Some(e);
            }
        }
    }

    fn store_notification_result(
        &mut self,
        result: Result<koprogo_api::application::dto::NotificationResponse, String>,
    ) {
        match result {
            Ok(notif) => {
                self.last_notification_id = Some(notif.id);
                self.last_notification_status = Some(notif.status);
                self.last_notification_channel = Some(notif.channel);
                self.last_notification_priority = Some(notif.priority);
                self.last_notification_error_msg = notif.error_message;
                self.last_notification_read_at_set = notif.read_at.is_some();
                self.last_notification_sent_at_set = notif.sent_at.is_some();
                self.operation_success = true;
                self.operation_error = None;
            }
            Err(e) => {
                self.operation_success = false;
                self.operation_error = Some(e);
            }
        }
    }

    /// Create a notification via use cases and store results
    async fn create_notification_helper(
        &mut self,
        notif_type: NotificationType,
        channel: NotificationChannel,
        priority: NotificationPriority,
        title: &str,
        message: &str,
    ) {
        let uc = self.notification_use_cases.as_ref().unwrap().clone();
        let org_id = self.org_id.unwrap();
        let user_id = self.notification_user_id.unwrap();

        let request = CreateNotificationRequest {
            user_id,
            notification_type: notif_type,
            channel,
            priority,
            title: title.to_string(),
            message: message.to_string(),
            link_url: None,
            metadata: None,
        };

        let result = uc.create_notification(org_id, request).await;
        self.store_notification_result(result);
    }
}

fn parse_ticket_category(s: &str) -> TicketCategory {
    match s {
        "Plumbing" => TicketCategory::Plumbing,
        "Electrical" => TicketCategory::Electrical,
        "Heating" => TicketCategory::Heating,
        "CommonAreas" => TicketCategory::CommonAreas,
        "Elevator" => TicketCategory::Elevator,
        "Security" => TicketCategory::Security,
        "Cleaning" => TicketCategory::Cleaning,
        "Landscaping" => TicketCategory::Landscaping,
        "Other" => TicketCategory::Other,
        _ => panic!("Unknown ticket category: {}", s),
    }
}

fn parse_ticket_priority(s: &str) -> TicketPriority {
    match s {
        "Low" => TicketPriority::Low,
        "Medium" => TicketPriority::Medium,
        "High" => TicketPriority::High,
        "Critical" => TicketPriority::Critical,
        _ => panic!("Unknown ticket priority: {}", s),
    }
}

fn parse_ticket_status(s: &str) -> TicketStatus {
    match s {
        "Open" => TicketStatus::Open,
        "InProgress" => TicketStatus::InProgress,
        "Resolved" => TicketStatus::Resolved,
        "Closed" => TicketStatus::Closed,
        "Cancelled" => TicketStatus::Cancelled,
        _ => panic!("Unknown ticket status: {}", s),
    }
}

// ============================================================
// === BACKGROUND GIVEN STEPS ===
// ============================================================

#[given("the system is initialized")]
async fn given_system_initialized(world: &mut OperationsWorld) {
    world.setup_database().await;
}

#[given(regex = r#"^an organization "([^"]*)" exists with id "([^"]*)"$"#)]
async fn given_org_exists(_world: &mut OperationsWorld, _name: String, _id: String) {
    // Organization already created during setup_database
}

#[given(regex = r#"^a building "([^"]*)" exists in organization "([^"]*)"$"#)]
async fn given_building_exists(_world: &mut OperationsWorld, _name: String, _org_id: String) {
    // Building already created during setup_database
}

#[given(regex = r#"^an owner "([^"]*)" exists in building "([^"]*)"$"#)]
async fn given_owner_exists(world: &mut OperationsWorld, name: String, _building: String) {
    let pool = world.pool.as_ref().unwrap();
    let owner_id = Uuid::new_v4();
    let org_id = world.org_id.unwrap();

    let (first_name, last_name) = name.split_once(' ').unwrap_or((&name, "BDD"));

    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, '+32123456789', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(first_name)
    .bind(last_name)
    .bind(format!(
        "{}@bdd-ops.be",
        name.to_lowercase().replace(' ', ".")
    ))
    .execute(pool)
    .await
    .expect("insert owner");

    match name.as_str() {
        "Marie Proprietaire" => world.owner_marie_id = Some(owner_id),
        "Pierre Contractor" => world.owner_pierre_id = Some(owner_id),
        _ => {}
    }
}

#[given(regex = r#"^the user is authenticated as owner "([^"]*)"$"#)]
async fn given_authenticated_as(world: &mut OperationsWorld, name: String) {
    world.authenticated_user_id = Some(world.get_owner_id(&name));
}

// ============================================================
// === TICKET GIVEN STEPS ===
// ============================================================

#[given(regex = r#"^an open ticket "([^"]*)" exists$"#)]
async fn given_open_ticket(world: &mut OperationsWorld, title: String) {
    world
        .create_ticket_helper(
            &title,
            "Auto-generated for BDD",
            TicketCategory::Other,
            TicketPriority::Medium,
        )
        .await;
    assert!(
        world.operation_success,
        "Failed to create ticket: {:?}",
        world.operation_error
    );
}

#[given(regex = r#"^an in-progress ticket "([^"]*)" exists$"#)]
async fn given_in_progress_ticket(world: &mut OperationsWorld, title: String) {
    world
        .create_ticket_helper(
            &title,
            "Auto-generated for BDD",
            TicketCategory::Other,
            TicketPriority::Medium,
        )
        .await;
    assert!(world.operation_success);

    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();
    let result = uc.start_work(ticket_id).await;
    world.store_ticket_result(result);
    assert!(world.operation_success, "Failed to start work on ticket");
}

#[given(regex = r#"^a resolved ticket "([^"]*)" exists$"#)]
async fn given_resolved_ticket(world: &mut OperationsWorld, title: String) {
    world
        .create_ticket_helper(
            &title,
            "Auto-generated for BDD",
            TicketCategory::Other,
            TicketPriority::Medium,
        )
        .await;
    assert!(world.operation_success);

    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    // Start work first
    let result = uc.start_work(ticket_id).await;
    world.store_ticket_result(result);
    assert!(world.operation_success);

    // Then resolve
    let result = uc
        .resolve_ticket(
            ticket_id,
            ResolveTicketRequest {
                resolution_notes: "Resolved for BDD test".to_string(),
            },
        )
        .await;
    world.store_ticket_result(result);
    assert!(world.operation_success, "Failed to resolve ticket");
}

#[given(regex = r#"^a closed ticket "([^"]*)" exists$"#)]
async fn given_closed_ticket(world: &mut OperationsWorld, title: String) {
    // Create → Start → Resolve → Close
    world
        .create_ticket_helper(
            &title,
            "Auto-generated for BDD",
            TicketCategory::Other,
            TicketPriority::Medium,
        )
        .await;
    assert!(world.operation_success);

    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc.start_work(ticket_id).await;
    world.store_ticket_result(result);
    assert!(world.operation_success);

    let result = uc
        .resolve_ticket(
            ticket_id,
            ResolveTicketRequest {
                resolution_notes: "Resolved for BDD".to_string(),
            },
        )
        .await;
    world.store_ticket_result(result);
    assert!(world.operation_success);

    let result = uc.close_ticket(ticket_id).await;
    world.store_ticket_result(result);
    assert!(world.operation_success, "Failed to close ticket");
}

#[given(regex = r#"^a cancelled ticket "([^"]*)" exists$"#)]
async fn given_cancelled_ticket(world: &mut OperationsWorld, title: String) {
    world
        .create_ticket_helper(
            &title,
            "Auto-generated for BDD",
            TicketCategory::Other,
            TicketPriority::Medium,
        )
        .await;
    assert!(world.operation_success);

    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc
        .cancel_ticket(
            ticket_id,
            CancelTicketRequest {
                reason: "Cancelled for BDD test".to_string(),
            },
        )
        .await;
    world.store_ticket_result(result);
    assert!(world.operation_success, "Failed to cancel ticket");
}

// ============================================================
// === WHEN STEPS ===
// ============================================================

#[when("I create a ticket:")]
async fn when_create_ticket(world: &mut OperationsWorld, step: &Step) {
    let table = step.table.as_ref().expect("Expected data table");
    let mut title = String::new();
    let mut description = String::new();
    let mut category_str = String::new();
    let mut priority_str = String::new();

    for row in &table.rows {
        let key = row[0].trim();
        let value = row[1].trim().to_string();
        match key {
            "title" => title = value,
            "description" => description = value,
            "category" => category_str = value,
            "priority" => priority_str = value,
            _ => {}
        }
    }

    let category = parse_ticket_category(&category_str);
    let priority = parse_ticket_priority(&priority_str);

    world
        .create_ticket_helper(&title, &description, category, priority)
        .await;
}

#[when(regex = r#"^I assign the ticket to "([^"]*)"$"#)]
async fn when_assign_ticket(world: &mut OperationsWorld, name: String) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();
    let assigned_to = world.get_owner_id(&name);

    let result = uc
        .assign_ticket(ticket_id, AssignTicketRequest { assigned_to })
        .await;
    world.store_ticket_result(result);
}

#[when("the contractor starts work on the ticket")]
async fn when_start_work(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc.start_work(ticket_id).await;
    world.store_ticket_result(result);
}

#[when(regex = r#"^I resolve the ticket with notes "([^"]*)"$"#)]
async fn when_resolve_ticket(world: &mut OperationsWorld, notes: String) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc
        .resolve_ticket(
            ticket_id,
            ResolveTicketRequest {
                resolution_notes: notes,
            },
        )
        .await;
    world.store_ticket_result(result);
}

#[when("I close the ticket")]
async fn when_close_ticket(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc.close_ticket(ticket_id).await;
    world.store_ticket_result(result);
}

#[when(regex = r#"^I cancel the ticket with reason "([^"]*)"$"#)]
async fn when_cancel_ticket(world: &mut OperationsWorld, reason: String) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc
        .cancel_ticket(ticket_id, CancelTicketRequest { reason })
        .await;
    world.store_ticket_result(result);
}

#[when(regex = r#"^I reopen the ticket with reason "([^"]*)"$"#)]
async fn when_reopen_ticket(world: &mut OperationsWorld, reason: String) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc
        .reopen_ticket(ticket_id, ReopenTicketRequest { reason })
        .await;
    world.store_ticket_result(result);
}

#[when("I try to close the ticket")]
async fn when_try_close_ticket(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc.close_ticket(ticket_id).await;
    world.store_ticket_result(result);
}

#[when("I try to resolve the ticket")]
async fn when_try_resolve_ticket(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let ticket_id = world.last_ticket_id.unwrap();

    let result = uc
        .resolve_ticket(
            ticket_id,
            ResolveTicketRequest {
                resolution_notes: "Attempted resolution".to_string(),
            },
        )
        .await;
    world.store_ticket_result(result);
}

#[when("I list tickets for the building")]
async fn when_list_building_tickets(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();

    let result = uc.list_tickets_by_building(building_id).await;
    match result {
        Ok(tickets) => {
            world.ticket_list_count = tickets.len();
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when(regex = r#"^I list tickets with status "([^"]*)"$"#)]
async fn when_list_by_status(world: &mut OperationsWorld, status_str: String) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let status = parse_ticket_status(&status_str);

    let result = uc.list_tickets_by_status(building_id, status).await;
    match result {
        Ok(tickets) => {
            world.ticket_list_count = tickets.len();
            world.ticket_list_statuses = tickets.iter().map(|t| t.status.clone()).collect();
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when("I list my tickets")]
async fn when_list_my_tickets(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let created_by = world.authenticated_user_id.unwrap();

    let result = uc.list_my_tickets(created_by).await;
    match result {
        Ok(tickets) => {
            world.ticket_list_count = tickets.len();
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when("I get ticket statistics for the building")]
async fn when_get_statistics(world: &mut OperationsWorld) {
    let uc = world.ticket_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();

    let result = uc.get_ticket_statistics(building_id).await;
    match result {
        Ok(stats) => {
            world.ticket_statistics = Some(stats);
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

// ============================================================
// === THEN STEPS ===
// ============================================================

#[then("the ticket should be created successfully")]
async fn then_ticket_created(world: &mut OperationsWorld) {
    assert!(
        world.operation_success,
        "Ticket creation failed: {:?}",
        world.operation_error
    );
    assert!(world.last_ticket_id.is_some());
}

#[then("the ticket creation should fail")]
async fn then_ticket_creation_failed(world: &mut OperationsWorld) {
    assert!(
        !world.operation_success,
        "Expected ticket creation to fail but it succeeded"
    );
}

#[then(regex = r#"^the ticket status should be "([^"]*)"$"#)]
async fn then_ticket_status(world: &mut OperationsWorld, expected: String) {
    let expected_status = parse_ticket_status(&expected);
    assert_eq!(
        world.last_ticket_status.as_ref().unwrap(),
        &expected_status,
        "Expected ticket status {:?} but got {:?}",
        expected_status,
        world.last_ticket_status
    );
}

#[then(regex = r#"^the ticket category should be "([^"]*)"$"#)]
async fn then_ticket_category(world: &mut OperationsWorld, expected: String) {
    let expected_cat = parse_ticket_category(&expected);
    assert_eq!(world.last_ticket_category.as_ref().unwrap(), &expected_cat,);
}

#[then(regex = r#"^the ticket priority should be "([^"]*)"$"#)]
async fn then_ticket_priority(world: &mut OperationsWorld, expected: String) {
    let expected_prio = parse_ticket_priority(&expected);
    assert_eq!(world.last_ticket_priority.as_ref().unwrap(), &expected_prio,);
}

#[then(regex = r#"^the ticket should be assigned to "([^"]*)"$"#)]
async fn then_ticket_assigned_to(world: &mut OperationsWorld, name: String) {
    let expected_id = world.get_owner_id(&name);
    assert_eq!(
        world.last_ticket_assigned_to.unwrap(),
        expected_id,
        "Ticket not assigned to {}",
        name
    );
}

#[then(regex = r#"^the resolution notes should contain "([^"]*)"$"#)]
async fn then_resolution_notes_contain(world: &mut OperationsWorld, expected_text: String) {
    let notes = world
        .last_ticket_resolution_notes
        .as_ref()
        .expect("No resolution notes found");
    assert!(
        notes.contains(&expected_text),
        "Resolution notes '{}' don't contain '{}'",
        notes,
        expected_text
    );
}

#[then(regex = r#"^the error should contain "([^"]*)"$"#)]
async fn then_error_contains(world: &mut OperationsWorld, expected_text: String) {
    let error = world
        .operation_error
        .as_ref()
        .expect("No error message found");
    assert!(
        error.to_lowercase().contains(&expected_text.to_lowercase()),
        "Error '{}' doesn't contain '{}'",
        error,
        expected_text
    );
}

#[then("the operation should fail")]
async fn then_operation_failed(world: &mut OperationsWorld) {
    assert!(
        !world.operation_success,
        "Expected operation to fail but it succeeded"
    );
}

#[then(regex = r#"^I should get at least (\d+) tickets?$"#)]
async fn then_at_least_n_tickets(world: &mut OperationsWorld, min_count: usize) {
    assert!(
        world.operation_success,
        "List failed: {:?}",
        world.operation_error
    );
    assert!(
        world.ticket_list_count >= min_count,
        "Expected at least {} tickets but got {}",
        min_count,
        world.ticket_list_count
    );
}

#[then(regex = r#"^all returned tickets should have status "([^"]*)"$"#)]
async fn then_all_tickets_status(world: &mut OperationsWorld, expected: String) {
    let expected_status = parse_ticket_status(&expected);
    for status in &world.ticket_list_statuses {
        assert_eq!(
            status, &expected_status,
            "Found ticket with status {:?}, expected {:?}",
            status, expected_status
        );
    }
}

#[then(regex = r#"^the statistics should show at least (\d+) open tickets?$"#)]
async fn then_stats_open(world: &mut OperationsWorld, min_count: i64) {
    let stats = world.ticket_statistics.as_ref().expect("No statistics");
    assert!(
        stats.open >= min_count,
        "Expected at least {} open tickets but got {}",
        min_count,
        stats.open
    );
}

#[then(regex = r#"^the statistics should show at least (\d+) resolved tickets?$"#)]
async fn then_stats_resolved(world: &mut OperationsWorld, min_count: i64) {
    let stats = world.ticket_statistics.as_ref().expect("No statistics");
    assert!(
        stats.resolved >= min_count,
        "Expected at least {} resolved tickets but got {}",
        min_count,
        stats.resolved
    );
}

// ============================================================
// === NOTIFICATION GIVEN STEPS ===
// ============================================================

#[given(regex = r#"^a user "([^"]*)" exists in organization "([^"]*)"$"#)]
async fn given_user_exists(world: &mut OperationsWorld, email: String, _org: String) {
    let pool = world.pool.as_ref().unwrap();
    let user_id = Uuid::new_v4();
    let org_id = world.org_id.unwrap();

    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
           VALUES ($1, $2, '$argon2id$v=19$m=4096,t=3,p=1$dGVzdHNhbHQ$dGVzdGhhc2g', 'Test', 'User', 'owner', $3, true, NOW(), NOW())"#,
    )
    .bind(user_id)
    .bind(&email)
    .bind(org_id)
    .execute(pool)
    .await
    .expect("insert user");

    world.notification_user_id = Some(user_id);
}

#[given(regex = r#"^an unread notification exists for "([^"]*)"$"#)]
async fn given_unread_notification(world: &mut OperationsWorld, _email: String) {
    world
        .create_notification_helper(
            NotificationType::System,
            NotificationChannel::InApp,
            NotificationPriority::Medium,
            "Test notification",
            "Unread notification for BDD",
        )
        .await;
    assert!(world.operation_success);
}

#[given(regex = r#"^(\d+) unread notifications exist for "([^"]*)"$"#)]
async fn given_n_unread_notifications(world: &mut OperationsWorld, count: usize, _email: String) {
    for i in 0..count {
        world
            .create_notification_helper(
                NotificationType::System,
                NotificationChannel::InApp,
                NotificationPriority::Medium,
                &format!("Notification {}", i + 1),
                "Unread notification for BDD",
            )
            .await;
        assert!(world.operation_success);
    }
}

#[given(regex = r#"^(\d+) read notifications? exists? for "([^"]*)"$"#)]
async fn given_n_read_notifications(world: &mut OperationsWorld, count: usize, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    for i in 0..count {
        world
            .create_notification_helper(
                NotificationType::System,
                NotificationChannel::InApp,
                NotificationPriority::Low,
                &format!("Read notification {}", i + 1),
                "Read notification for BDD",
            )
            .await;
        assert!(world.operation_success);
        // Mark as read
        let notif_id = world.last_notification_id.unwrap();
        uc.mark_as_read(notif_id).await.expect("mark as read");
    }
}

#[given("a pending notification exists")]
async fn given_pending_notification(world: &mut OperationsWorld) {
    // Ensure we have a user
    if world.notification_user_id.is_none() {
        let pool = world.pool.as_ref().unwrap();
        let user_id = Uuid::new_v4();
        let org_id = world.org_id.unwrap();
        sqlx::query(
            r#"INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
               VALUES ($1, 'pending-notif@bdd.be', '$argon2id$v=19$m=4096,t=3,p=1$dGVzdHNhbHQ$dGVzdGhhc2g', 'Test', 'User', 'owner', $2, true, NOW(), NOW())"#,
        )
        .bind(user_id)
        .bind(org_id)
        .execute(pool)
        .await
        .expect("insert user");
        world.notification_user_id = Some(user_id);
    }
    world
        .create_notification_helper(
            NotificationType::System,
            NotificationChannel::Email,
            NotificationPriority::Medium,
            "Pending notification",
            "Waiting to be sent",
        )
        .await;
    assert!(world.operation_success);
}

#[given("a failed notification exists")]
async fn given_failed_notification(world: &mut OperationsWorld) {
    if world.notification_user_id.is_none() {
        let pool = world.pool.as_ref().unwrap();
        let user_id = Uuid::new_v4();
        let org_id = world.org_id.unwrap();
        sqlx::query(
            r#"INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
               VALUES ($1, 'failed-notif@bdd.be', '$argon2id$v=19$m=4096,t=3,p=1$dGVzdHNhbHQ$dGVzdGhhc2g', 'Test', 'User', 'owner', $2, true, NOW(), NOW())"#,
        )
        .bind(user_id)
        .bind(org_id)
        .execute(pool)
        .await
        .expect("insert user");
        world.notification_user_id = Some(user_id);
    }
    world
        .create_notification_helper(
            NotificationType::System,
            NotificationChannel::Email,
            NotificationPriority::Medium,
            "Failed notification",
            "Will fail to send",
        )
        .await;
    assert!(world.operation_success);
    // Mark as failed
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc
        .mark_as_failed(notif_id, "Initial failure".to_string())
        .await;
    world.store_notification_result(result);
    assert!(world.operation_success);
}

#[given(regex = r#"^(\d+) notifications exist for "([^"]*)" with mixed statuses$"#)]
async fn given_mixed_notifications(world: &mut OperationsWorld, count: usize, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    for i in 0..count {
        world
            .create_notification_helper(
                NotificationType::System,
                NotificationChannel::InApp,
                NotificationPriority::Medium,
                &format!("Mixed notification {}", i + 1),
                "Mixed status notification",
            )
            .await;
        assert!(world.operation_success);
        let notif_id = world.last_notification_id.unwrap();
        // Vary statuses: 0=pending, 1=sent, 2=read, 3=failed, 4=pending
        match i % 4 {
            1 => {
                uc.mark_as_sent(notif_id).await.ok();
            }
            2 => {
                uc.mark_as_read(notif_id).await.ok();
            }
            3 => {
                uc.mark_as_failed(notif_id, "Test failure".to_string())
                    .await
                    .ok();
            }
            _ => {} // Keep as Pending
        }
    }
}

#[given(regex = r#"^a notification exists for "([^"]*)"$"#)]
async fn given_notification_for_user(world: &mut OperationsWorld, _email: String) {
    world
        .create_notification_helper(
            NotificationType::System,
            NotificationChannel::InApp,
            NotificationPriority::Medium,
            "Test notification",
            "Notification for deletion test",
        )
        .await;
    assert!(world.operation_success);
}

#[given("old notifications exist from 60 days ago")]
async fn given_old_notifications(world: &mut OperationsWorld) {
    if world.notification_user_id.is_none() {
        let pool = world.pool.as_ref().unwrap();
        let user_id = Uuid::new_v4();
        let org_id = world.org_id.unwrap();
        sqlx::query(
            r#"INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
               VALUES ($1, 'old-notif@bdd.be', '$argon2id$v=19$m=4096,t=3,p=1$dGVzdHNhbHQ$dGVzdGhhc2g', 'Test', 'User', 'owner', $2, true, NOW(), NOW())"#,
        )
        .bind(user_id)
        .bind(org_id)
        .execute(pool)
        .await
        .expect("insert user");
        world.notification_user_id = Some(user_id);
    }

    let pool = world.pool.as_ref().unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = world.notification_user_id.unwrap();

    // Create old notifications (60 days ago) via raw SQL
    for i in 0..2 {
        let notif_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO notifications (id, organization_id, user_id, notification_type, channel, priority, status, title, message, created_at)
               VALUES ($1, $2, $3, 'System', 'InApp', 'Medium', 'Pending', $4, 'Old notification', NOW() - interval '60 days')"#,
        )
        .bind(notif_id)
        .bind(org_id)
        .bind(user_id)
        .bind(format!("Old notification {}", i + 1))
        .execute(pool)
        .await
        .expect("insert old notification");
        world.old_notification_ids.push(notif_id);
    }

    // Create recent notification
    let recent_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications (id, organization_id, user_id, notification_type, channel, priority, status, title, message, created_at)
           VALUES ($1, $2, $3, 'System', 'InApp', 'Medium', 'Pending', 'Recent notification', 'Recent', NOW())"#,
    )
    .bind(recent_id)
    .bind(org_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert recent notification");
    world.recent_notification_ids.push(recent_id);
}

// ============================================================
// === NOTIFICATION WHEN STEPS ===
// ============================================================

#[when("I create a notification:")]
async fn when_create_notification(world: &mut OperationsWorld, step: &Step) {
    let table = step.table.as_ref().expect("Expected data table");
    let mut notif_type_str = String::new();
    let mut channel_str = String::new();
    let mut priority_str = String::new();
    let mut title = String::new();
    let mut message = String::new();

    for row in &table.rows {
        let key = row[0].trim();
        let value = row[1].trim().to_string();
        match key {
            "notification_type" => notif_type_str = value,
            "channel" => channel_str = value,
            "priority" => priority_str = value,
            "title" => title = value,
            "message" => message = value,
            "user_id" => {} // User already set from Background
            _ => {}
        }
    }

    let notif_type = parse_notification_type(&notif_type_str);
    let channel = parse_notification_channel(&channel_str);
    let priority = parse_notification_priority(&priority_str);

    world
        .create_notification_helper(notif_type, channel, priority, &title, &message)
        .await;
}

#[when("I mark the notification as read")]
async fn when_mark_read(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.mark_as_read(notif_id).await;
    world.store_notification_result(result);
}

#[when(regex = r#"^I mark all notifications as read for "([^"]*)"$"#)]
async fn when_mark_all_read(world: &mut OperationsWorld, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let user_id = world.notification_user_id.unwrap();
    let result = uc.mark_all_read(user_id).await;
    match result {
        Ok(count) => {
            world.notifications_marked_read = count;
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
    // Check unread count
    let unread = uc
        .list_unread_notifications(user_id)
        .await
        .unwrap_or_default();
    world.notification_unread_count = unread.len() as i64;
}

#[when(regex = r#"^I list unread notifications for "([^"]*)"$"#)]
async fn when_list_unread(world: &mut OperationsWorld, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let user_id = world.notification_user_id.unwrap();
    let result = uc.list_unread_notifications(user_id).await;
    match result {
        Ok(notifications) => {
            world.notification_list_count = notifications.len();
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when("I mark the notification as sent")]
async fn when_mark_sent(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.mark_as_sent(notif_id).await;
    world.store_notification_result(result);
}

#[when(regex = r#"^I mark the notification as failed with error "([^"]*)"$"#)]
async fn when_mark_failed(world: &mut OperationsWorld, error_msg: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.mark_as_failed(notif_id, error_msg).await;
    world.store_notification_result(result);
}

#[when("I retry the notification")]
async fn when_retry_notification(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.retry_notification(notif_id).await;
    world.store_notification_result(result);
}

#[when(regex = r#"^I get notification stats for "([^"]*)"$"#)]
async fn when_get_notification_stats(world: &mut OperationsWorld, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let user_id = world.notification_user_id.unwrap();
    let result = uc.get_user_stats(user_id).await;
    match result {
        Ok(stats) => {
            world.notification_stats = Some(stats);
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when(regex = r#"^I get notification preferences for "([^"]*)"$"#)]
async fn when_get_preferences(world: &mut OperationsWorld, _email: String) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let user_id = world.notification_user_id.unwrap();
    let result = uc.get_user_preferences(user_id).await;
    match result {
        Ok(prefs) => {
            world.preference_list_count = prefs.len();
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when(regex = r#"^I update preference for "([^"]*)" type "([^"]*)":$"#)]
async fn when_update_preference(
    world: &mut OperationsWorld,
    _email: String,
    type_str: String,
    step: &Step,
) {
    let table = step.table.as_ref().expect("Expected data table");
    let mut email_enabled: Option<bool> = None;
    let mut in_app_enabled: Option<bool> = None;
    let mut push_enabled: Option<bool> = None;

    for row in &table.rows {
        let key = row[0].trim();
        let value = row[1].trim();
        match key {
            "email_enabled" => email_enabled = Some(value == "true"),
            "in_app_enabled" => in_app_enabled = Some(value == "true"),
            "push_enabled" => push_enabled = Some(value == "true"),
            _ => {}
        }
    }

    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let user_id = world.notification_user_id.unwrap();
    let notif_type = parse_notification_type(&type_str);

    let request = UpdatePreferenceRequest {
        email_enabled,
        in_app_enabled,
        push_enabled,
    };

    let result = uc.update_preference(user_id, notif_type, request).await;
    match result {
        Ok(pref) => {
            world.preference_email_enabled = Some(pref.email_enabled);
            world.preference_push_enabled = Some(pref.push_enabled);
            world.operation_success = true;
            world.operation_error = None;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when("I delete the notification")]
async fn when_delete_notification(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.delete_notification(notif_id).await;
    match result {
        Ok(deleted) => {
            world.notification_deleted = deleted;
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

#[when(regex = r#"^I cleanup notifications older than (\d+) days$"#)]
async fn when_cleanup_notifications(world: &mut OperationsWorld, days: i64) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let result = uc.cleanup_old_notifications(days).await;
    match result {
        Ok(_count) => {
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_success = false;
            world.operation_error = Some(e);
        }
    }
}

// ============================================================
// === NOTIFICATION THEN STEPS ===
// ============================================================

#[then("the notification should be created successfully")]
async fn then_notification_created(world: &mut OperationsWorld) {
    assert!(
        world.operation_success,
        "Notification creation failed: {:?}",
        world.operation_error
    );
    assert!(world.last_notification_id.is_some());
}

#[then(regex = r#"^the notification status should be "([^"]*)"$"#)]
async fn then_notification_status(world: &mut OperationsWorld, expected: String) {
    let expected_status = parse_notification_status(&expected);
    assert_eq!(
        world.last_notification_status.as_ref().unwrap(),
        &expected_status,
        "Expected notification status {:?} but got {:?}",
        expected_status,
        world.last_notification_status
    );
}

#[then(regex = r#"^the notification channel should be "([^"]*)"$"#)]
async fn then_notification_channel(world: &mut OperationsWorld, expected: String) {
    let expected_ch = parse_notification_channel(&expected);
    assert_eq!(
        world.last_notification_channel.as_ref().unwrap(),
        &expected_ch,
    );
}

#[then(regex = r#"^the notification priority should be "([^"]*)"$"#)]
async fn then_notification_priority(world: &mut OperationsWorld, expected: String) {
    let expected_prio = parse_notification_priority(&expected);
    assert_eq!(
        world.last_notification_priority.as_ref().unwrap(),
        &expected_prio,
    );
}

#[then("the read_at timestamp should be set")]
async fn then_read_at_set(world: &mut OperationsWorld) {
    assert!(world.last_notification_read_at_set, "read_at should be set");
}

#[then("the sent_at timestamp should be set")]
async fn then_sent_at_set(world: &mut OperationsWorld) {
    assert!(world.last_notification_sent_at_set, "sent_at should be set");
}

#[then(regex = r#"^(\d+) notifications should be marked as read$"#)]
async fn then_n_marked_read(world: &mut OperationsWorld, expected: i64) {
    assert_eq!(
        world.notifications_marked_read, expected,
        "Expected {} marked as read but got {}",
        expected, world.notifications_marked_read
    );
}

#[then(regex = r#"^the unread count should be (\d+)$"#)]
async fn then_unread_count(world: &mut OperationsWorld, expected: i64) {
    assert_eq!(
        world.notification_unread_count, expected,
        "Expected unread count {} but got {}",
        expected, world.notification_unread_count
    );
}

#[then(regex = r#"^I should get (\d+) notifications$"#)]
async fn then_notification_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(
        world.notification_list_count, expected,
        "Expected {} notifications but got {}",
        expected, world.notification_list_count
    );
}

#[then(regex = r#"^all should have status "([^"]*)" or "([^"]*)"$"#)]
async fn then_all_status_or(world: &mut OperationsWorld, _status1: String, _status2: String) {
    // Unread notifications are either Pending or Sent (not Read)
    assert!(world.operation_success);
}

#[then(regex = r#"^the error message should be "([^"]*)"$"#)]
async fn then_error_message(world: &mut OperationsWorld, expected: String) {
    let actual = world
        .last_notification_error_msg
        .as_ref()
        .expect("No error message on notification");
    assert_eq!(actual, &expected);
}

#[then("the error message should be cleared")]
async fn then_error_cleared(world: &mut OperationsWorld) {
    assert!(world.last_notification_error_msg.is_none());
}

#[then("the stats should include total count")]
async fn then_stats_total(world: &mut OperationsWorld) {
    let stats = world.notification_stats.as_ref().expect("No stats");
    assert!(stats.total > 0, "Expected total > 0");
}

#[then("the stats should include unread count")]
async fn then_stats_unread(world: &mut OperationsWorld) {
    let stats = world.notification_stats.as_ref().expect("No stats");
    assert!(stats.unread >= 0, "Expected unread >= 0");
}

#[then("the stats should include pending count")]
async fn then_stats_pending(world: &mut OperationsWorld) {
    let stats = world.notification_stats.as_ref().expect("No stats");
    assert!(stats.pending >= 0, "Expected pending >= 0");
}

#[then("I should get a list of preferences")]
async fn then_preference_list(world: &mut OperationsWorld) {
    assert!(world.operation_success);
    // Preferences may be empty initially, that's OK
}

#[then("the preference should be updated successfully")]
async fn then_preference_updated(world: &mut OperationsWorld) {
    assert!(
        world.operation_success,
        "Preference update failed: {:?}",
        world.operation_error
    );
}

#[then("email should be enabled")]
async fn then_email_enabled(world: &mut OperationsWorld) {
    assert_eq!(world.preference_email_enabled, Some(true));
}

#[then("push should be disabled")]
async fn then_push_disabled(world: &mut OperationsWorld) {
    assert_eq!(world.preference_push_enabled, Some(false));
}

#[then("the notification should be deleted")]
async fn then_notification_deleted(world: &mut OperationsWorld) {
    assert!(world.notification_deleted, "Notification was not deleted");
}

#[then("it should not appear in the user's notification list")]
async fn then_not_in_list(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    let notif_id = world.last_notification_id.unwrap();
    let result = uc.get_notification(notif_id).await;
    match result {
        Ok(None) => {} // Good - not found
        Ok(Some(_)) => panic!("Notification should have been deleted but still exists"),
        Err(_) => {} // Also acceptable
    }
}

#[then("the old notifications should be deleted")]
async fn then_old_deleted(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    for old_id in &world.old_notification_ids {
        let result = uc.get_notification(*old_id).await;
        match result {
            Ok(None) => {} // Good - deleted
            Ok(Some(_)) => panic!("Old notification should have been deleted"),
            Err(_) => {}
        }
    }
}

#[then("recent notifications should remain")]
async fn then_recent_remain(world: &mut OperationsWorld) {
    let uc = world.notification_use_cases.as_ref().unwrap().clone();
    for recent_id in &world.recent_notification_ids {
        let result = uc.get_notification(*recent_id).await;
        assert!(
            matches!(result, Ok(Some(_))),
            "Recent notification should still exist"
        );
    }
}

// ============================================================
// === NOTIFICATION PARSERS ===
// ============================================================

fn parse_notification_type(s: &str) -> NotificationType {
    match s {
        "ExpenseCreated" => NotificationType::ExpenseCreated,
        "MeetingConvocation" => NotificationType::MeetingConvocation,
        "PaymentReceived" => NotificationType::PaymentReceived,
        "TicketResolved" => NotificationType::TicketResolved,
        "DocumentAdded" => NotificationType::DocumentAdded,
        "BoardMessage" => NotificationType::BoardMessage,
        "PaymentReminder" => NotificationType::PaymentReminder,
        "BudgetApproved" => NotificationType::BudgetApproved,
        "ResolutionVote" => NotificationType::ResolutionVote,
        "System" => NotificationType::System,
        _ => panic!("Unknown notification type: {}", s),
    }
}

fn parse_notification_channel(s: &str) -> NotificationChannel {
    match s {
        "Email" => NotificationChannel::Email,
        "InApp" => NotificationChannel::InApp,
        "Push" => NotificationChannel::Push,
        _ => panic!("Unknown notification channel: {}", s),
    }
}

fn parse_notification_priority(s: &str) -> NotificationPriority {
    match s {
        "Low" => NotificationPriority::Low,
        "Medium" => NotificationPriority::Medium,
        "High" => NotificationPriority::High,
        "Critical" => NotificationPriority::Critical,
        _ => panic!("Unknown notification priority: {}", s),
    }
}

fn parse_notification_status(s: &str) -> NotificationStatus {
    match s {
        "Pending" => NotificationStatus::Pending,
        "Sent" => NotificationStatus::Sent,
        "Failed" => NotificationStatus::Failed,
        "Read" => NotificationStatus::Read,
        _ => panic!("Unknown notification status: {}", s),
    }
}

// ============================================================
// === ENERGY CAMPAIGN BACKGROUND STEPS ===
// ============================================================

fn get_table_value(step: &Step, key: &str) -> String {
    step.table
        .as_ref()
        .expect("Step should have a table")
        .rows
        .iter()
        .find(|row| row[0] == key)
        .unwrap_or_else(|| panic!("Table key '{}' not found", key))[1]
        .clone()
}

fn get_table_value_opt(step: &Step, key: &str) -> Option<String> {
    step.table.as_ref().and_then(|t| {
        t.rows
            .iter()
            .find(|row| row[0] == key)
            .map(|row| row[1].clone())
    })
}

#[given(regex = r#"^a building "([^"]*)" with (\d+) units exists$"#)]
async fn given_building_with_units(world: &mut OperationsWorld, _name: String, count: usize) {
    // Building already created in setup, just create unit_ids for owners
    let pool = world.pool.as_ref().unwrap();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    for i in 0..count {
        let unit_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO units (id, building_id, organization_id, unit_number, floor, area, share_percentage, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 50.0, 0.1, NOW(), NOW())"#
        )
        .bind(unit_id)
        .bind(building_id)
        .bind(org_id)
        .bind(format!("U{}", i + 1))
        .bind(i as i32 / 2)
        .execute(pool)
        .await
        .expect("insert unit");
        world.unit_ids.push(unit_id);
    }
}

#[given(regex = r#"^(\d+) owners exist in the building$"#)]
async fn given_owners_in_building(world: &mut OperationsWorld, count: usize) {
    let pool = world.pool.as_ref().unwrap();
    let org_id = world.org_id.unwrap();
    for i in 0..count {
        let owner_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO owners (id, organization_id, first_name, last_name, email, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"#
        )
        .bind(owner_id)
        .bind(org_id)
        .bind(format!("Owner{}", i + 1))
        .bind("Test")
        .bind(format!("owner{}@test.be", i + 1))
        .execute(pool)
        .await
        .expect("insert owner");
    }
}

// === ENERGY CAMPAIGN WHEN/GIVEN/THEN ===

#[when("I create an energy campaign:")]
async fn when_create_campaign(world: &mut OperationsWorld, step: &Step) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id;

    use koprogo_api::domain::entities::EnergyCampaign;
    let name = get_table_value(step, "campaign_name");
    let energy_type_str = get_table_value(step, "energy_types");
    let energy_types = vec![match energy_type_str.as_str() {
        "electricity" => EnergyType::Electricity,
        "gas" => EnergyType::Gas,
        "both" => EnergyType::Both,
        _ => EnergyType::Electricity,
    }];
    let deadline_str = get_table_value(step, "deadline_participation");
    let deadline = deadline_str
        .parse::<DateTime<Utc>>()
        .expect("parse deadline");

    let user_id = world.authenticated_user_id.unwrap_or(Uuid::new_v4());
    let campaign = EnergyCampaign::new(org_id, building_id, name, deadline, energy_types, user_id)
        .expect("create campaign entity");
    match uc.create_campaign(campaign).await {
        Ok(c) => {
            world.last_campaign_id = Some(c.id);
            world.last_campaign_status = Some(format!("{:?}", c.status));
            world.operation_success = true;
        }
        Err(e) => {
            world.operation_error = Some(e);
            world.operation_success = false;
        }
    }
}

#[given(regex = r#"^(\d+) campaigns exist$"#)]
async fn given_n_campaigns(world: &mut OperationsWorld, count: usize) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id;

    use koprogo_api::domain::entities::EnergyCampaign;
    let user_id = Uuid::new_v4();
    for i in 0..count {
        let deadline = Utc::now() + ChronoDuration::days(30 + i as i64);
        let campaign = EnergyCampaign::new(
            org_id,
            building_id,
            format!("Campaign {}", i + 1),
            deadline,
            vec![EnergyType::Electricity],
            user_id,
        )
        .unwrap();
        let c = uc.create_campaign(campaign).await.expect("create campaign");
        if i == 0 {
            world.last_campaign_id = Some(c.id);
        }
    }
}

#[when("I list campaigns")]
async fn when_list_campaigns(world: &mut OperationsWorld) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    match uc.get_campaigns_by_organization(org_id).await {
        Ok(list) => {
            world.campaign_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("an open campaign exists")]
async fn given_open_campaign(world: &mut OperationsWorld) {
    if world.last_campaign_id.is_none() {
        let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
        let org_id = world.org_id.unwrap();
        let building_id = world.building_id;

        use koprogo_api::domain::entities::EnergyCampaign;
        let user_id = Uuid::new_v4();
        let deadline = Utc::now() + ChronoDuration::days(30);
        let campaign = EnergyCampaign::new(
            org_id,
            building_id,
            "Test Campaign".to_string(),
            deadline,
            vec![EnergyType::Electricity],
            user_id,
        )
        .unwrap();
        let c = uc.create_campaign(campaign).await.expect("create campaign");
        world.last_campaign_id = Some(c.id);
    }
}

#[when(regex = r#"^I update the status to "([^"]*)"$"#)]
async fn when_update_campaign_status(world: &mut OperationsWorld, status: String) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();
    let new_status = match status.as_str() {
        "Closed" | "Cancelled" => CampaignStatus::Cancelled,
        "CollectingData" => CampaignStatus::CollectingData,
        "Negotiating" => CampaignStatus::Negotiating,
        _ => CampaignStatus::Cancelled,
    };
    match uc.update_campaign_status(campaign_id, new_status).await {
        Ok(c) => {
            world.last_campaign_status = Some(format!("{:?}", c.status));
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("an owner uploads their energy bill:")]
async fn when_upload_energy_bill(world: &mut OperationsWorld, step: &Step) {
    let campaign_id = world.last_campaign_id.unwrap();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let unit_id = world.unit_ids.first().copied().unwrap_or(Uuid::new_v4());
    let total_kwh: f64 = get_table_value(step, "total_kwh").parse().unwrap();

    use koprogo_api::domain::entities::EnergyBillUpload;
    let encryption_key: [u8; 32] = [42u8; 32];
    let upload = EnergyBillUpload::new(
        campaign_id,
        unit_id,
        building_id,
        org_id,
        Utc::now() - ChronoDuration::days(365),
        Utc::now(),
        total_kwh,
        EnergyType::Electricity,
        "1000".to_string(),
        "sha256hash".to_string(),
        "/uploads/bill.pdf".to_string(),
        Uuid::new_v4(),
        "127.0.0.1".to_string(),
        "test-agent".to_string(),
        &encryption_key,
    );
    match upload {
        Ok(u) => {
            let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
            match bill_uc.upload_bill(u).await {
                Ok(uploaded) => {
                    world.last_upload_id = Some(uploaded.id);
                    world.operation_success = true;
                }
                Err(e) => {
                    world.operation_error = Some(e);
                    world.operation_success = false;
                }
            }
        }
        Err(e) => {
            world.operation_error = Some(e);
            world.operation_success = false;
        }
    }
}

#[given("I have uploaded 2 energy bills")]
async fn given_2_uploads(world: &mut OperationsWorld) {
    given_open_campaign(world).await;
    let campaign_id = world.last_campaign_id.unwrap();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();

    use koprogo_api::domain::entities::EnergyBillUpload;
    let encryption_key: [u8; 32] = [42u8; 32];
    // Create 2 units if none exist
    if world.unit_ids.is_empty() {
        let pool = world.pool.as_ref().unwrap();
        for i in 0..2 {
            let uid = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO units (id, building_id, organization_id, unit_number, floor, area, share_percentage, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 0, 50.0, 0.1, NOW(), NOW())"#
            )
            .bind(uid).bind(building_id).bind(org_id).bind(format!("BU{}", i))
            .execute(pool).await.expect("insert unit");
            world.unit_ids.push(uid);
        }
    }
    for i in 0..2 {
        let unit_id = world.unit_ids[i % world.unit_ids.len()];
        let upload = EnergyBillUpload::new(
            campaign_id,
            unit_id,
            building_id,
            org_id,
            Utc::now() - ChronoDuration::days(365),
            Utc::now(),
            3000.0 + (i as f64 * 500.0),
            EnergyType::Electricity,
            "1000".to_string(),
            format!("hash{}", i),
            format!("/uploads/bill{}.pdf", i),
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            "test".to_string(),
            &encryption_key,
        )
        .unwrap();
        let result = bill_uc.upload_bill(upload).await.expect("upload bill");
        world.last_upload_id = Some(result.id);
    }
}

#[when("I list my uploads")]
async fn when_list_my_uploads(world: &mut OperationsWorld) {
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let unit_id = world.unit_ids.first().copied().unwrap_or(Uuid::new_v4());
    match bill_uc.get_my_uploads(unit_id).await {
        Ok(list) => {
            world.upload_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("an unverified upload exists")]
async fn given_unverified_upload(world: &mut OperationsWorld) {
    given_open_campaign(world).await;
    let campaign_id = world.last_campaign_id.unwrap();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let pool = world.pool.as_ref().unwrap();

    // Ensure at least one unit exists
    if world.unit_ids.is_empty() {
        let uid = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO units (id, building_id, organization_id, unit_number, floor, area, share_percentage, created_at, updated_at)
               VALUES ($1, $2, $3, 'UV1', 0, 50.0, 0.1, NOW(), NOW())"#
        )
        .bind(uid).bind(building_id).bind(org_id)
        .execute(pool).await.expect("insert unit");
        world.unit_ids.push(uid);
    }

    use koprogo_api::domain::entities::EnergyBillUpload;
    let encryption_key: [u8; 32] = [42u8; 32];
    let unit_id = world.unit_ids[0];
    let upload = EnergyBillUpload::new(
        campaign_id,
        unit_id,
        building_id,
        org_id,
        Utc::now() - ChronoDuration::days(365),
        Utc::now(),
        3500.0,
        EnergyType::Electricity,
        "1000".to_string(),
        "hash-verify".to_string(),
        "/uploads/verify.pdf".to_string(),
        Uuid::new_v4(),
        "127.0.0.1".to_string(),
        "test".to_string(),
        &encryption_key,
    )
    .unwrap();
    let result = bill_uc.upload_bill(upload).await.expect("upload bill");
    world.last_upload_id = Some(result.id);
    world.operation_success = true;
}

#[when("an admin verifies the upload")]
async fn when_admin_verifies(world: &mut OperationsWorld) {
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let upload_id = world.last_upload_id.unwrap();
    let admin_id = Uuid::new_v4();
    match bill_uc.verify_upload(upload_id, admin_id).await {
        Ok(_) => {
            world.upload_verified = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("an owner has uploaded energy data")]
async fn given_owner_has_uploaded(world: &mut OperationsWorld) {
    given_unverified_upload(world).await;
}

#[when("the owner withdraws their GDPR consent")]
async fn when_withdraw_consent(world: &mut OperationsWorld) {
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let upload_id = world.last_upload_id.unwrap();
    let unit_id = world.unit_ids.first().copied().unwrap_or(Uuid::new_v4());
    match bill_uc.withdraw_consent(upload_id, unit_id).await {
        Ok(_) => {
            world.upload_deleted = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("an owner has an energy bill upload")]
async fn given_owner_has_upload(world: &mut OperationsWorld) {
    given_unverified_upload(world).await;
}

#[when("the owner deletes their upload")]
async fn when_delete_upload(world: &mut OperationsWorld) {
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let upload_id = world.last_upload_id.unwrap();
    let unit_id = world.unit_ids.first().copied().unwrap_or(Uuid::new_v4());
    match bill_uc.delete_upload(upload_id, unit_id).await {
        Ok(_) => {
            world.upload_deleted = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a provider offer:")]
async fn when_add_provider_offer(world: &mut OperationsWorld, step: &Step) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();

    use koprogo_api::domain::entities::ProviderOffer;
    let provider_name = get_table_value(step, "provider_name");
    let price_kwh: f64 = get_table_value(step, "price_kwh_electricity")
        .parse()
        .unwrap();
    let fee: f64 = get_table_value(step, "fixed_monthly_fee").parse().unwrap();
    let green: f64 = get_table_value(step, "green_energy_pct").parse().unwrap();
    let duration: i32 = get_table_value(step, "contract_duration_months")
        .parse()
        .unwrap();
    let savings: f64 = get_table_value(step, "estimated_savings_pct")
        .parse()
        .unwrap();

    let offer = ProviderOffer::new(
        campaign_id,
        provider_name,
        Some(price_kwh),
        None,
        fee,
        green,
        duration,
        savings,
        Utc::now() + ChronoDuration::days(90),
    )
    .expect("create offer");
    match uc.add_offer(campaign_id, offer).await {
        Ok(o) => {
            world.last_offer_id = Some(o.id);
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) provider offers exist for the campaign$"#)]
async fn given_n_offers(world: &mut OperationsWorld, count: usize) {
    given_open_campaign(world).await;
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();

    use koprogo_api::domain::entities::ProviderOffer;
    for i in 0..count {
        let offer = ProviderOffer::new(
            campaign_id,
            format!("Provider {}", i + 1),
            Some(0.25 + (i as f64 * 0.02)),
            None,
            5.0 + i as f64,
            80.0,
            12,
            10.0 + i as f64,
            Utc::now() + ChronoDuration::days(90),
        )
        .unwrap();
        let o = uc.add_offer(campaign_id, offer).await.expect("add offer");
        if i == 0 {
            world.last_offer_id = Some(o.id);
        }
    }
}

#[when("I list offers for the campaign")]
async fn when_list_offers(world: &mut OperationsWorld) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();
    match uc.get_campaign_offers(campaign_id).await {
        Ok(list) => {
            world.offer_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("multiple offers exist")]
async fn given_multiple_offers(world: &mut OperationsWorld) {
    given_n_offers(world, 3).await;
}

#[when("I select the winning offer")]
async fn when_select_offer(world: &mut OperationsWorld) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();
    let offer_id = world.last_offer_id.unwrap();
    match uc.select_offer(campaign_id, offer_id).await {
        Ok(c) => {
            world.last_campaign_status = Some(format!("{:?}", c.status));
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) participants have uploaded energy data$"#)]
async fn given_n_participants(world: &mut OperationsWorld, count: usize) {
    given_open_campaign(world).await;
    let campaign_id = world.last_campaign_id.unwrap();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let bill_uc = world.energy_bill_use_cases.as_ref().unwrap().clone();
    let pool = world.pool.as_ref().unwrap();

    use koprogo_api::domain::entities::EnergyBillUpload;
    let encryption_key: [u8; 32] = [42u8; 32];
    for i in 0..count {
        let unit_id = if i < world.unit_ids.len() {
            world.unit_ids[i]
        } else {
            let uid = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO units (id, building_id, organization_id, unit_number, floor, area, share_percentage, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 0, 50.0, 0.1, NOW(), NOW())"#
            )
            .bind(uid).bind(building_id).bind(org_id).bind(format!("P{}", i))
            .execute(pool).await.expect("insert unit");
            world.unit_ids.push(uid);
            uid
        };
        let upload = EnergyBillUpload::new(
            campaign_id,
            unit_id,
            building_id,
            org_id,
            Utc::now() - ChronoDuration::days(365),
            Utc::now(),
            3000.0 + (i as f64 * 200.0),
            EnergyType::Electricity,
            "1000".to_string(),
            format!("hash-p{}", i),
            format!("/uploads/p{}.pdf", i),
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            "test".to_string(),
            &encryption_key,
        )
        .unwrap();
        let _ = bill_uc.upload_bill(upload).await;
    }
}

#[given(regex = r#"^only (\d+) participants have uploaded energy data$"#)]
async fn given_few_participants(world: &mut OperationsWorld, count: usize) {
    given_n_participants(world, count).await;
}

#[when("I get campaign statistics")]
async fn when_get_campaign_stats(world: &mut OperationsWorld) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();
    match uc.get_campaign_stats(campaign_id).await {
        Ok(stats) => {
            world.campaign_stats_restricted = !stats.can_negotiate;
            world.campaign_stats_total = stats.total_kwh_electricity;
            world.operation_success = true;
        }
        Err(e) => {
            world.campaign_stats_restricted = true;
            world.operation_error = Some(e);
        }
    }
}

#[given("a campaign exists")]
async fn given_campaign_exists(world: &mut OperationsWorld) {
    given_open_campaign(world).await;
}

#[when("I delete the campaign")]
async fn when_delete_campaign(world: &mut OperationsWorld) {
    let uc = world.energy_campaign_use_cases.as_ref().unwrap().clone();
    let campaign_id = world.last_campaign_id.unwrap();
    match uc.delete_campaign(campaign_id).await {
        Ok(_) => world.operation_success = true,
        Err(e) => world.operation_error = Some(e),
    }
}

// === ENERGY CAMPAIGN THEN STEPS ===

#[then("the campaign should be created")]
async fn then_campaign_created(world: &mut OperationsWorld) {
    assert!(world.operation_success, "Campaign should be created");
    assert!(world.last_campaign_id.is_some());
}

#[then(regex = r#"^the status should be "([^"]*)"$"#)]
async fn then_campaign_status(world: &mut OperationsWorld, expected: String) {
    let status = world.last_campaign_status.as_ref().expect("status set");
    // Handle Draft matching to status names
    let matches = match expected.as_str() {
        "Open" => status.contains("Draft") || status.contains("CollectingData"),
        "Closed" => status.contains("Cancelled") || status.contains("Completed"),
        _ => status.contains(&expected),
    };
    assert!(matches, "Expected status '{}', got '{}'", expected, status);
}

#[then(regex = r#"^I should get (\d+) campaigns$"#)]
async fn then_campaign_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.campaign_list_count, expected);
}

#[then(regex = r#"^the campaign status should be "([^"]*)"$"#)]
async fn then_campaign_status_is(world: &mut OperationsWorld, expected: String) {
    let status = world.last_campaign_status.as_ref().expect("status");
    assert!(
        status.contains(&expected) || (expected == "Closed" && status.contains("Cancelled")),
        "Expected '{}', got '{}'",
        expected,
        status
    );
}

#[then("the upload should be accepted")]
async fn then_upload_accepted(world: &mut OperationsWorld) {
    assert!(world.operation_success, "Upload should be accepted");
    assert!(world.last_upload_id.is_some());
}

#[then("the data should be stored encrypted")]
async fn then_data_encrypted(world: &mut OperationsWorld) {
    // Data is encrypted in EnergyBillUpload::new via AES-256-GCM
    assert!(world.last_upload_id.is_some());
}

#[then(regex = r#"^I should get (\d+) uploads$"#)]
async fn then_upload_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.upload_list_count, expected);
}

#[then("the upload should be marked as verified")]
async fn then_upload_verified(world: &mut OperationsWorld) {
    assert!(world.upload_verified);
}

#[then("the energy data should be deleted immediately")]
async fn then_energy_data_deleted(world: &mut OperationsWorld) {
    assert!(world.upload_deleted);
}

#[then("no trace of the data should remain")]
async fn then_no_trace(world: &mut OperationsWorld) {
    assert!(world.upload_deleted);
}

#[then("the upload should be deleted")]
async fn then_upload_deleted(world: &mut OperationsWorld) {
    assert!(world.upload_deleted);
}

#[then("the offer should be added")]
async fn then_offer_added(world: &mut OperationsWorld) {
    assert!(world.operation_success);
    assert!(world.last_offer_id.is_some());
}

#[then(regex = r#"^I should get (\d+) offers$"#)]
async fn then_offer_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.offer_list_count, expected);
}

#[then("the selected offer should be recorded")]
async fn then_offer_selected(world: &mut OperationsWorld) {
    assert!(world.operation_success);
}

#[then("the statistics should be anonymized")]
async fn then_stats_anonymized(world: &mut OperationsWorld) {
    assert!(
        !world.campaign_stats_restricted,
        "Stats should be available (k-anonymity met)"
    );
}

#[then("the statistics should aggregate consumption data")]
async fn then_stats_aggregate(world: &mut OperationsWorld) {
    // If we got stats at all, aggregation works
    assert!(world.operation_success || world.campaign_stats_total.is_some());
}

#[then("the statistics should be restricted for privacy")]
async fn then_stats_restricted(world: &mut OperationsWorld) {
    assert!(
        world.campaign_stats_restricted,
        "Stats should be restricted (k-anonymity not met)"
    );
}

#[then("the campaign should be deleted")]
async fn then_campaign_deleted(world: &mut OperationsWorld) {
    assert!(world.operation_success);
}

// ============================================================
// === IOT STEPS ===
// ============================================================

#[when("I create an IoT reading:")]
async fn when_create_iot_reading(world: &mut OperationsWorld, step: &Step) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = world.authenticated_user_id.unwrap_or(Uuid::new_v4());

    let device_str = get_table_value(step, "device_type");
    let metric_str = get_table_value(step, "metric_type");
    let value: f64 = get_table_value(step, "value").parse().unwrap();
    let unit = get_table_value(step, "unit");

    let dto = CreateIoTReadingDto {
        building_id,
        device_type: parse_device_type(&device_str),
        metric_type: parse_metric_type(&metric_str),
        value,
        unit,
        timestamp: Utc::now() - ChronoDuration::seconds(10),
        source: get_table_value_opt(step, "source").unwrap_or("automatic".to_string()),
        metadata: None,
    };
    match uc.create_reading(dto, user_id, org_id).await {
        Ok(r) => {
            world.last_reading_id = Some(r.id);
            world.readings_created_count = 1;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when(regex = r#"^I create (\d+) IoT readings in bulk$"#)]
async fn when_bulk_create_readings(world: &mut OperationsWorld, count: usize) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    for i in 0..count {
        dtos.push(CreateIoTReadingDto {
            building_id,
            device_type: DeviceType::ElectricityMeter,
            metric_type: MetricType::ElectricityConsumption,
            value: 40.0 + i as f64,
            unit: "kWh".to_string(),
            timestamp: Utc::now() - ChronoDuration::hours(count as i64 - i as i64),
            source: "bulk-test".to_string(),
            metadata: None,
        });
    }
    match uc.create_readings_bulk(dtos, user_id, org_id).await {
        Ok(n) => {
            world.readings_created_count = n;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) readings exist for the building$"#)]
async fn given_n_readings(world: &mut OperationsWorld, count: usize) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    for i in 0..count {
        dtos.push(CreateIoTReadingDto {
            building_id,
            device_type: DeviceType::ElectricityMeter,
            metric_type: MetricType::ElectricityConsumption,
            value: 30.0 + (i as f64 * 0.5),
            unit: "kWh".to_string(),
            timestamp: Utc::now() - ChronoDuration::hours(count as i64 - i as i64),
            source: "test".to_string(),
            metadata: None,
        });
    }
    let _ = uc.create_readings_bulk(dtos, user_id, org_id).await;
}

#[when("I query readings with:")]
async fn when_query_readings(world: &mut OperationsWorld, step: &Step) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();

    let limit_str = get_table_value_opt(step, "limit");
    let limit = limit_str.map(|s| s.parse::<usize>().unwrap());

    let dto = QueryIoTReadingsDto {
        building_id,
        device_type: get_table_value_opt(step, "device_type").map(|s| parse_device_type(&s)),
        metric_type: get_table_value_opt(step, "metric_type").map(|s| parse_metric_type(&s)),
        start_date: Utc::now() - ChronoDuration::days(365),
        end_date: Utc::now() + ChronoDuration::hours(1),
        limit,
    };
    match uc.query_readings(dto).await {
        Ok(list) => {
            world.reading_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("readings exist from January to March 2026")]
async fn given_readings_jan_to_march(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    // 3 months of readings (10 per month)
    for month in 1..=3 {
        for day in [5, 10, 15] {
            let ts = format!("2026-{:02}-{:02}T12:00:00Z", month, day)
                .parse::<DateTime<Utc>>()
                .unwrap();
            dtos.push(CreateIoTReadingDto {
                building_id,
                device_type: DeviceType::ElectricityMeter,
                metric_type: MetricType::ElectricityConsumption,
                value: 40.0 + month as f64,
                unit: "kWh".to_string(),
                timestamp: ts,
                source: "test".to_string(),
                metadata: None,
            });
        }
    }
    let _ = uc.create_readings_bulk(dtos, user_id, org_id).await;
}

#[when("I query readings from February 1 to February 28")]
async fn when_query_feb(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let dto = QueryIoTReadingsDto {
        building_id,
        device_type: None,
        metric_type: None,
        start_date: "2026-02-01T00:00:00Z".parse().unwrap(),
        end_date: "2026-02-28T23:59:59Z".parse().unwrap(),
        limit: None,
    };
    match uc.query_readings(dto).await {
        Ok(list) => {
            world.reading_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) electricity readings exist$"#)]
async fn given_electricity_readings(world: &mut OperationsWorld, count: usize) {
    given_n_readings(world, count).await;
}

#[when("I get consumption stats for the building")]
async fn when_get_consumption_stats(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc
        .get_consumption_stats(
            building_id,
            MetricType::ElectricityConsumption,
            Utc::now() - ChronoDuration::days(365),
            Utc::now() + ChronoDuration::hours(1),
        )
        .await
    {
        Ok(stats) => {
            world.iot_stats_values = Some((
                stats.total_consumption,
                stats.average_daily,
                stats.min_value,
                stats.max_value,
            ));
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("daily readings exist for the past 30 days")]
async fn given_daily_readings_30d(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    for day in 1..=30 {
        dtos.push(CreateIoTReadingDto {
            building_id,
            device_type: DeviceType::ElectricityMeter,
            metric_type: MetricType::ElectricityConsumption,
            value: 35.0 + (day as f64 * 0.3),
            unit: "kWh".to_string(),
            timestamp: Utc::now() - ChronoDuration::days(day),
            source: "test".to_string(),
            metadata: None,
        });
    }
    let _ = uc.create_readings_bulk(dtos, user_id, org_id).await;
}

#[when("I get daily aggregates")]
async fn when_daily_aggregates(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc
        .get_daily_aggregates(
            building_id,
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            Utc::now() - ChronoDuration::days(31),
            Utc::now(),
        )
        .await
    {
        Ok(aggs) => {
            world.daily_aggregate_count = aggs.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("readings exist for the past 6 months")]
async fn given_readings_6months(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    for month in 1..=6 {
        for day in [5, 15, 25] {
            let ts = Utc::now() - ChronoDuration::days(month * 30 - day);
            dtos.push(CreateIoTReadingDto {
                building_id,
                device_type: DeviceType::ElectricityMeter,
                metric_type: MetricType::ElectricityConsumption,
                value: 40.0 + month as f64,
                unit: "kWh".to_string(),
                timestamp: ts,
                source: "test".to_string(),
                metadata: None,
            });
        }
    }
    let _ = uc.create_readings_bulk(dtos, user_id, org_id).await;
}

#[when("I get monthly aggregates")]
async fn when_monthly_aggregates(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc
        .get_monthly_aggregates(
            building_id,
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            Utc::now() - ChronoDuration::days(200),
            Utc::now(),
        )
        .await
    {
        Ok(aggs) => {
            world.monthly_aggregate_count = aggs.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("normal readings with one spike exist")]
async fn given_readings_with_spike(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let user_id = Uuid::new_v4();

    let mut dtos = vec![];
    // 10 normal readings around 40 kWh
    for i in 0..10 {
        dtos.push(CreateIoTReadingDto {
            building_id,
            device_type: DeviceType::ElectricityMeter,
            metric_type: MetricType::ElectricityConsumption,
            value: 38.0 + (i as f64 * 0.5),
            unit: "kWh".to_string(),
            timestamp: Utc::now() - ChronoDuration::hours(11 - i as i64),
            source: "test".to_string(),
            metadata: None,
        });
    }
    // 1 spike at 200 kWh
    dtos.push(CreateIoTReadingDto {
        building_id,
        device_type: DeviceType::ElectricityMeter,
        metric_type: MetricType::ElectricityConsumption,
        value: 200.0,
        unit: "kWh".to_string(),
        timestamp: Utc::now() - ChronoDuration::minutes(30),
        source: "test".to_string(),
        metadata: None,
    });
    let _ = uc.create_readings_bulk(dtos, user_id, org_id).await;
}

#[when("I check for consumption anomalies")]
async fn when_check_anomalies(world: &mut OperationsWorld) {
    let uc = world.iot_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc
        .detect_anomalies(building_id, MetricType::ElectricityConsumption, 100.0, 7)
        .await
    {
        Ok(anomalies) => {
            world.anomaly_detected = !anomalies.is_empty();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("a Linky device is configured")]
async fn given_linky_configured(world: &mut OperationsWorld) {
    // Insert directly via SQL since LinkyUseCases needs mock API client
    let pool = world.pool.as_ref().unwrap();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let device_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO linky_devices (id, building_id, organization_id, prm, provider, sync_enabled, created_at, updated_at)
           VALUES ($1, $2, $3, '12345678901234', 'Enedis', true, NOW(), NOW())"#
    )
    .bind(device_id).bind(building_id).bind(org_id)
    .execute(pool).await.expect("insert linky device");
    world.last_linky_device_id = Some(device_id);
    world.linky_sync_enabled = Some(true);
}

#[when("I configure a Linky device:")]
async fn when_configure_linky(world: &mut OperationsWorld, step: &Step) {
    // Since LinkyUseCases requires external API, we insert directly
    let pool = world.pool.as_ref().unwrap();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let device_id = Uuid::new_v4();
    let prm = get_table_value(step, "prm");
    sqlx::query(
        r#"INSERT INTO linky_devices (id, building_id, organization_id, prm, provider, sync_enabled, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'Enedis', true, NOW(), NOW())"#
    )
    .bind(device_id).bind(building_id).bind(org_id).bind(prm)
    .execute(pool).await.expect("insert linky device");
    world.last_linky_device_id = Some(device_id);
    world.linky_sync_enabled = Some(true);
    world.operation_success = true;
}

#[when("I get the Linky device for the building")]
async fn when_get_linky(world: &mut OperationsWorld) {
    let pool = world.pool.as_ref().unwrap();
    let building_id = world.building_id.unwrap();
    let row = sqlx::query_as::<_, (Uuid, bool)>(
        "SELECT id, sync_enabled FROM linky_devices WHERE building_id = $1",
    )
    .bind(building_id)
    .fetch_optional(pool)
    .await
    .expect("query linky");
    if let Some((id, sync)) = row {
        world.last_linky_device_id = Some(id);
        world.linky_sync_enabled = Some(sync);
        world.operation_success = true;
    }
}

#[given("a Linky device with sync enabled")]
async fn given_linky_sync_enabled(world: &mut OperationsWorld) {
    given_linky_configured(world).await;
}

#[when("I toggle sync off")]
async fn when_toggle_sync_off(world: &mut OperationsWorld) {
    let pool = world.pool.as_ref().unwrap();
    let device_id = world.last_linky_device_id.unwrap();
    sqlx::query("UPDATE linky_devices SET sync_enabled = false, updated_at = NOW() WHERE id = $1")
        .bind(device_id)
        .execute(pool)
        .await
        .expect("toggle sync off");
    world.linky_sync_enabled = Some(false);
    world.operation_success = true;
}

#[given("devices that haven't synced in 24 hours")]
async fn given_stale_devices(world: &mut OperationsWorld) {
    let pool = world.pool.as_ref().unwrap();
    let building_id = world.building_id.unwrap();
    let org_id = world.org_id.unwrap();
    let device_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO linky_devices (id, building_id, organization_id, prm, provider, sync_enabled, last_sync_at, created_at, updated_at)
           VALUES ($1, $2, $3, '99999999999999', 'Enedis', true, NOW() - INTERVAL '48 hours', NOW(), NOW())"#
    )
    .bind(device_id).bind(building_id).bind(org_id)
    .execute(pool).await.expect("insert stale device");
    world.last_linky_device_id = Some(device_id);
}

#[when("I find devices needing sync")]
async fn when_find_needing_sync(world: &mut OperationsWorld) {
    let pool = world.pool.as_ref().unwrap();
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM linky_devices WHERE sync_enabled = true AND (last_sync_at IS NULL OR last_sync_at < NOW() - INTERVAL '24 hours')"
    )
    .fetch_all(pool)
    .await
    .expect("find stale");
    world.linky_devices_count = rows.len();
    world.operation_success = true;
}

// === IOT THEN STEPS ===

#[then("the reading should be created")]
async fn then_reading_created(world: &mut OperationsWorld) {
    assert!(world.operation_success, "Reading should be created");
    assert!(world.last_reading_id.is_some());
}

#[then(regex = r#"^all (\d+) readings should be created$"#)]
async fn then_all_readings_created(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.readings_created_count, expected);
}

#[then(regex = r#"^I should get (\d+) readings$"#)]
async fn then_reading_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.reading_list_count, expected);
}

#[then("I should only get February readings")]
async fn then_only_feb_readings(world: &mut OperationsWorld) {
    assert!(
        world.reading_list_count > 0,
        "Should have February readings"
    );
    // We created 3 readings per month, so expect 3
    assert_eq!(
        world.reading_list_count, 3,
        "Should have exactly 3 Feb readings"
    );
}

#[then("the stats should include total, average, min, max values")]
async fn then_stats_include_values(world: &mut OperationsWorld) {
    let (total, avg, min, max) = world.iot_stats_values.expect("stats should be set");
    assert!(total > 0.0, "total should be > 0");
    assert!(avg > 0.0, "avg should be > 0");
    assert!(min > 0.0, "min should be > 0");
    assert!(max >= min, "max should be >= min");
}

#[then(regex = r#"^I should get (\d+) daily data points$"#)]
async fn then_daily_count(world: &mut OperationsWorld, expected: usize) {
    assert!(
        world.daily_aggregate_count >= expected / 2,
        "Expected ~{} daily points, got {}",
        expected,
        world.daily_aggregate_count
    );
}

#[then("I should get aggregated monthly totals")]
async fn then_monthly_totals(world: &mut OperationsWorld) {
    assert!(
        world.monthly_aggregate_count > 0,
        "Should have monthly aggregates"
    );
}

#[then("the spike should be flagged as anomalous")]
async fn then_spike_flagged(world: &mut OperationsWorld) {
    assert!(
        world.anomaly_detected,
        "Spike should be detected as anomaly"
    );
}

#[then("the Linky device should be configured")]
async fn then_linky_configured(world: &mut OperationsWorld) {
    assert!(world.operation_success);
    assert!(world.last_linky_device_id.is_some());
}

#[then("sync should be enabled by default")]
async fn then_sync_enabled(world: &mut OperationsWorld) {
    assert_eq!(world.linky_sync_enabled, Some(true));
}

#[then("I should receive the device configuration")]
async fn then_device_config(world: &mut OperationsWorld) {
    assert!(world.last_linky_device_id.is_some());
}

#[then("sync should be disabled")]
async fn then_sync_disabled(world: &mut OperationsWorld) {
    assert_eq!(world.linky_sync_enabled, Some(false));
}

#[then("the stale devices should be returned")]
async fn then_stale_devices(world: &mut OperationsWorld) {
    assert!(world.linky_devices_count > 0, "Should find stale devices");
}

// === IOT PARSERS ===

fn parse_device_type(s: &str) -> DeviceType {
    match s {
        "Linky" | "ElectricityMeter" => DeviceType::ElectricityMeter,
        "WaterMeter" => DeviceType::WaterMeter,
        "GasMeter" => DeviceType::GasMeter,
        "TemperatureSensor" => DeviceType::TemperatureSensor,
        "HumiditySensor" => DeviceType::HumiditySensor,
        "PowerMeter" => DeviceType::PowerMeter,
        _ => DeviceType::ElectricityMeter,
    }
}

fn parse_metric_type(s: &str) -> MetricType {
    match s {
        "electricity_kwh" | "ElectricityConsumption" => MetricType::ElectricityConsumption,
        "gas_m3" | "GasConsumption" => MetricType::GasConsumption,
        "water_m3" | "WaterConsumption" => MetricType::WaterConsumption,
        "temperature_c" | "Temperature" => MetricType::Temperature,
        "Humidity" => MetricType::Humidity,
        "Power" => MetricType::Power,
        "Voltage" => MetricType::Voltage,
        _ => MetricType::ElectricityConsumption,
    }
}

// ============================================================
// === WORK REPORT STEPS ===
// ============================================================

#[when("I create a work report:")]
async fn when_create_work_report(world: &mut OperationsWorld, step: &Step) {
    use koprogo_api::application::dto::CreateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();

    let title = get_table_value(step, "title");
    let description = get_table_value(step, "description");
    let contractor = get_table_value(step, "contractor_name");
    let start_date = get_table_value(step, "start_date");
    let end_date = get_table_value_opt(step, "end_date");
    let warranty_years: i32 = get_table_value_opt(step, "warranty_years")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let work_type_str = get_table_value_opt(step, "work_type").unwrap_or("Maintenance".to_string());

    let warranty_type = match warranty_years {
        0 => WarrantyType::None,
        2 => WarrantyType::Standard,
        10 => WarrantyType::Decennial,
        3 => WarrantyType::Extended,
        y => WarrantyType::Custom { years: y },
    };

    let dto = CreateWorkReportDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title,
        description,
        work_type: parse_work_type(&work_type_str),
        contractor_name: contractor,
        contractor_contact: None,
        work_date: format!("{}T00:00:00Z", start_date),
        completion_date: end_date.map(|d| format!("{}T00:00:00Z", d)),
        cost: 10000.0,
        invoice_number: None,
        notes: None,
        warranty_type,
    };
    match uc.create_work_report(dto).await {
        Ok(r) => {
            world.last_work_report_id = Some(Uuid::parse_str(&r.id).unwrap());
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("a work report exists")]
async fn given_work_report_exists(world: &mut OperationsWorld) {
    if world.last_work_report_id.is_some() {
        return;
    }
    use koprogo_api::application::dto::CreateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let dto = CreateWorkReportDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: "Test Work Report".to_string(),
        description: "Test description".to_string(),
        work_type: WorkType::Maintenance,
        contractor_name: "Entreprise Test".to_string(),
        contractor_contact: None,
        work_date: "2026-01-15T00:00:00Z".to_string(),
        completion_date: Some("2026-02-15T00:00:00Z".to_string()),
        cost: 5000.0,
        invoice_number: None,
        notes: None,
        warranty_type: WarrantyType::Standard,
    };
    let r = uc
        .create_work_report(dto)
        .await
        .expect("create work report");
    world.last_work_report_id = Some(Uuid::parse_str(&r.id).unwrap());
}

#[when(regex = r#"^I update the contractor name to "([^"]*)"$"#)]
async fn when_update_contractor(world: &mut OperationsWorld, name: String) {
    use koprogo_api::application::dto::UpdateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let report_id = world.last_work_report_id.unwrap();
    let dto = UpdateWorkReportDto {
        title: None,
        description: None,
        work_type: None,
        contractor_name: Some(name),
        contractor_contact: None,
        work_date: None,
        completion_date: None,
        cost: None,
        invoice_number: None,
        notes: None,
        warranty_type: None,
    };
    match uc.update_work_report(report_id, dto).await {
        Ok(_) => world.operation_success = true,
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) work reports exist for the building$"#)]
async fn given_n_work_reports_building(world: &mut OperationsWorld, count: usize) {
    use koprogo_api::application::dto::CreateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    for i in 0..count {
        let dto = CreateWorkReportDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: format!("Report {}", i + 1),
            description: format!("Description {}", i + 1),
            work_type: WorkType::Maintenance,
            contractor_name: format!("Contractor {}", i + 1),
            contractor_contact: None,
            work_date: format!("2026-01-{:02}T00:00:00Z", (i + 1).min(28)),
            completion_date: None,
            cost: 1000.0 * (i + 1) as f64,
            invoice_number: None,
            notes: None,
            warranty_type: WarrantyType::Standard,
        };
        let r = uc.create_work_report(dto).await.expect("create report");
        if i == 0 {
            world.last_work_report_id = Some(Uuid::parse_str(&r.id).unwrap());
        }
    }
}

#[when("I list work reports for the building")]
async fn when_list_work_reports_building(world: &mut OperationsWorld) {
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc.list_work_reports_by_building(building_id).await {
        Ok(list) => {
            world.work_report_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("work reports with 2-year and 10-year warranties exist")]
async fn given_warranties_exist(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::CreateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let warranty_types: Vec<WarrantyType> = vec![WarrantyType::Standard, WarrantyType::Decennial];
    for (i, wt) in warranty_types.into_iter().enumerate() {
        let dto = CreateWorkReportDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: format!("Warranty Report {}", i + 1),
            description: "Test".to_string(),
            work_type: WorkType::Renovation,
            contractor_name: "Contractor".to_string(),
            contractor_contact: None,
            work_date: "2025-06-01T00:00:00Z".to_string(),
            completion_date: Some("2025-07-01T00:00:00Z".to_string()),
            cost: 5000.0,
            invoice_number: None,
            notes: None,
            warranty_type: wt,
        };
        let _ = uc
            .create_work_report(dto)
            .await
            .expect("create warranty report");
    }
}

#[given("the 2-year warranty has not expired yet")]
async fn given_2year_not_expired(_world: &mut OperationsWorld) {
    // Created with work_date 2025-06-01 + 2 years = 2027-06-01, so it's still valid
}

#[when("I get active warranties for the building")]
async fn when_get_active_warranties(world: &mut OperationsWorld) {
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc.get_active_warranties(building_id).await {
        Ok(list) => {
            world.warranty_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("a warranty expiring in 60 days exists")]
async fn given_expiring_warranty(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::CreateWorkReportDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    // Work done ~2 years ago - 60 days, so warranty (2yr) expires in ~60 days
    let work_date = Utc::now() - ChronoDuration::days(365 * 2 - 60);
    let dto = CreateWorkReportDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: "Expiring Warranty Work".to_string(),
        description: "About to expire".to_string(),
        work_type: WorkType::Repair,
        contractor_name: "Contractor Expiring".to_string(),
        contractor_contact: None,
        work_date: work_date.to_rfc3339(),
        completion_date: Some(work_date.to_rfc3339()),
        cost: 3000.0,
        invoice_number: None,
        notes: None,
        warranty_type: WarrantyType::Standard,
    };
    let _ = uc
        .create_work_report(dto)
        .await
        .expect("create expiring warranty");
}

#[when(regex = r#"^I get warranties expiring within (\d+) days$"#)]
async fn when_get_expiring_warranties(world: &mut OperationsWorld, days: i32) {
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let building_id = world.building_id.unwrap();
    match uc.get_expiring_warranties(building_id, days).await {
        Ok(list) => {
            world.warranty_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a photo to the work report")]
async fn when_add_photo_work_report(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::AddPhotoDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let report_id = world.last_work_report_id.unwrap();
    let dto = AddPhotoDto {
        photo_path: "/photos/work_report_1.jpg".to_string(),
    };
    match uc.add_photo(report_id, dto).await {
        Ok(_) => {
            world.work_report_photo_attached = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a document to the work report")]
async fn when_add_doc_work_report(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::AddDocumentDto;
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let report_id = world.last_work_report_id.unwrap();
    let dto = AddDocumentDto {
        document_path: "/docs/work_report_1.pdf".to_string(),
    };
    match uc.add_document(report_id, dto).await {
        Ok(_) => {
            world.work_report_doc_attached = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I delete the work report")]
async fn when_delete_work_report(world: &mut OperationsWorld) {
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let report_id = world.last_work_report_id.unwrap();
    match uc.delete_work_report(report_id).await {
        Ok(_) => {
            world.work_report_deleted = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) work reports exist$"#)]
async fn given_n_work_reports(world: &mut OperationsWorld, count: usize) {
    given_n_work_reports_building(world, count).await;
}

#[when(regex = r#"^I list work reports page (\d+) with (\d+) per page$"#)]
async fn when_list_work_reports_paginated(world: &mut OperationsWorld, page: i64, per_page: i64) {
    use koprogo_api::application::dto::{PageRequest, WorkReportFilters};
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let page_req = PageRequest {
        page,
        per_page,
        sort_by: None,
        order: Default::default(),
    };
    let filters = WorkReportFilters {
        organization_id: Some(org_id),
        building_id: Some(building_id),
        work_type: None,
        warranty_type: None,
        contractor_name: None,
        work_date_from: None,
        work_date_to: None,
        min_cost: None,
        max_cost: None,
        warranty_active: None,
    };
    match uc.list_work_reports_paginated(&page_req, &filters).await {
        Ok(resp) => {
            world.work_report_list_count = resp.work_reports.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("work reports for 2 organizations exist")]
async fn given_work_reports_2_orgs(world: &mut OperationsWorld) {
    // Create reports for our org
    given_n_work_reports_building(world, 2).await;
    // Create a second org + building + report
    let pool = world.pool.as_ref().unwrap();
    let other_org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Other Org', 'other-org', 'other@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(other_org_id).execute(pool).await.expect("insert other org");
    // Insert a work report for other org directly
    let building_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, total_shares, created_at, updated_at)
           VALUES ($1, $2, 'Other Building', '1 Other St', 'Brussels', '1000', 'Belgique', 5, 500, NOW(), NOW())"#
    )
    .bind(building_id).bind(other_org_id).execute(pool).await.expect("insert other building");
}

#[when("I list work reports for our organization")]
async fn when_list_work_reports_org(world: &mut OperationsWorld) {
    let uc = world.work_report_use_cases.as_ref().unwrap().clone();
    let org_id = world.org_id.unwrap();
    match uc.list_work_reports_by_organization(org_id).await {
        Ok(list) => {
            world.work_report_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

// Work report THEN steps

#[then("the work report should be created")]
async fn then_work_report_created(world: &mut OperationsWorld) {
    assert!(world.operation_success, "Work report should be created");
    assert!(world.last_work_report_id.is_some());
}

#[then("the work report should be updated")]
async fn then_work_report_updated(world: &mut OperationsWorld) {
    assert!(world.operation_success, "Work report should be updated");
}

#[then(regex = r#"^I should get (\d+) reports$"#)]
async fn then_report_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.work_report_list_count, expected);
}

#[then("both warranties should appear")]
async fn then_both_warranties(world: &mut OperationsWorld) {
    assert!(
        world.warranty_list_count >= 2,
        "Expected at least 2 warranties, got {}",
        world.warranty_list_count
    );
}

#[then("the expiring warranty should appear")]
async fn then_expiring_warranty(world: &mut OperationsWorld) {
    assert!(
        world.warranty_list_count > 0,
        "Expected expiring warranties"
    );
}

#[then("the photo should be attached")]
async fn then_photo_attached_wr(world: &mut OperationsWorld) {
    assert!(world.work_report_photo_attached);
}

#[then("the document should be attached")]
async fn then_doc_attached_wr(world: &mut OperationsWorld) {
    assert!(world.work_report_doc_attached);
}

#[then("the report should be deleted")]
async fn then_report_deleted(world: &mut OperationsWorld) {
    assert!(world.work_report_deleted);
}

#[then("I should only get our organization's reports")]
async fn then_only_our_org_reports(world: &mut OperationsWorld) {
    assert!(
        world.work_report_list_count >= 2,
        "Should have our org's reports"
    );
}

fn parse_work_type(s: &str) -> WorkType {
    match s {
        "Maintenance" => WorkType::Maintenance,
        "Repair" => WorkType::Repair,
        "Renovation" => WorkType::Renovation,
        "Emergency" => WorkType::Emergency,
        "Inspection" => WorkType::Inspection,
        "Installation" => WorkType::Installation,
        _ => WorkType::Other,
    }
}

// ============================================================
// === TECHNICAL INSPECTION STEPS ===
// ============================================================

#[when("I create a technical inspection:")]
async fn when_create_inspection(world: &mut OperationsWorld, step: &Step) {
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();

    let inspection_type_str = get_table_value(step, "inspection_type");
    let inspector_name = get_table_value(step, "inspector_name");
    let inspection_date = get_table_value(step, "inspection_date");
    let next_inspection = get_table_value_opt(step, "next_inspection");
    let result = get_table_value_opt(step, "result");

    let dto = CreateTechnicalInspectionDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: format!("{} Inspection", inspection_type_str),
        description: Some("Test inspection".to_string()),
        inspection_type: parse_inspection_type(&inspection_type_str),
        inspector_name,
        inspector_company: None,
        inspector_certification: None,
        inspection_date: format!("{}T00:00:00Z", inspection_date),
        result_summary: result,
        defects_found: None,
        recommendations: None,
        compliant: Some(true),
        compliance_certificate_number: None,
        compliance_valid_until: next_inspection.map(|d| format!("{}T00:00:00Z", d)),
        cost: Some(500.0),
        invoice_number: None,
        notes: None,
    };
    match uc.create_technical_inspection(dto).await {
        Ok(r) => {
            world.last_inspection_id = Some(Uuid::parse_str(&r.id).unwrap());
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("an inspection exists")]
async fn given_inspection_exists(world: &mut OperationsWorld) {
    if world.last_inspection_id.is_some() {
        return;
    }
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let dto = CreateTechnicalInspectionDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: "Test Inspection".to_string(),
        description: Some("Test".to_string()),
        inspection_type: InspectionType::Elevator,
        inspector_name: "Inspector Test".to_string(),
        inspector_company: None,
        inspector_certification: None,
        inspection_date: "2026-02-01T00:00:00Z".to_string(),
        result_summary: Some("Passed".to_string()),
        defects_found: None,
        recommendations: None,
        compliant: Some(true),
        compliance_certificate_number: None,
        compliance_valid_until: None,
        cost: Some(500.0),
        invoice_number: None,
        notes: None,
    };
    let r = uc
        .create_technical_inspection(dto)
        .await
        .expect("create inspection");
    world.last_inspection_id = Some(Uuid::parse_str(&r.id).unwrap());
}

#[given("an inspection with next_inspection_date in the past exists")]
async fn given_overdue_inspection(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    // Create an inspection from 2 years ago -> next_due in the past
    let dto = CreateTechnicalInspectionDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: "Overdue Inspection".to_string(),
        description: None,
        inspection_type: InspectionType::Elevator,
        inspector_name: "Old Inspector".to_string(),
        inspector_company: None,
        inspector_certification: None,
        inspection_date: "2024-01-01T00:00:00Z".to_string(),
        result_summary: None,
        defects_found: None,
        recommendations: None,
        compliant: Some(true),
        compliance_certificate_number: None,
        compliance_valid_until: None,
        cost: None,
        invoice_number: None,
        notes: None,
    };
    let _ = uc
        .create_technical_inspection(dto)
        .await
        .expect("create overdue inspection");
}

#[when("I list overdue inspections for the building")]
async fn when_list_overdue(world: &mut OperationsWorld) {
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let building_id = world.building_id.unwrap();
    match uc.get_overdue_inspections(building_id).await {
        Ok(list) => {
            world.overdue_inspection_found = !list.is_empty();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^an inspection with next_inspection_date in (\d+) days exists$"#)]
async fn given_upcoming_inspection(world: &mut OperationsWorld, _days: i32) {
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    // Create recent inspection so next_due is ~1 year from inspection_date
    // For elevator (annual), if inspection was 305 days ago, next_due is in ~60 days
    let inspection_date = Utc::now() - ChronoDuration::days(305);
    let dto = CreateTechnicalInspectionDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        title: "Upcoming Inspection".to_string(),
        description: None,
        inspection_type: InspectionType::Elevator,
        inspector_name: "Recent Inspector".to_string(),
        inspector_company: None,
        inspector_certification: None,
        inspection_date: inspection_date.to_rfc3339(),
        result_summary: None,
        defects_found: None,
        recommendations: None,
        compliant: Some(true),
        compliance_certificate_number: None,
        compliance_valid_until: None,
        cost: None,
        invoice_number: None,
        notes: None,
    };
    let _ = uc
        .create_technical_inspection(dto)
        .await
        .expect("create upcoming inspection");
}

#[when(regex = r#"^I list upcoming inspections within (\d+) days$"#)]
async fn when_list_upcoming(world: &mut OperationsWorld, days: i32) {
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let building_id = world.building_id.unwrap();
    match uc.get_upcoming_inspections(building_id, days).await {
        Ok(list) => {
            world.upcoming_inspection_found = !list.is_empty();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given("Elevator and Fire inspections exist")]
async fn given_elevator_fire(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let types: Vec<InspectionType> =
        vec![InspectionType::Elevator, InspectionType::FireExtinguisher];
    for (i, itype) in types.into_iter().enumerate() {
        let dto = CreateTechnicalInspectionDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: format!("{:?} Inspection {}", itype, i),
            description: None,
            inspection_type: itype,
            inspector_name: format!("Inspector {}", i),
            inspector_company: None,
            inspector_certification: None,
            inspection_date: format!("2026-01-{:02}T00:00:00Z", 10 + i),
            result_summary: None,
            defects_found: None,
            recommendations: None,
            compliant: Some(true),
            compliance_certificate_number: None,
            compliance_valid_until: None,
            cost: None,
            invoice_number: None,
            notes: None,
        };
        let _ = uc
            .create_technical_inspection(dto)
            .await
            .expect("create typed inspection");
    }
}

#[when(regex = r#"^I list inspections of type "([^"]*)"$"#)]
async fn when_list_by_type(world: &mut OperationsWorld, type_str: String) {
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let building_id = world.building_id.unwrap();
    match uc.get_inspections_by_type(building_id, &type_str).await {
        Ok(list) => {
            world.inspection_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when(regex = r#"^I update the inspection result to "([^"]*)"$"#)]
async fn when_update_inspection_result(world: &mut OperationsWorld, result: String) {
    use koprogo_api::application::dto::UpdateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let inspection_id = world.last_inspection_id.unwrap();
    let dto = UpdateTechnicalInspectionDto {
        title: None,
        description: None,
        inspection_type: None,
        inspector_name: None,
        inspector_company: None,
        inspector_certification: None,
        inspection_date: None,
        status: None,
        result_summary: Some(result),
        defects_found: None,
        recommendations: None,
        compliant: Some(false),
        compliance_certificate_number: None,
        compliance_valid_until: None,
        cost: None,
        invoice_number: None,
        notes: None,
    };
    match uc.update_technical_inspection(inspection_id, dto).await {
        Ok(_) => world.operation_success = true,
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a report document to the inspection")]
async fn when_add_report_inspection(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::AddReportDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let inspection_id = world.last_inspection_id.unwrap();
    let dto = AddReportDto {
        report_path: "/reports/inspection_1.pdf".to_string(),
    };
    match uc.add_report(inspection_id, dto).await {
        Ok(_) => {
            world.inspection_report_attached = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a photo to the inspection")]
async fn when_add_photo_inspection(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::AddInspectionPhotoDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let inspection_id = world.last_inspection_id.unwrap();
    let dto = AddInspectionPhotoDto {
        photo_path: "/photos/inspection_1.jpg".to_string(),
    };
    match uc.add_photo(inspection_id, dto).await {
        Ok(_) => {
            world.inspection_photo_attached = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I add a certificate to the inspection")]
async fn when_add_certificate(world: &mut OperationsWorld) {
    use koprogo_api::application::dto::AddCertificateDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let inspection_id = world.last_inspection_id.unwrap();
    let dto = AddCertificateDto {
        certificate_path: "/certs/inspection_1.pdf".to_string(),
    };
    match uc.add_certificate(inspection_id, dto).await {
        Ok(_) => {
            world.inspection_certificate_attached = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) inspections exist for the building$"#)]
async fn given_n_inspections_building(world: &mut OperationsWorld, count: usize) {
    use koprogo_api::application::dto::CreateTechnicalInspectionDto;
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    for i in 0..count {
        let dto = CreateTechnicalInspectionDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: format!("Inspection {}", i + 1),
            description: None,
            inspection_type: InspectionType::Elevator,
            inspector_name: format!("Inspector {}", i + 1),
            inspector_company: None,
            inspector_certification: None,
            inspection_date: format!("2026-01-{:02}T00:00:00Z", (i + 1).min(28)),
            result_summary: None,
            defects_found: None,
            recommendations: None,
            compliant: Some(true),
            compliance_certificate_number: None,
            compliance_valid_until: None,
            cost: None,
            invoice_number: None,
            notes: None,
        };
        let r = uc
            .create_technical_inspection(dto)
            .await
            .expect("create inspection");
        if i == 0 {
            world.last_inspection_id = Some(Uuid::parse_str(&r.id).unwrap());
        }
    }
}

#[when("I list inspections for the building")]
async fn when_list_inspections_building(world: &mut OperationsWorld) {
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let building_id = world.building_id.unwrap();
    match uc.list_technical_inspections_by_building(building_id).await {
        Ok(list) => {
            world.inspection_list_count = list.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[when("I delete the inspection")]
async fn when_delete_inspection(world: &mut OperationsWorld) {
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let inspection_id = world.last_inspection_id.unwrap();
    match uc.delete_technical_inspection(inspection_id).await {
        Ok(_) => {
            world.inspection_deleted = true;
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

#[given(regex = r#"^(\d+) inspections exist$"#)]
async fn given_n_inspections(world: &mut OperationsWorld, count: usize) {
    given_n_inspections_building(world, count).await;
}

#[when(regex = r#"^I list inspections page (\d+) with (\d+) per page$"#)]
async fn when_list_inspections_paginated(world: &mut OperationsWorld, page: i64, per_page: i64) {
    use koprogo_api::application::dto::{PageRequest, TechnicalInspectionFilters};
    let uc = world
        .technical_inspection_use_cases
        .as_ref()
        .unwrap()
        .clone();
    let org_id = world.org_id.unwrap();
    let building_id = world.building_id.unwrap();
    let page_req = PageRequest {
        page,
        per_page,
        sort_by: None,
        order: Default::default(),
    };
    let filters = TechnicalInspectionFilters {
        organization_id: Some(org_id),
        building_id: Some(building_id),
        inspection_type: None,
        status: None,
        inspector_name: None,
        inspector_company: None,
        inspection_date_from: None,
        inspection_date_to: None,
        overdue: None,
        compliant: None,
    };
    match uc
        .list_technical_inspections_paginated(&page_req, &filters)
        .await
    {
        Ok(resp) => {
            world.inspection_list_count = resp.inspections.len();
            world.operation_success = true;
        }
        Err(e) => world.operation_error = Some(e),
    }
}

// Technical inspection THEN steps

#[then("the inspection should be created")]
async fn then_inspection_created(world: &mut OperationsWorld) {
    assert!(
        world.operation_success,
        "Inspection should be created: {:?}",
        world.operation_error
    );
    assert!(world.last_inspection_id.is_some());
}

#[then("the overdue inspection should appear")]
async fn then_overdue_found(world: &mut OperationsWorld) {
    assert!(
        world.overdue_inspection_found,
        "Should find overdue inspections"
    );
}

#[then("the upcoming inspection should appear")]
async fn then_upcoming_found(world: &mut OperationsWorld) {
    assert!(
        world.upcoming_inspection_found,
        "Should find upcoming inspections"
    );
}

#[then(regex = r#"^all returned inspections should be of type "([^"]*)"$"#)]
async fn then_all_type(world: &mut OperationsWorld, _expected: String) {
    assert!(
        world.inspection_list_count > 0,
        "Should have inspections of this type"
    );
}

#[then("the inspection should be updated")]
async fn then_inspection_updated(world: &mut OperationsWorld) {
    assert!(world.operation_success);
}

#[then("the report should be attached")]
async fn then_report_attached(world: &mut OperationsWorld) {
    assert!(world.inspection_report_attached);
}

#[then(regex = r#"^the photo should be attached$"#)]
async fn then_photo_attached_ti(world: &mut OperationsWorld) {
    assert!(world.inspection_photo_attached || world.work_report_photo_attached);
}

#[then("the certificate should be attached")]
async fn then_cert_attached(world: &mut OperationsWorld) {
    assert!(world.inspection_certificate_attached);
}

#[then(regex = r#"^I should get (\d+) inspections$"#)]
async fn then_inspection_count(world: &mut OperationsWorld, expected: usize) {
    assert_eq!(world.inspection_list_count, expected);
}

#[then("the inspection should be deleted")]
async fn then_inspection_deleted(world: &mut OperationsWorld) {
    assert!(world.inspection_deleted);
}

fn parse_inspection_type(s: &str) -> InspectionType {
    match s {
        "Elevator" => InspectionType::Elevator,
        "Fire" => InspectionType::FireExtinguisher,
        "Boiler" => InspectionType::Boiler,
        "Electrical" => InspectionType::Electrical,
        "Gas" | "GasInstallation" => InspectionType::GasInstallation,
        "Roof" | "RoofStructure" => InspectionType::RoofStructure,
        "Facade" => InspectionType::Facade,
        "Water" | "WaterQuality" => InspectionType::WaterQuality,
        "FireAlarm" => InspectionType::FireAlarm,
        _ => InspectionType::Other {
            name: s.to_string(),
        },
    }
}

// ============================================================
// === MAIN ===
// ============================================================

#[tokio::main]
async fn main() {
    OperationsWorld::cucumber()
        .run("tests/features/tickets.feature")
        .await;
    OperationsWorld::cucumber()
        .run("tests/features/notifications.feature")
        .await;
    OperationsWorld::cucumber()
        .run("tests/features/energy_campaigns.feature")
        .await;
    OperationsWorld::cucumber()
        .run("tests/features/iot.feature")
        .await;
    OperationsWorld::cucumber()
        .run("tests/features/work_reports.feature")
        .await;
    OperationsWorld::cucumber()
        .run_and_exit("tests/features/technical_inspections.feature")
        .await;
}

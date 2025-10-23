use cucumber::{given, then, when, World};
use koprogo_api::application::dto::{CreateBuildingDto, CreateMeetingRequest, PcnReportRequest};
use koprogo_api::application::use_cases::{BuildingUseCases, DocumentUseCases, MeetingUseCases, PcnUseCases};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::infrastructure::database::{create_pool, PostgresBuildingRepository, PostgresDocumentRepository, PostgresExpenseRepository, PostgresMeetingRepository};
use koprogo_api::infrastructure::storage::FileStorage;
use uuid::Uuid;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};

#[derive(World)]
#[world(init = Self::new)]
pub struct BuildingWorld {
    use_cases: Option<Arc<BuildingUseCases>>,
    meeting_use_cases: Option<Arc<MeetingUseCases>>,
    document_use_cases: Option<Arc<DocumentUseCases>>,
    pcn_use_cases: Option<Arc<PcnUseCases>>,
    building_dto: Option<CreateBuildingDto>,
    last_result: Option<Result<String, String>>,
    _container: Option<ContainerAsync<Postgres>>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,
}

impl std::fmt::Debug for BuildingWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildingWorld")
            .field("use_cases", &"<BuildingUseCases>")
            .field("building_dto", &self.building_dto)
            .field("last_result", &self.last_result)
            .finish()
    }
}

impl BuildingWorld {
    async fn new() -> Self {
        Self {
            use_cases: None,
            meeting_use_cases: None,
            document_use_cases: None,
            pcn_use_cases: None,
            building_dto: None,
            last_result: None,
            _container: None,
            org_id: None,
            building_id: None,
        }
    }

    async fn setup_database(&mut self) {
        let postgres_container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

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

        // Create organization for FK
        let org_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
               VALUES ($1, 'Org BDD', 'org-bdd', 'bdd@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
        )
        .bind(org_id)
        .execute(&pool)
        .await
        .expect("insert org");

        let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
        let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
        let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
        let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));

        let building_use_cases = BuildingUseCases::new(building_repo.clone());
        let meeting_use_cases = MeetingUseCases::new(meeting_repo);
        let storage = FileStorage::new(std::env::temp_dir().join("koprogo_bdd_uploads")).expect("storage");
        let document_use_cases = DocumentUseCases::new(document_repo, storage);
        let pcn_use_cases = PcnUseCases::new(expense_repo);

        // Create one building for meeting/doc scenarios
        let building_id = {
            use koprogo_api::domain::entities::Building as DomBuilding;
            let b = DomBuilding::new(
                org_id,
                "BDD Building".to_string(),
                "1 Test St".to_string(),
                "Bruxelles".to_string(),
                "1000".to_string(),
                "Belgique".to_string(),
                5,
                Some(1999),
            ).unwrap();
            let bid = b.id;
            building_repo.create(&b).await.expect("create building");
            bid
        };

        self.use_cases = Some(Arc::new(building_use_cases));
        self.meeting_use_cases = Some(Arc::new(meeting_use_cases));
        self.document_use_cases = Some(Arc::new(document_use_cases));
        self.pcn_use_cases = Some(Arc::new(pcn_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
        self.building_id = Some(building_id);
    }
}

#[given("a coproperty management system")]
async fn given_system(world: &mut BuildingWorld) {
    world.setup_database().await;
}

#[when(regex = r#"^I create a building named "([^"]*)" in "([^"]*)"$"#)]
async fn when_create_building(world: &mut BuildingWorld, name: String, city: String) {
    let dto = CreateBuildingDto {
        organization_id: world.org_id.unwrap().to_string(),
        name: name.clone(),
        address: "123 Test St".to_string(),
        city: city.clone(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    world.building_dto = Some(dto.clone());

    if let Some(use_cases) = &world.use_cases {
        let result = use_cases.create_building(dto).await;
        world.last_result = Some(result.map(|b| b.id).map_err(|e| e.to_string()));
    }
}

#[then("the building should be created successfully")]
async fn then_building_created(world: &mut BuildingWorld) {
    assert!(world.last_result.is_some());
    assert!(world.last_result.as_ref().unwrap().is_ok());
}

#[then(regex = r#"^the building should be in "([^"]*)"$"#)]
async fn then_building_in_city(world: &mut BuildingWorld, city: String) {
    assert!(world.building_dto.is_some());
    assert_eq!(world.building_dto.as_ref().unwrap().city, city);
}

#[tokio::main]
async fn main() {
    BuildingWorld::cucumber()
        .run_and_exit("tests/features")
        .await;
}

// Meetings BDD
#[when(regex = r#"^I create a meeting titled "([^"]*)"$"#)]
async fn when_create_meeting(world: &mut BuildingWorld, title: String) {
    let org = world.org_id.unwrap();
    let bid = world.building_id.unwrap();
    let req = CreateMeetingRequest {
        organization_id: org,
        building_id: bid,
        meeting_type: koprogo_api::domain::entities::MeetingType::Ordinary,
        title,
        description: None,
        scheduled_date: chrono::Utc::now(),
        location: "Salle A".to_string(),
    };
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let res = uc.create_meeting(req).await;
    world.last_result = Some(res.map(|m| m.id.to_string()).map_err(|e| e.to_string()));
}

#[then("the meeting should exist")]
async fn then_meeting_exists(world: &mut BuildingWorld) {
    if let Some(Err(e)) = world.last_result.as_ref() {
        panic!("meeting creation error: {}", e);
    }
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

// Documents BDD
#[when(regex = r#"^I upload a document named "([^"]*)"$"#)]
async fn when_upload_document(world: &mut BuildingWorld, name: String) {
    use koprogo_api::domain::entities::DocumentType;
    let org = world.org_id.unwrap();
    let bid = world.building_id.unwrap();
    let content = b"BDD content".to_vec();
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc
        .upload_document(
            org,
            bid,
            DocumentType::Other,
            name,
            None,
            "bdd.txt".to_string(),
            content,
            "text/plain".to_string(),
            Uuid::new_v4(),
        )
        .await;
    world.last_result = Some(res.map(|d| d.id.to_string()).map_err(|e| e.to_string()));
}

#[then("the document should be stored")]
async fn then_document_stored(world: &mut BuildingWorld) {
    if let Some(Err(e)) = world.last_result.as_ref() {
        panic!("document upload error: {}", e);
    }
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

// PCN BDD
#[when("I generate a PCN report for the building")]
async fn when_generate_pcn(world: &mut BuildingWorld) {
    let bid = world.building_id.unwrap();
    let uc = world.pcn_use_cases.as_ref().unwrap();
    let req = PcnReportRequest { building_id: bid, start_date: None, end_date: None };
    let res = uc.generate_report(req).await;
    world.last_result = Some(res.map(|r| r.total_entries.to_string()).map_err(|e| e.to_string()));
}

#[then("the PCN report should be generated")]
async fn then_pcn_generated(world: &mut BuildingWorld) {
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

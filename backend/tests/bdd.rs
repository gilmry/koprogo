use cucumber::{given, then, when, World};
use koprogo_api::application::dto::{CreateBuildingDto, CreateMeetingRequest, PcnReportRequest, PageRequest, SortOrder, UpdateMeetingRequest, CompleteMeetingRequest};
use koprogo_api::application::use_cases::{BuildingUseCases, DocumentUseCases, MeetingUseCases, PcnUseCases};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::infrastructure::database::{create_pool, PostgresBuildingRepository, PostgresDocumentRepository, PostgresExpenseRepository, PostgresMeetingRepository, PostgresUserRepository, PostgresRefreshTokenRepository};
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
    auth_use_cases: Option<Arc<koprogo_api::application::use_cases::AuthUseCases>>,
    building_dto: Option<CreateBuildingDto>,
    last_result: Option<Result<String, String>>,
    _container: Option<ContainerAsync<Postgres>>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,
    last_count: Option<usize>,
    second_org_id: Option<Uuid>,
    second_building_id: Option<Uuid>,
    last_document_id: Option<Uuid>,
    last_meeting_id: Option<Uuid>,
    last_download: Option<(Vec<u8>, String, String)>,
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
            auth_use_cases: None,
            building_dto: None,
            last_result: None,
            _container: None,
            org_id: None,
            building_id: None,
            last_count: None,
            second_org_id: None,
            second_building_id: None,
            last_document_id: None,
            last_meeting_id: None,
            last_download: None,
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
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));

        let building_use_cases = BuildingUseCases::new(building_repo.clone());
        let meeting_use_cases = MeetingUseCases::new(meeting_repo);
        let storage = FileStorage::new(std::env::temp_dir().join("koprogo_bdd_uploads")).expect("storage");
        let document_use_cases = DocumentUseCases::new(document_repo, storage);
        let pcn_use_cases = PcnUseCases::new(expense_repo);
        let auth_use_cases = koprogo_api::application::use_cases::AuthUseCases::new(
            user_repo,
            refresh_repo,
            "test-secret-key".to_string(),
        );

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
        self.auth_use_cases = Some(Arc::new(auth_use_cases));
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
    match res {
        Ok(m) => {
            world.last_meeting_id = Some(m.id);
            world.last_result = Some(Ok(m.id.to_string()));
        }
        Err(e) => world.last_result = Some(Err(e)),
    }
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
    match res {
        Ok(d) => {
            world.last_document_id = Some(d.id);
            world.last_result = Some(Ok(d.id.to_string()));
        }
        Err(e) => world.last_result = Some(Err(e)),
    }
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

// Document linking + download
#[when("I link the document to the meeting")]
async fn when_link_document_to_meeting(world: &mut BuildingWorld) {
    use koprogo_api::application::dto::LinkDocumentToMeetingRequest;
    let doc_id = world.last_document_id.expect("uploaded document");
    let meeting_id = world.last_meeting_id.expect("created meeting");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc
        .link_to_meeting(doc_id, LinkDocumentToMeetingRequest { meeting_id })
        .await;
    world.last_result = Some(res.map(|d| d.id.to_string()).map_err(|e| e));
}

#[when("I download the last document")]
async fn when_download_last_document(world: &mut BuildingWorld) {
    let doc_id = world.last_document_id.expect("document id");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc.download_document(doc_id).await;
    match res {
        Ok(tuple) => world.last_download = Some(tuple),
        Err(e) => panic!("download error: {}", e),
    }
}

#[then("the downloaded content should not be empty")]
async fn then_download_not_empty(world: &mut BuildingWorld) {
    let (bytes, mime, filename) = world.last_download.as_ref().expect("downloaded");
    assert!(!bytes.is_empty());
    assert!(!mime.is_empty());
    assert!(!filename.is_empty());
}

// Meetings lifecycle
#[when(regex = r#"^I update the last meeting title to "([^"]*)" and location to "([^"]*)"$"#)]
async fn when_update_last_meeting(world: &mut BuildingWorld, title: String, location: String) {
    let id = world.last_meeting_id.expect("meeting id");
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let req = UpdateMeetingRequest {
        title: Some(title),
        description: None,
        scheduled_date: None,
        location: Some(location),
    };
    let res = uc.update_meeting(id, req).await;
    world.last_result = Some(res.map(|m| m.id.to_string()).map_err(|e| e));
}

#[then("the meeting update should succeed")]
async fn then_meeting_update_ok(world: &mut BuildingWorld) {
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

#[when(regex = r#"^I complete the last meeting with (\d+) attendees$"#)]
async fn when_complete_last_meeting(world: &mut BuildingWorld, attendees: i32) {
    let id = world.last_meeting_id.expect("meeting id");
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let req = CompleteMeetingRequest { attendees_count: attendees };
    let res = uc.complete_meeting(id, req).await;
    world.last_result = Some(res.map(|m| m.id.to_string()).map_err(|e| e));
}

#[then("the meeting completion should succeed")]
async fn then_meeting_complete_ok(world: &mut BuildingWorld) {
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

#[when("I cancel the last meeting")]
async fn when_cancel_last_meeting(world: &mut BuildingWorld) {
    let id = world.last_meeting_id.expect("meeting id");
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let res = uc.cancel_meeting(id).await;
    world.last_result = Some(res.map(|m| m.id.to_string()).map_err(|e| e));
}

#[then("the meeting cancellation should succeed")]
async fn then_meeting_cancel_ok(world: &mut BuildingWorld) {
    assert!(world.last_result.as_ref().map(|r| r.is_ok()).unwrap_or(false));
}

#[when("I list meetings for the building")]
async fn when_list_meetings_for_building(world: &mut BuildingWorld) {
    let bid = world.building_id.unwrap();
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let res = uc.list_meetings_by_building(bid).await.expect("list meetings");
    world.last_count = Some(res.len());
}

#[then("I should get at least 1 meeting")]
async fn then_at_least_one_meeting(world: &mut BuildingWorld) {
    assert!(world.last_count.unwrap_or(0) >= 1);
}

// Auth BDD
#[when("I register a new user and login")]
async fn when_register_and_login(world: &mut BuildingWorld) {
    use koprogo_api::application::dto::{RegisterRequest, LoginRequest};
    let auth = world.auth_use_cases.as_ref().unwrap();
    let email = format!("user+{}@test.com", Uuid::new_v4());
    let org = world.org_id.unwrap();
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "BDD".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: org,
    };
    let _ = auth.register(reg).await.expect("register");
    let login = LoginRequest { email: email.clone(), password: "Passw0rd!".to_string() };
    let res = auth.login(login).await;
    world.last_result = Some(res.map(|r| r.refresh_token).map_err(|e| e));
}

#[then("I receive an access token and a refresh token")]
async fn then_tokens_received(world: &mut BuildingWorld) {
    let auth = world.auth_use_cases.as_ref().unwrap();
    let refresh = world.last_result.as_ref().unwrap().as_ref().unwrap();
    // last_result holds refresh token; verify it's non-empty and decodable via refresh flow
    assert!(!refresh.is_empty());
    // Not refreshing here (done in next scenario), just ensure something was returned
    let _ = auth; // keep for future use
}

#[given("I have a valid refresh token")]
async fn given_have_refresh_token(world: &mut BuildingWorld) {
    when_register_and_login(world).await;
}

#[when("I refresh my session")]
async fn when_refresh_session(world: &mut BuildingWorld) {
    use koprogo_api::application::dto::RefreshTokenRequest;
    let auth = world.auth_use_cases.as_ref().unwrap();
    let refresh = world.last_result.as_ref().unwrap().as_ref().unwrap().clone();
    let res = auth.refresh_token(RefreshTokenRequest { refresh_token: refresh }).await;
    world.last_result = Some(res.map(|r| r.token).map_err(|e| e));
}

#[then("I receive a new access token")]
async fn then_new_access_token(world: &mut BuildingWorld) {
    let token = world.last_result.as_ref().unwrap().as_ref().unwrap();
    assert!(!token.is_empty());
}

// Pagination & Filtering BDD
#[when(regex = r#"^I list buildings page (\d+) with per_page (\d+) sorted by created_at desc$"#)]
async fn when_list_buildings_paginated(world: &mut BuildingWorld, page: i32, per_page: i32) {
    let page_req = PageRequest { page: i64::from(page), per_page: i64::from(per_page.min(100)), sort_by: Some("created_at".to_string()), order: SortOrder::Desc };
    let uc = world.use_cases.as_ref().unwrap();
    let (items, _total) = uc
        .list_buildings_paginated(&page_req, world.org_id)
        .await
        .expect("paginated list");
    world.last_count = Some(items.len());
}

#[then("I should get at least 1 building")]
async fn then_at_least_one_building(world: &mut BuildingWorld) {
    assert!(world.last_count.unwrap_or(0) >= 1);
}

// Multi-tenancy BDD
#[given("a coproperty management system with two organizations")]
async fn given_two_orgs(world: &mut BuildingWorld) {
    world.setup_database().await;

    // reuse same DB pool by re-building connection string
    let container = world._container.as_ref().unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.expect("host port");
    let pool = create_pool(&format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", host_port))
        .await
        .expect("pool");

    let second_org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org B', 'org-b', 'b@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(second_org_id)
    .execute(&pool)
    .await
    .expect("insert second org");

    // Create a building for second org
    let building_repo = PostgresBuildingRepository::new(pool.clone());
    use koprogo_api::domain::entities::Building as DomBuilding;
    let b = DomBuilding::new(
        second_org_id,
        "Second Org Building".to_string(),
        "2 Test St".to_string(),
        "Namur".to_string(),
        "5000".to_string(),
        "Belgique".to_string(),
        3,
        Some(2001),
    ).unwrap();
    let bid = b.id;
    building_repo.create(&b).await.expect("create second org building");
    world.second_org_id = Some(second_org_id);
    world.second_building_id = Some(bid);
}

#[when("I list buildings for the first organization")]
async fn when_list_buildings_for_first_org(world: &mut BuildingWorld) {
    let page_req = PageRequest { page: 1, per_page: 50, sort_by: Some("created_at".to_string()), order: SortOrder::Desc };
    let uc = world.use_cases.as_ref().unwrap();
    let (items, _total) = uc
        .list_buildings_paginated(&page_req, world.org_id)
        .await
        .expect("list first org");
    // Ensure none belong to second org (by id mismatch)
    let forbidden_id = world.second_building_id.unwrap();
    assert!(items.iter().all(|b| b.id != forbidden_id.to_string()));
}

#[then("I should not see buildings from the second organization")]
async fn then_no_cross_org(_world: &mut BuildingWorld) {
    // Assertion already enforced in previous step
    assert!(true);
}

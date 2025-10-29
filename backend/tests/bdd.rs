use cucumber::{given, then, when, World};
use koprogo_api::application::dto::{
    Claims, CompleteMeetingRequest, CreateBuildingDto, CreateExpenseDto, CreateMeetingRequest,
    LinkDocumentToExpenseRequest, LinkDocumentToMeetingRequest, LoginRequest, LoginResponse,
    PageRequest, PcnReportRequest, RefreshTokenRequest, RegisterRequest, SortOrder,
    UpdateMeetingRequest,
};
use koprogo_api::application::ports::{BuildingRepository, UserRoleRepository};
use koprogo_api::application::use_cases::{
    BuildingUseCases, DocumentUseCases, ExpenseUseCases, MeetingUseCases, PcnUseCases,
};
use koprogo_api::domain::entities::{ExpenseCategory, UserRole, UserRoleAssignment};
use koprogo_api::domain::i18n::{I18n, Language, TranslationKey};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresDocumentRepository, PostgresExpenseRepository,
    PostgresMeetingRepository, PostgresRefreshTokenRepository, PostgresUserRepository,
    PostgresUserRoleRepository,
};
use koprogo_api::infrastructure::pool::DbPool;
use koprogo_api::infrastructure::storage::{FileStorage, StorageProvider};
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(World)]
#[world(init = Self::new)]
pub struct BuildingWorld {
    use_cases: Option<Arc<BuildingUseCases>>,
    meeting_use_cases: Option<Arc<MeetingUseCases>>,
    document_use_cases: Option<Arc<DocumentUseCases>>,
    pcn_use_cases: Option<Arc<PcnUseCases>>,
    auth_use_cases: Option<Arc<koprogo_api::application::use_cases::AuthUseCases>>,
    expense_use_cases: Option<Arc<ExpenseUseCases>>,
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
    last_expense_id: Option<Uuid>,
    last_user_id: Option<Uuid>,
    user_role_repo: Option<Arc<dyn UserRoleRepository>>,
    multi_user_email: Option<String>,
    multi_user_password: Option<String>,
    multi_user_id: Option<Uuid>,
    secondary_role_id: Option<Uuid>,
    last_login_response: Option<LoginResponse>,
    last_token_claims: Option<Claims>,
    last_access_token: Option<String>,
    last_refresh_token: Option<String>,
    pool: Option<DbPool>,
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
            expense_use_cases: None,
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
            last_expense_id: None,
            last_user_id: None,
            user_role_repo: None,
            multi_user_email: None,
            multi_user_password: None,
            multi_user_id: None,
            secondary_role_id: None,
            last_login_response: None,
            last_token_claims: None,
            last_access_token: None,
            last_refresh_token: None,
            pool: None,
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
        let user_role_repo: Arc<dyn UserRoleRepository> =
            Arc::new(PostgresUserRoleRepository::new(pool.clone()));

        let building_use_cases = BuildingUseCases::new(building_repo.clone());
        let meeting_use_cases = MeetingUseCases::new(meeting_repo);
        let storage_root = std::env::temp_dir().join("koprogo_bdd_uploads");
        let storage: Arc<dyn StorageProvider> =
            Arc::new(FileStorage::new(&storage_root).expect("storage"));
        let document_use_cases = DocumentUseCases::new(document_repo, storage.clone());
        let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
        let expense_use_cases = ExpenseUseCases::new(expense_repo);
        let auth_use_cases = koprogo_api::application::use_cases::AuthUseCases::new(
            user_repo,
            refresh_repo,
            user_role_repo.clone(),
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
            )
            .unwrap();
            let bid = b.id;
            building_repo.create(&b).await.expect("create building");
            bid
        };

        self.use_cases = Some(Arc::new(building_use_cases));
        self.meeting_use_cases = Some(Arc::new(meeting_use_cases));
        self.document_use_cases = Some(Arc::new(document_use_cases));
        self.pcn_use_cases = Some(Arc::new(pcn_use_cases));
        self.auth_use_cases = Some(Arc::new(auth_use_cases));
        self.expense_use_cases = Some(Arc::new(expense_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
        self.building_id = Some(building_id);
        self.user_role_repo = Some(user_role_repo);
    }

    async fn ensure_second_org(&mut self) -> Uuid {
        if let Some(id) = self.second_org_id {
            return id;
        }

        let pool = self.pool.as_ref().expect("pool").clone();
        let second_org_id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
               VALUES ($1, 'Org Secondary', 'org-secondary', 'sec@org.com', 'starter', 10, 10, true, NOW(), NOW())"#,
        )
        .bind(second_org_id)
        .execute(&pool)
        .await
        .expect("insert secondary org");

        self.second_org_id = Some(second_org_id);
        second_org_id
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
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
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
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
}

// PCN BDD
#[when("I generate a PCN report for the building")]
async fn when_generate_pcn(world: &mut BuildingWorld) {
    let bid = world.building_id.unwrap();
    let uc = world.pcn_use_cases.as_ref().unwrap();
    let req = PcnReportRequest {
        building_id: bid,
        start_date: None,
        end_date: None,
    };
    let res = uc.generate_report(req).await;
    world.last_result = Some(
        res.map(|r| r.total_entries.to_string())
            .map_err(|e| e.to_string()),
    );
}

#[then("the PCN report should be generated")]
async fn then_pcn_generated(world: &mut BuildingWorld) {
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
}

// Document linking + download
#[when("I link the document to the meeting")]
async fn when_link_document_to_meeting(world: &mut BuildingWorld) {
    let doc_id = world.last_document_id.expect("uploaded document");
    let meeting_id = world.last_meeting_id.expect("created meeting");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc
        .link_to_meeting(doc_id, LinkDocumentToMeetingRequest { meeting_id })
        .await;
    world.last_result = Some(res.map(|d| d.id.to_string()));
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

#[when("I delete the last document")]
async fn when_delete_last_document(world: &mut BuildingWorld) {
    let doc_id = world.last_document_id.expect("doc id");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc.delete_document(doc_id).await.expect("delete doc");
    assert!(res);
}

#[when("I try to download the last document")]
async fn when_try_download_last_document(world: &mut BuildingWorld) {
    let doc_id = world.last_document_id.expect("doc id");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc.download_document(doc_id).await;
    world.last_result = Some(res.map(|_| String::new()));
}

#[then("the download should fail")]
async fn then_download_should_fail(world: &mut BuildingWorld) {
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_err())
        .unwrap_or(false));
}

// i18n
#[when(regex = r#"^I translate key "([^"]*)" to "([a-zA-Z]+)"$"#)]
async fn when_translate_key(_world: &mut BuildingWorld, key: String, lang: String) {
    let tk = match key.as_str() {
        "BuildingNameEmpty" => TranslationKey::BuildingNameEmpty,
        "TotalUnitsMustBePositive" => TranslationKey::TotalUnitsMustBePositive,
        "DescriptionEmpty" => TranslationKey::DescriptionEmpty,
        "AmountMustBePositive" => TranslationKey::AmountMustBePositive,
        "FirstNameEmpty" => TranslationKey::FirstNameEmpty,
        "LastNameEmpty" => TranslationKey::LastNameEmpty,
        "InvalidEmailFormat" => TranslationKey::InvalidEmailFormat,
        "NotFound" => TranslationKey::NotFound,
        "Unauthorized" => TranslationKey::Unauthorized,
        _ => TranslationKey::InternalError,
    };
    let lang = Language::from_code(&lang).unwrap_or_default();
    let txt = I18n::translate(tk, lang);
    // stash in last_result for assertion
    // store as Ok(text)
    // We cannot mutate world directly beyond fields; reuse last_result
    // use empty Err for none
    _world.last_result = Some(Ok(txt));
}

#[then(regex = r#"^the translation should be "([^"]*)"$"#)]
async fn then_translation_equals(world: &mut BuildingWorld, expected: String) {
    let got = world.last_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(got, &expected);
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
    world.last_result = Some(res.map(|m| m.id.to_string()));
}

#[then("the meeting update should succeed")]
async fn then_meeting_update_ok(world: &mut BuildingWorld) {
    if let Some(Err(e)) = world.last_result.as_ref() {
        panic!("meeting update error: {}", e);
    }
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
}

#[when(regex = r#"^I complete the last meeting with (\d+) attendees$"#)]
async fn when_complete_last_meeting(world: &mut BuildingWorld, attendees: i32) {
    let id = world.last_meeting_id.expect("meeting id");
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let req = CompleteMeetingRequest {
        attendees_count: attendees,
    };
    let res = uc.complete_meeting(id, req).await;
    world.last_result = Some(res.map(|m| m.id.to_string()));
}

#[then("the meeting completion should succeed")]
async fn then_meeting_complete_ok(world: &mut BuildingWorld) {
    if let Some(Err(e)) = world.last_result.as_ref() {
        panic!("meeting complete error: {}", e);
    }
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
}

#[when("I cancel the last meeting")]
async fn when_cancel_last_meeting(world: &mut BuildingWorld) {
    let id = world.last_meeting_id.expect("meeting id");
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let res = uc.cancel_meeting(id).await;
    world.last_result = Some(res.map(|m| m.id.to_string()));
}

#[then("the meeting cancellation should succeed")]
async fn then_meeting_cancel_ok(world: &mut BuildingWorld) {
    if let Some(Err(e)) = world.last_result.as_ref() {
        panic!("meeting cancel error: {}", e);
    }
    assert!(world
        .last_result
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false));
}

#[when("I list meetings for the building")]
async fn when_list_meetings_for_building(world: &mut BuildingWorld) {
    let bid = world.building_id.unwrap();
    let uc = world.meeting_use_cases.as_ref().unwrap();
    let res = uc
        .list_meetings_by_building(bid)
        .await
        .expect("list meetings");
    world.last_count = Some(res.len());
}

#[then("I should get at least 1 meeting")]
async fn then_at_least_one_meeting(world: &mut BuildingWorld) {
    assert!(world.last_count.unwrap_or(0) >= 1);
}

// Expenses + documents linking
#[given(regex = r#"^I create an expense of amount ([-+]?[0-9]*\.?[0-9]+)$"#)]
async fn given_create_expense(world: &mut BuildingWorld, amount: f64) {
    let uc = world.expense_use_cases.as_ref().unwrap();
    let dto = CreateExpenseDto {
        organization_id: world.org_id.unwrap().to_string(),
        building_id: world.building_id.unwrap().to_string(),
        category: ExpenseCategory::Maintenance,
        description: "BDD Expense".to_string(),
        amount,
        expense_date: chrono::Utc::now().to_rfc3339(),
        supplier: Some("Supplier".to_string()),
        invoice_number: Some("INV-BDD".to_string()),
    };
    let res = uc.create_expense(dto).await.expect("create expense");
    world.last_expense_id = Some(Uuid::parse_str(&res.id).unwrap());
}

#[when("I link the document to the expense")]
async fn when_link_document_to_expense(world: &mut BuildingWorld) {
    let doc_id = world.last_document_id.expect("doc id");
    let exp_id = world.last_expense_id.expect("expense id");
    let uc = world.document_use_cases.as_ref().unwrap();
    let res = uc
        .link_to_expense(doc_id, LinkDocumentToExpenseRequest { expense_id: exp_id })
        .await;
    world.last_result = Some(res.map(|d| d.id.to_string()));
}

// Documents access control: filter by org
#[when("I list documents for the second organization")]
async fn when_list_documents_for_second_org(world: &mut BuildingWorld) {
    let uc = world.document_use_cases.as_ref().unwrap();
    let page = koprogo_api::application::dto::PageRequest {
        page: 1,
        per_page: 50,
        sort_by: None,
        order: SortOrder::Desc,
    };
    let (docs, _total) = uc
        .list_documents_paginated(&page, world.second_org_id)
        .await
        .expect("list docs second org");
    world.last_count = Some(docs.len());
}

#[then(regex = r#"^I should get (\d+) documents$"#)]
async fn then_documents_count(world: &mut BuildingWorld, expected: i32) {
    assert_eq!(world.last_count.unwrap_or(0) as i32, expected);
}

// Expenses pagination
#[when(regex = r#"^I list expenses page (\d+) with per_page (\d+)$"#)]
async fn when_list_expenses_paginated(world: &mut BuildingWorld, page: i32, per_page: i32) {
    let uc = world.expense_use_cases.as_ref().unwrap();
    let page_req = PageRequest {
        page: i64::from(page),
        per_page: i64::from(per_page.min(100)),
        sort_by: Some("expense_date".to_string()),
        order: SortOrder::Desc,
    };
    let (items, _total) = uc
        .list_expenses_paginated(&page_req, world.org_id)
        .await
        .expect("list expenses");
    world.last_count = Some(items.len());
}

#[then("I should get at least 1 expense")]
async fn then_at_least_one_expense(world: &mut BuildingWorld) {
    assert!(world.last_count.unwrap_or(0) >= 1);
}

// Auth BDD
#[when("I register a new user and login")]
async fn when_register_and_login(world: &mut BuildingWorld) {
    let auth = world
        .auth_use_cases
        .as_ref()
        .expect("auth use cases")
        .clone();
    let email = format!("user+{}@test.com", Uuid::new_v4());
    let org = world.org_id.unwrap();
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "BDD".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org),
    };
    let _ = auth.register(reg).await.expect("register");
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let res: Result<LoginResponse, String> = auth.login(login).await;
    match res {
        Ok(r) => {
            world.last_user_id = Some(r.user.id);
            world.last_access_token = Some(r.token.clone());
            world.last_refresh_token = Some(r.refresh_token.clone());
            world.last_login_response = Some(r.clone());
            world.last_token_claims =
                Some(auth.verify_token(&r.token).expect("validate token claims"));
            world.last_result = Some(Ok(r.refresh_token.clone()));
        }
        Err(e) => world.last_result = Some(Err(e)),
    }
}

#[then("I receive an access token and a refresh token")]
async fn then_tokens_received(world: &mut BuildingWorld) {
    let access = world.last_access_token.as_ref().expect("access token");
    let refresh = world.last_refresh_token.as_ref().expect("refresh token");
    assert!(!access.is_empty());
    assert!(!refresh.is_empty());
}

#[given("I have a valid refresh token")]
async fn given_have_refresh_token(world: &mut BuildingWorld) {
    when_register_and_login(world).await;
}

#[when("I refresh my session")]
async fn when_refresh_session(world: &mut BuildingWorld) {
    let auth = world
        .auth_use_cases
        .as_ref()
        .expect("auth use cases")
        .clone();
    let refresh = world
        .last_refresh_token
        .as_ref()
        .expect("refresh token")
        .clone();
    let res = auth
        .refresh_token(RefreshTokenRequest {
            refresh_token: refresh,
        })
        .await;
    let response = res.expect("refresh response");
    world.last_access_token = Some(response.token.clone());
    world.last_refresh_token = Some(response.refresh_token.clone());
    world.last_login_response = Some(response.clone());
    world.last_token_claims = Some(auth.verify_token(&response.token).expect("claims"));
    world.last_result = Some(Ok(response.token));
}

#[then("I receive a new access token")]
async fn then_new_access_token(world: &mut BuildingWorld) {
    let token = world.last_access_token.as_ref().expect("new access token");
    assert!(!token.is_empty());
}

#[given("a user with multiple roles")]
async fn given_user_with_multiple_roles(world: &mut BuildingWorld) {
    if world.org_id.is_none() {
        world.setup_database().await;
    }

    let auth = world
        .auth_use_cases
        .as_ref()
        .expect("auth use cases")
        .clone();
    let org_primary = world.org_id.expect("primary org");
    let email = format!("multi+{}@test.com", Uuid::new_v4());
    let password = format!("Passw0rd-{}", Uuid::new_v4());

    world.multi_user_email = Some(email.clone());
    world.multi_user_password = Some(password.clone());

    let register = RegisterRequest {
        email: email.clone(),
        password: password.clone(),
        first_name: "Multi".to_string(),
        last_name: "Role".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_primary),
    };
    auth.register(register)
        .await
        .expect("register multi-role user");

    let login_response = auth
        .login(LoginRequest {
            email: email.clone(),
            password: password.clone(),
        })
        .await
        .expect("login multi-role");

    world.multi_user_id = Some(login_response.user.id);
    world.last_user_id = Some(login_response.user.id);
    world.last_login_response = Some(login_response.clone());
    world.last_access_token = Some(login_response.token.clone());
    world.last_refresh_token = Some(login_response.refresh_token.clone());
    world.last_token_claims = Some(
        auth.verify_token(&login_response.token)
            .expect("claims for multi-role login"),
    );

    let second_org_id = world.ensure_second_org().await;
    let role_repo = world
        .user_role_repo
        .as_ref()
        .expect("role repository")
        .clone();

    let secondary_assignment = role_repo
        .create(&UserRoleAssignment::new(
            login_response.user.id,
            UserRole::Accountant,
            Some(second_org_id),
            false,
        ))
        .await
        .expect("create secondary role");
    world.secondary_role_id = Some(secondary_assignment.id);

    // Re-login to fetch updated roles list
    let refreshed_login = auth
        .login(LoginRequest { email, password })
        .await
        .expect("login after adding role");

    world.last_login_response = Some(refreshed_login.clone());
    world.last_access_token = Some(refreshed_login.token.clone());
    world.last_refresh_token = Some(refreshed_login.refresh_token.clone());
    world.last_token_claims = Some(
        auth.verify_token(&refreshed_login.token)
            .expect("claims after secondary role login"),
    );
    world.last_result = Some(Ok(refreshed_login.user.id.to_string()));
}

#[when("I switch to the secondary role")]
async fn when_switch_to_secondary_role(world: &mut BuildingWorld) {
    let auth = world
        .auth_use_cases
        .as_ref()
        .expect("auth use cases")
        .clone();
    let user_id = world.multi_user_id.expect("multi-role user id");
    let role_id = world.secondary_role_id.expect("secondary role id");

    let response = auth
        .switch_active_role(user_id, role_id)
        .await
        .expect("switch role");

    world.last_login_response = Some(response.clone());
    world.last_access_token = Some(response.token.clone());
    world.last_refresh_token = Some(response.refresh_token.clone());
    world.last_token_claims = Some(
        auth.verify_token(&response.token)
            .expect("claims after switch"),
    );
    world.last_result = Some(Ok(role_id.to_string()));
}

#[then(regex = r#"^my active role should be \"([^\"]*)\"$"#)]
async fn then_active_role(world: &mut BuildingWorld, expected_role: String) {
    let response = world.last_login_response.as_ref().expect("login response");
    let active_role = response.user.active_role.as_ref().expect("active_role");
    assert_eq!(
        active_role.role.to_lowercase(),
        expected_role.to_lowercase()
    );
}

#[then("the user response should list multiple roles")]
async fn then_multiple_roles_listed(world: &mut BuildingWorld) {
    let response = world.last_login_response.as_ref().expect("login response");
    assert!(response.user.roles.len() >= 2);
}

#[then(regex = r#"^the JWT claims should use role \"([^\"]*)\"$"#)]
async fn then_claims_use_role(world: &mut BuildingWorld, expected: String) {
    let claims = world.last_token_claims.as_ref().expect("token claims");
    assert_eq!(claims.role.to_lowercase(), expected.to_lowercase());
}

#[then("the JWT claims should reference the selected role")]
async fn then_claims_reference_role(world: &mut BuildingWorld) {
    let claims = world.last_token_claims.as_ref().expect("token claims");
    let role_id = world.secondary_role_id.expect("secondary role id");
    assert_eq!(claims.role_id, Some(role_id));
}

// Pagination & Filtering BDD
#[when(regex = r#"^I list buildings page (\d+) with per_page (\d+) sorted by created_at desc$"#)]
async fn when_list_buildings_paginated(world: &mut BuildingWorld, page: i32, per_page: i32) {
    let page_req = PageRequest {
        page: i64::from(page),
        per_page: i64::from(per_page.min(100)),
        sort_by: Some("created_at".to_string()),
        order: SortOrder::Desc,
    };
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

// Aliases so Gherkin "And" following Given works for these steps
#[given(regex = r#"^I create a meeting titled \"([^\"]*)\"$"#)]
async fn given_create_meeting(world: &mut BuildingWorld, title: String) {
    when_create_meeting(world, title).await;
}

#[given(regex = r#"^I upload a document named \"([^\"]*)\"$"#)]
async fn given_upload_document(world: &mut BuildingWorld, name: String) {
    when_upload_document(world, name).await;
}

// Multi-tenancy BDD
#[given("a coproperty management system with two organizations")]
async fn given_two_orgs(world: &mut BuildingWorld) {
    if world.pool.is_none() {
        world.setup_database().await;
    }
    let pool = world.pool.as_ref().expect("pool").clone();

    let second_org_id = world.ensure_second_org().await;

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
    )
    .unwrap();
    let bid = b.id;
    building_repo
        .create(&b)
        .await
        .expect("create second org building");
    world.second_org_id = Some(second_org_id);
    world.second_building_id = Some(bid);
}

#[when("I list buildings for the first organization")]
async fn when_list_buildings_for_first_org(world: &mut BuildingWorld) {
    let page_req = PageRequest {
        page: 1,
        per_page: 50,
        sort_by: Some("created_at".to_string()),
        order: SortOrder::Desc,
    };
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
async fn then_no_cross_org(_world: &mut BuildingWorld) {}

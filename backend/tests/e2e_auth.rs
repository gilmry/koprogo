use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use koprogo_api::infrastructure::database::create_pool;
use koprogo_api::infrastructure::database::repositories::*;
use koprogo_api::infrastructure::storage::FileStorage;
use koprogo_api::application::use_cases::*;
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_app() -> (actix_web::web::Data<AppState>, ContainerAsync<Postgres>) {
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

    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");

    // repos
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));

    // use cases
    let jwt_secret = "e2e-secret".to_string();
    let auth_use_cases = AuthUseCases::new(user_repo, refresh_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo);
    let owner_use_cases = OwnerUseCases::new(owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let meeting_use_cases = MeetingUseCases::new(meeting_repo);
    let storage = FileStorage::new(std::env::temp_dir().join("koprogo_e2e_http_uploads")).expect("storage");
    let document_use_cases = DocumentUseCases::new(document_repo, storage);
    let pcn_use_cases = PcnUseCases::new(expense_repo);

    let app_state = actix_web::web::Data::new(AppState::new(
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        expense_use_cases,
        meeting_use_cases,
        document_use_cases,
        pcn_use_cases,
        pool.clone(),
    ));

    (app_state, postgres_container)
}

#[actix_web::test]
#[serial]
async fn protected_route_requires_jwt() {
    let (app_state, _container) = setup_app().await;

    let app = test::init_service(App::new().app_data(app_state.clone()).configure(configure_routes)).await;

    // Without Authorization → 401
    let req = test::TestRequest::get().uri("/api/v1/buildings").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
#[serial]
async fn protected_route_with_valid_jwt_succeeds() {
    let (app_state, container) = setup_app().await;
    let pool = app_state.pool.clone();

    // Create organization and user then login to get token
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org E2E', 'org-e2e', 'e2e@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("insert org");

    // Register + login
    use koprogo_api::application::dto::{RegisterRequest, LoginRequest};
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest { email: email.clone(), password: "Passw0rd!".to_string(), first_name: "E2E".to_string(), last_name: "User".to_string(), role: "syndic".to_string(), organization_id: Some(org_id) };
    let _ = app_state.auth_use_cases.register(reg).await.expect("register");
    let login = LoginRequest { email: email.clone(), password: "Passw0rd!".to_string() };
    let res = app_state.auth_use_cases.login(login).await.expect("login");
    let token = res.token;

    let app = test::init_service(App::new().app_data(app_state.clone()).configure(configure_routes)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .insert_header((actix_web::http::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Keep container alive until end of test
    drop(container);
}

#[actix_web::test]
#[serial]
async fn post_building_injects_org_from_jwt() {
    use actix_web::http::header;
    use serde::Deserialize;

    let (app_state, _container) = setup_app().await;
    let pool = app_state.pool.clone();

    // Create organization and user then login to get token (Org A)
    let org_a = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org A', 'orga', 'a@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_a)
    .execute(&pool)
    .await
    .expect("insert org");

    // Register + login for Org A
    use koprogo_api::application::dto::{RegisterRequest, LoginRequest};
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest { email: email.clone(), password: "Passw0rd!".to_string(), first_name: "E2E".to_string(), last_name: "User".to_string(), role: "syndic".to_string(), organization_id: Some(org_a) };
    let _ = app_state.auth_use_cases.register(reg).await.expect("register");
    let login = LoginRequest { email: email.clone(), password: "Passw0rd!".to_string() };
    let res = app_state.auth_use_cases.login(login).await.expect("login");
    let token = res.token;

    // Prepare POST body with a DIFFERENT organization_id to ensure server overrides it
    #[derive(Deserialize)]
    struct BuildingResp { id: String }

    let fake_org = Uuid::new_v4().to_string();
    let payload = serde_json::json!({
        "organization_id": fake_org,
        "name": "JWT Building",
        "address": "1 JWT St",
        "city": "Brussels",
        "postal_code": "1000",
        "country": "Belgium",
        "total_units": 5,
        "construction_year": 2000
    });

    let app = test::init_service(App::new().app_data(app_state.clone()).configure(configure_routes)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: BuildingResp = test::read_body_json(resp).await;
    let building_id = Uuid::parse_str(&body.id).expect("uuid");

    // Verify in DB that organization_id is Org A (not fake)
    let org_id: Uuid = sqlx::query_scalar("SELECT organization_id FROM buildings WHERE id = $1")
        .bind(building_id)
        .fetch_one(&pool)
        .await
        .expect("select org id");
    assert_eq!(org_id, org_a);
}

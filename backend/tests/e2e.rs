use actix_web::{test, App};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresDocumentRepository,
    PostgresExpenseRepository, PostgresOwnerRepository, PostgresRefreshTokenRepository,
    PostgresUnitRepository, PostgresUserRepository,
};
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use koprogo_api::infrastructure::storage::FileStorage;
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_test_db() -> (actix_web::web::Data<AppState>, ContainerAsync<Postgres>, Uuid) {
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

    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let meeting_repo = Arc::new(koprogo_api::infrastructure::database::repositories::PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));

    let jwt_secret = "test-secret-key".to_string();
    let auth_use_cases = AuthUseCases::new(user_repo, refresh_token_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo);
    let owner_use_cases = OwnerUseCases::new(owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let meeting_use_cases = MeetingUseCases::new(meeting_repo);
    let storage = FileStorage::new(std::env::temp_dir().join("koprogo_e2e_uploads")).expect("storage");
    let document_use_cases = DocumentUseCases::new(document_repo, storage);
    let pcn_use_cases = PcnUseCases::new(expense_repo);

    // Create an organization for FK references
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org Test', 'org-test', 'org@test.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("insert org");

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

    (app_state, postgres_container, org_id)
}

#[actix_web::test]
#[serial]
async fn test_health_endpoint() {
    let (app_state, _container, _org_id) = setup_test_db().await;

    let app = test::init_service(App::new().app_data(app_state).configure(configure_routes)).await;

    let req = test::TestRequest::get().uri("/api/v1/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let app = test::init_service(App::new().app_data(app_state).configure(configure_routes)).await;

    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building".to_string(),
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
#[serial]
async fn test_list_buildings_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let app = test::init_service(App::new().app_data(app_state).configure(configure_routes)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_validation_fails() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let app = test::init_service(App::new().app_data(app_state).configure(configure_routes)).await;

    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "".to_string(), // Invalid: empty name
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

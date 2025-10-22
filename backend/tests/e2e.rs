use actix_web::{test, App};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::database::{create_pool, *};
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;

async fn setup_test_db() -> (actix_web::web::Data<AppState>, ContainerAsync<Postgres>) {
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

    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo);
    let owner_use_cases = OwnerUseCases::new(owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo);

    let app_state = actix_web::web::Data::new(AppState::new(
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        expense_use_cases,
    ));

    (app_state, postgres_container)
}

#[actix_web::test]
#[serial]
async fn test_health_endpoint() {
    let (app_state, _container) = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_endpoint() {
    let (app_state, _container) = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes),
    )
    .await;

    let dto = CreateBuildingDto {
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
    let (app_state, _container) = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_validation_fails() {
    let (app_state, _container) = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes),
    )
    .await;

    let dto = CreateBuildingDto {
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

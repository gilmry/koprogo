use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::use_cases::BuildingUseCases;
use koprogo_api::infrastructure::database::{create_pool, PostgresBuildingRepository};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;

async fn setup_test_use_case() -> (BuildingUseCases, ContainerAsync<Postgres>) {
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

    let repo = Arc::new(PostgresBuildingRepository::new(pool));
    let use_cases = BuildingUseCases::new(repo);

    (use_cases, postgres_container)
}

#[tokio::test]
#[serial]
async fn test_building_use_case_create() {
    let (use_cases, _container) = setup_test_use_case().await;

    let dto = CreateBuildingDto {
        name: "Test Building".to_string(),
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    let result = use_cases.create_building(dto).await;
    assert!(result.is_ok());

    let building = result.unwrap();
    assert_eq!(building.name, "Test Building");
    assert_eq!(building.city, "Paris");
}

#[tokio::test]
#[serial]
async fn test_building_use_case_validation() {
    let (use_cases, _container) = setup_test_use_case().await;

    // Test with empty name (should fail)
    let dto = CreateBuildingDto {
        name: "".to_string(),
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    let result = use_cases.create_building(dto).await;
    assert!(result.is_err());
}

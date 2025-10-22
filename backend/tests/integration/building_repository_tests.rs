use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::domain::entities::Building;
use koprogo_api::infrastructure::database::{create_pool, PostgresBuildingRepository};
use serial_test::serial;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
#[serial]
async fn test_create_and_find_building() {
    let docker = Cli::default();
    let postgres_container = docker.run(Postgres::default());
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        postgres_container.get_host_port_ipv4(5432)
    );

    let pool = create_pool(&connection_string)
        .await
        .expect("Failed to create pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let repo = PostgresBuildingRepository::new(pool);

    // Create a building
    let building = Building::new(
        "Test Building".to_string(),
        "123 Test St".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
        10,
        Some(2000),
    )
    .unwrap();

    // Test create
    let created = repo.create(&building).await.unwrap();
    assert_eq!(created.name, "Test Building");

    // Test find by id
    let found = repo.find_by_id(building.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Building");

    // Test find all
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 1);

    // Test update
    let mut updated_building = building.clone();
    updated_building.update_info(
        "Updated Building".to_string(),
        "456 New St".to_string(),
        "Lyon".to_string(),
        "69001".to_string(),
    );
    let updated = repo.update(&updated_building).await.unwrap();
    assert_eq!(updated.name, "Updated Building");

    // Test delete
    let deleted = repo.delete(building.id).await.unwrap();
    assert!(deleted);

    let not_found = repo.find_by_id(building.id).await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
#[serial]
async fn test_find_all_buildings() {
    let docker = Cli::default();
    let postgres_container = docker.run(Postgres::default());
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        postgres_container.get_host_port_ipv4(5432)
    );

    let pool = create_pool(&connection_string)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let repo = PostgresBuildingRepository::new(pool);

    // Create multiple buildings
    for i in 1..=3 {
        let building = Building::new(
            format!("Building {}", i),
            format!("{} Test St", i),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
            10,
            Some(2000),
        )
        .unwrap();
        repo.create(&building).await.unwrap();
    }

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 3);
}

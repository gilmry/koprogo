use koprogo_api::application::ports::{OwnerRepository, UnitOwnerRepository, UnitRepository};
use koprogo_api::domain::entities::unit::UnitType;
use koprogo_api::domain::entities::{Owner, Unit, UnitOwner};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresOwnerRepository, PostgresUnitOwnerRepository, PostgresUnitRepository,
};
use serial_test::serial;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_test_db() -> (
    PostgresUnitOwnerRepository,
    PostgresUnitRepository,
    PostgresOwnerRepository,
    ContainerAsync<Postgres>,
    Uuid, // org_id
    Uuid, // building_id
) {
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

    // Create test organization and building
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Test Org', 'test-org', 'test@test.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("Failed to insert organization");

    let building_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at)
           VALUES ($1, $2, 'Test Building', '123 Main St', 'Brussels', '1000', 'Belgium', 10, 2020, NOW(), NOW())"#
    )
    .bind(building_id)
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("Failed to insert building");

    let unit_owner_repo = PostgresUnitOwnerRepository::new(pool.clone());
    let unit_repo = PostgresUnitRepository::new(pool.clone());
    let owner_repo = PostgresOwnerRepository::new(pool.clone());

    (
        unit_owner_repo,
        unit_repo,
        owner_repo,
        postgres_container,
        org_id,
        building_id,
    )
}

// Helper to create test unit
async fn create_test_unit(
    unit_repo: &PostgresUnitRepository,
    org_id: Uuid,
    building_id: Uuid,
    number: &str,
) -> Unit {
    let unit = Unit::new(
        org_id,
        building_id,
        number.to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();

    unit_repo.create(&unit).await.unwrap()
}

// Helper to create test owner
async fn create_test_owner(owner_repo: &PostgresOwnerRepository, org_id: Uuid) -> Owner {
    let owner = Owner::new(
        org_id,
        "Jean".to_string(),
        "Dupont".to_string(),
        format!("test-{}@example.com", Uuid::new_v4()),
        Some("+32 123 456 789".to_string()),
        "Rue de la Loi 1".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
    )
    .unwrap();

    owner_repo.create(&owner).await.unwrap()
}

#[tokio::test]
#[serial]
async fn test_create_unit_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A101").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let unit_owner = UnitOwner::new(unit.id, owner.id, 1.0, true).unwrap();

    let result = repo.create(&unit_owner).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.unit_id, unit.id);
    assert_eq!(created.owner_id, owner.id);
    assert_eq!(created.ownership_percentage, 1.0);
    assert!(created.is_primary_contact);
}

#[tokio::test]
#[serial]
async fn test_find_by_id() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A102").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let unit_owner = UnitOwner::new(unit.id, owner.id, 0.5, false).unwrap();
    let created = repo.create(&unit_owner).await.unwrap();

    let found = repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_some());

    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.ownership_percentage, 0.5);
}

#[tokio::test]
#[serial]
async fn test_find_current_owners_by_unit() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A103").await;
    let owner1 = create_test_owner(&owner_repo, org_id).await;
    let owner2 = create_test_owner(&owner_repo, org_id).await;

    // Add two active owners
    let uo1 = UnitOwner::new(unit.id, owner1.id, 0.6, true).unwrap();
    let uo2 = UnitOwner::new(unit.id, owner2.id, 0.4, false).unwrap();

    repo.create(&uo1).await.unwrap();
    repo.create(&uo2).await.unwrap();

    let owners = repo.find_current_owners_by_unit(unit.id).await.unwrap();
    assert_eq!(owners.len(), 2);

    // Should be ordered by is_primary_contact DESC
    assert!(owners[0].is_primary_contact);
    assert!(!owners[1].is_primary_contact);
}

#[tokio::test]
#[serial]
async fn test_find_current_units_by_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit1 = create_test_unit(&unit_repo, org_id, building_id, "A104").await;
    let unit2 = create_test_unit(&unit_repo, org_id, building_id, "A105").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    // Owner owns two units
    let uo1 = UnitOwner::new(unit1.id, owner.id, 1.0, true).unwrap();
    let uo2 = UnitOwner::new(unit2.id, owner.id, 1.0, true).unwrap();

    repo.create(&uo1).await.unwrap();
    repo.create(&uo2).await.unwrap();

    let units = repo.find_current_units_by_owner(owner.id).await.unwrap();
    assert_eq!(units.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_update_unit_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A106").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let mut unit_owner = UnitOwner::new(unit.id, owner.id, 0.5, false).unwrap();
    let created = repo.create(&unit_owner).await.unwrap();

    // Update percentage
    unit_owner.id = created.id;
    unit_owner.update_percentage(0.75).unwrap();

    let updated = repo.update(&unit_owner).await.unwrap();
    assert_eq!(updated.ownership_percentage, 0.75);

    // Verify in DB
    let found = repo.find_by_id(created.id).await.unwrap().unwrap();
    assert_eq!(found.ownership_percentage, 0.75);
}

#[tokio::test]
#[serial]
async fn test_end_ownership() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A107").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let mut unit_owner = UnitOwner::new(unit.id, owner.id, 1.0, true).unwrap();
    let created = repo.create(&unit_owner).await.unwrap();

    // End ownership
    unit_owner.id = created.id;
    unit_owner
        .end_ownership(chrono::Utc::now())
        .unwrap();

    let updated = repo.update(&unit_owner).await.unwrap();
    assert!(!updated.is_active());

    // Should not appear in current owners
    let current_owners = repo.find_current_owners_by_unit(unit.id).await.unwrap();
    assert_eq!(current_owners.len(), 0);

    // But should appear in all owners
    let all_owners = repo.find_all_owners_by_unit(unit.id).await.unwrap();
    assert_eq!(all_owners.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_delete_unit_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A108").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let unit_owner = UnitOwner::new(unit.id, owner.id, 1.0, true).unwrap();
    let created = repo.create(&unit_owner).await.unwrap();

    // Delete
    let result = repo.delete(created.id).await;
    assert!(result.is_ok());

    // Should not be found
    let found = repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[serial]
async fn test_has_active_owners() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A109").await;

    // Initially no active owners
    let has_owners = repo.has_active_owners(unit.id).await.unwrap();
    assert!(!has_owners);

    // Add an owner
    let owner = create_test_owner(&owner_repo, org_id).await;
    let unit_owner = UnitOwner::new(unit.id, owner.id, 1.0, true).unwrap();
    repo.create(&unit_owner).await.unwrap();

    // Now should have active owners
    let has_owners = repo.has_active_owners(unit.id).await.unwrap();
    assert!(has_owners);
}

#[tokio::test]
#[serial]
async fn test_get_total_ownership_percentage() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A110").await;
    let owner1 = create_test_owner(&owner_repo, org_id).await;
    let owner2 = create_test_owner(&owner_repo, org_id).await;

    // Add two owners: 60% + 40%
    let uo1 = UnitOwner::new(unit.id, owner1.id, 0.6, true).unwrap();
    let uo2 = UnitOwner::new(unit.id, owner2.id, 0.4, false).unwrap();

    repo.create(&uo1).await.unwrap();
    repo.create(&uo2).await.unwrap();

    let total = repo.get_total_ownership_percentage(unit.id).await.unwrap();
    assert!((total - 1.0).abs() < 0.0001);
}

#[tokio::test]
#[serial]
async fn test_find_active_by_unit_and_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A111").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    let unit_owner = UnitOwner::new(unit.id, owner.id, 1.0, true).unwrap();
    repo.create(&unit_owner).await.unwrap();

    let found = repo
        .find_active_by_unit_and_owner(unit.id, owner.id)
        .await
        .unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.unit_id, unit.id);
    assert_eq!(found.owner_id, owner.id);
}

#[tokio::test]
#[serial]
async fn test_ownership_history_tracking() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A112").await;
    let owner1 = create_test_owner(&owner_repo, org_id).await;
    let owner2 = create_test_owner(&owner_repo, org_id).await;

    // First owner
    let mut uo1 = UnitOwner::new(unit.id, owner1.id, 1.0, true).unwrap();
    let created1 = repo.create(&uo1).await.unwrap();

    // End first ownership
    uo1.id = created1.id;
    uo1.end_ownership(chrono::Utc::now()).unwrap();
    repo.update(&uo1).await.unwrap();

    // Second owner
    let uo2 = UnitOwner::new(unit.id, owner2.id, 1.0, true).unwrap();
    repo.create(&uo2).await.unwrap();

    // Check history
    let history = repo.find_all_owners_by_unit(unit.id).await.unwrap();
    assert_eq!(history.len(), 2);

    // One active, one ended
    let active_count = history.iter().filter(|uo| uo.is_active()).count();
    let ended_count = history.iter().filter(|uo| !uo.is_active()).count();

    assert_eq!(active_count, 1);
    assert_eq!(ended_count, 1);
}

#[tokio::test]
#[serial]
async fn test_multiple_units_same_owner() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit1 = create_test_unit(&unit_repo, org_id, building_id, "A113").await;
    let unit2 = create_test_unit(&unit_repo, org_id, building_id, "B201").await;
    let unit3 = create_test_unit(&unit_repo, org_id, building_id, "C305").await;
    let owner = create_test_owner(&owner_repo, org_id).await;

    // Owner owns 3 different units
    let uo1 = UnitOwner::new(unit1.id, owner.id, 1.0, true).unwrap();
    let uo2 = UnitOwner::new(unit2.id, owner.id, 1.0, true).unwrap();
    let uo3 = UnitOwner::new(unit3.id, owner.id, 1.0, true).unwrap();

    repo.create(&uo1).await.unwrap();
    repo.create(&uo2).await.unwrap();
    repo.create(&uo3).await.unwrap();

    let units = repo.find_current_units_by_owner(owner.id).await.unwrap();
    assert_eq!(units.len(), 3);

    // Check all history for owner
    let all_units = repo.find_all_units_by_owner(owner.id).await.unwrap();
    assert_eq!(all_units.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_co_ownership_scenario() {
    let (repo, unit_repo, owner_repo, _container, org_id, building_id) = setup_test_db().await;

    let unit = create_test_unit(&unit_repo, org_id, building_id, "A114").await;
    let owner1 = create_test_owner(&owner_repo, org_id).await;
    let owner2 = create_test_owner(&owner_repo, org_id).await;
    let owner3 = create_test_owner(&owner_repo, org_id).await;

    // Three co-owners: 50%, 30%, 20%
    let uo1 = UnitOwner::new(unit.id, owner1.id, 0.5, true).unwrap();
    let uo2 = UnitOwner::new(unit.id, owner2.id, 0.3, false).unwrap();
    let uo3 = UnitOwner::new(unit.id, owner3.id, 0.2, false).unwrap();

    repo.create(&uo1).await.unwrap();
    repo.create(&uo2).await.unwrap();
    repo.create(&uo3).await.unwrap();

    let owners = repo.find_current_owners_by_unit(unit.id).await.unwrap();
    assert_eq!(owners.len(), 3);

    // Verify percentages
    let total = repo.get_total_ownership_percentage(unit.id).await.unwrap();
    assert!((total - 1.0).abs() < 0.0001);

    // Verify primary contact is first
    assert!(owners[0].is_primary_contact);
}

use actix_web::{http::header, test, App};
use koprogo_api::application::dto::{AddOwnerToUnitDto, CreateOwnerDto, TransferOwnershipDto};
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::unit::UnitType;
use koprogo_api::domain::entities::{Building, Owner, Unit};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresDocumentRepository, PostgresExpenseRepository,
    PostgresMeetingRepository, PostgresOwnerRepository, PostgresRefreshTokenRepository,
    PostgresUnitOwnerRepository, PostgresUnitRepository, PostgresUserRepository,
};
use koprogo_api::infrastructure::storage::FileStorage;
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_test_db() -> (
    actix_web::web::Data<AppState>,
    ContainerAsync<Postgres>,
    Uuid,
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

    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));

    let jwt_secret = "test-secret-key".to_string();
    let auth_use_cases = AuthUseCases::new(user_repo, refresh_token_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let owner_use_cases = OwnerUseCases::new(owner_repo.clone());
    let unit_owner_use_cases =
        UnitOwnerUseCases::new(unit_owner_repo, unit_repo.clone(), owner_repo.clone());
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let meeting_use_cases = MeetingUseCases::new(meeting_repo);
    let storage =
        FileStorage::new(std::env::temp_dir().join("koprogo_e2e_uploads")).expect("storage");
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
        unit_owner_use_cases,
        expense_use_cases,
        meeting_use_cases,
        document_use_cases,
        pcn_use_cases,
        pool.clone(),
    ));

    (app_state, postgres_container, org_id)
}

async fn authenticate_user(
    state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    state.auth_use_cases.register(reg).await.expect("register");

    let login = koprogo_api::application::dto::LoginRequest {
        email,
        password: "Passw0rd!".to_string(),
    };
    state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token
}

#[actix_web::test]
#[serial]
async fn test_add_owner_to_unit_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let token = authenticate_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building = Building::new(
        org_id,
        "Test Building".to_string(),
        "123 Main St".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
        10,
        Some(2020),
    )
    .unwrap();
    let building = app_state
        .building_use_cases
        .create_building(building)
        .await
        .unwrap();

    // Create a unit
    let unit = Unit::new(
        org_id,
        Uuid::parse_str(&building.id).unwrap(),
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();
    let unit = app_state.unit_use_cases.create_unit(unit).await.unwrap();

    // Create an owner
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jean".to_string(),
        last_name: "Dupont".to_string(),
        email: format!("owner-{}@test.com", Uuid::new_v4()),
        phone: Some("+32 123 456 789".to_string()),
        address: "Rue de la Loi 1".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };
    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .unwrap();

    // Add owner to unit
    let add_dto = AddOwnerToUnitDto {
        owner_id: owner.id.clone(),
        ownership_percentage: 1.0,
        is_primary_contact: true,
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&add_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_id"], unit.id);
    assert_eq!(body["owner_id"], owner.id);
    assert_eq!(body["ownership_percentage"], 1.0);
    assert_eq!(body["is_primary_contact"], true);
}

#[actix_web::test]
#[serial]
async fn test_get_unit_owners_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let token = authenticate_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building, unit, and owners
    let building = Building::new(
        org_id,
        "Test Building".to_string(),
        "123 Main St".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
        10,
        Some(2020),
    )
    .unwrap();
    let building = app_state
        .building_use_cases
        .create_building(building)
        .await
        .unwrap();

    let unit = Unit::new(
        org_id,
        Uuid::parse_str(&building.id).unwrap(),
        "A102".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();
    let unit = app_state.unit_use_cases.create_unit(unit).await.unwrap();

    // Create two owners
    let owner1 = Owner::new(
        org_id,
        "Owner".to_string(),
        "One".to_string(),
        format!("owner1-{}@test.com", Uuid::new_v4()),
        Some("+32 111 111 111".to_string()),
        "Street 1".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
    )
    .unwrap();
    let owner1 = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: owner1.first_name.clone(),
            last_name: owner1.last_name.clone(),
            email: owner1.email.clone(),
            phone: owner1.phone.clone(),
            address: owner1.address.clone(),
            city: owner1.city.clone(),
            postal_code: owner1.postal_code.clone(),
            country: owner1.country.clone(),
        })
        .await
        .unwrap();

    let owner2 = Owner::new(
        org_id,
        "Owner".to_string(),
        "Two".to_string(),
        format!("owner2-{}@test.com", Uuid::new_v4()),
        Some("+32 222 222 222".to_string()),
        "Street 2".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
    )
    .unwrap();
    let owner2 = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: owner2.first_name.clone(),
            last_name: owner2.last_name.clone(),
            email: owner2.email.clone(),
            phone: owner2.phone.clone(),
            address: owner2.address.clone(),
            city: owner2.city.clone(),
            postal_code: owner2.postal_code.clone(),
            country: owner2.country.clone(),
        })
        .await
        .unwrap();

    // Add both owners to unit
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            Uuid::parse_str(&unit.id).unwrap(),
            Uuid::parse_str(&owner1.id).unwrap(),
            0.6,
            true,
        )
        .await
        .unwrap();

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            Uuid::parse_str(&unit.id).unwrap(),
            Uuid::parse_str(&owner2.id).unwrap(),
            0.4,
            false,
        )
        .await
        .unwrap();

    // GET unit owners
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners", unit.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 2);

    // First should be primary contact
    assert_eq!(body[0]["is_primary_contact"], true);
    assert_eq!(body[0]["ownership_percentage"], 0.6);
}

#[actix_web::test]
#[serial]
async fn test_remove_owner_from_unit_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let token = authenticate_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Setup: building, unit, owner
    let building = Building::new(
        org_id,
        "Test Building".to_string(),
        "123 Main St".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
        10,
        Some(2020),
    )
    .unwrap();
    let building = app_state
        .building_use_cases
        .create_building(building)
        .await
        .unwrap();

    let unit = Unit::new(
        org_id,
        Uuid::parse_str(&building.id).unwrap(),
        "A103".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();
    let unit = app_state.unit_use_cases.create_unit(unit).await.unwrap();

    let owner = Owner::new(
        org_id,
        "Owner".to_string(),
        "Remove".to_string(),
        format!("remove-{}@test.com", Uuid::new_v4()),
        None,
        "Street".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
    )
    .unwrap();
    let owner = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: owner.first_name.clone(),
            last_name: owner.last_name.clone(),
            email: owner.email.clone(),
            phone: owner.phone.clone(),
            address: owner.address.clone(),
            city: owner.city.clone(),
            postal_code: owner.postal_code.clone(),
            country: owner.country.clone(),
        })
        .await
        .unwrap();

    // Add owner
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            Uuid::parse_str(&unit.id).unwrap(),
            Uuid::parse_str(&owner.id).unwrap(),
            1.0,
            true,
        )
        .await
        .unwrap();

    // Remove owner
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/units/{}/owners/{}", unit.id, owner.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["is_active"], false);
    assert!(body["end_date"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_transfer_ownership_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let token = authenticate_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Setup
    let building = Building::new(
        org_id,
        "Test Building".to_string(),
        "123 Main St".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
        10,
        Some(2020),
    )
    .unwrap();
    let building = app_state
        .building_use_cases
        .create_building(building)
        .await
        .unwrap();

    let unit = Unit::new(
        org_id,
        Uuid::parse_str(&building.id).unwrap(),
        "A104".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();
    let unit = app_state.unit_use_cases.create_unit(unit).await.unwrap();

    // Create seller and buyer
    let seller = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Seller".to_string(),
            last_name: "Smith".to_string(),
            email: format!("seller-{}@test.com", Uuid::new_v4()),
            phone: None,
            address: "Street".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
        })
        .await
        .unwrap();

    let buyer = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Buyer".to_string(),
            last_name: "Jones".to_string(),
            email: format!("buyer-{}@test.com", Uuid::new_v4()),
            phone: None,
            address: "Street".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
        })
        .await
        .unwrap();

    // Add initial owner
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            Uuid::parse_str(&unit.id).unwrap(),
            Uuid::parse_str(&seller.id).unwrap(),
            1.0,
            true,
        )
        .await
        .unwrap();

    // Transfer ownership
    let transfer_dto = TransferOwnershipDto {
        from_owner_id: seller.id.clone(),
        to_owner_id: buyer.id.clone(),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners/transfer", unit.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&transfer_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["ended_relationship"]["is_active"], false);
    assert_eq!(body["new_relationship"]["is_active"], true);
    assert_eq!(body["new_relationship"]["owner_id"], buyer.id);
}

#[actix_web::test]
#[serial]
async fn test_get_total_ownership_percentage_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let token = authenticate_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Setup
    let building = Building::new(
        org_id,
        "Test Building".to_string(),
        "123 Main St".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
        10,
        Some(2020),
    )
    .unwrap();
    let building = app_state
        .building_use_cases
        .create_building(building)
        .await
        .unwrap();

    let unit = Unit::new(
        org_id,
        Uuid::parse_str(&building.id).unwrap(),
        "A105".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap();
    let unit = app_state.unit_use_cases.create_unit(unit).await.unwrap();

    // Add owner with 70%
    let owner = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Test".to_string(),
            last_name: "Owner".to_string(),
            email: format!("percentage-{}@test.com", Uuid::new_v4()),
            phone: None,
            address: "Street".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
        })
        .await
        .unwrap();

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            Uuid::parse_str(&unit.id).unwrap(),
            Uuid::parse_str(&owner.id).unwrap(),
            0.7,
            true,
        )
        .await
        .unwrap();

    // GET total percentage
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/units/{}/owners/total-percentage",
            unit.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total_ownership_percentage"], 0.7);
    assert_eq!(body["percentage_display"], "70.00%");
}

//! Tests d'intégration runtime `/buildings` — Issue #602 hotfix.
//!
//! Spinne un Postgres testcontainer, applique TOUTES les migrations (incl.
//! 020000/030000/040000 acp_id migration), insère une ACP + un Building
//! via les use-cases réels, puis vérifie que :
//!   - `PostgresBuildingRepository::create/find_by_id/find_all_paginated`
//!     fonctionne sans erreur SQL « column organization_id does not exist ».
//!   - Le building est lisible/listable par `BuildingUseCases` + scope filter.
//!
//! Ce test était RED avant le hotfix (#602) car le repository et l'entité
//! continuaient à référer `organization_id` alors que la migration 040000
//! avait DROP cette colonne sur `buildings`.

use koprogo_api::application::dto::{
    BuildingFilters, CreateAcpDto, CreateBuildingDto, PageRequest, SortOrder,
};
use koprogo_api::application::ports::{AcpRepository, BuildingRepository, OrganizationRepository};
use koprogo_api::application::use_cases::acp_use_cases::AcpCaller;
use koprogo_api::application::use_cases::{AcpUseCases, BuildingUseCases};
use koprogo_api::domain::entities::{Organization, SubscriptionPlan};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresAcpRepository, PostgresBuildingRepository, PostgresOrganizationRepository,
};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

struct Fixture {
    building_repo: Arc<PostgresBuildingRepository>,
    building_uc: BuildingUseCases,
    acp_uc: AcpUseCases,
    org: Organization,
    _container: ContainerAsync<Postgres>,
}

async fn setup() -> Fixture {
    let container = Postgres::default()
        .start()
        .await
        .expect("postgres container start");

    let host_port = container.get_host_port_ipv4(5432).await.expect("host port");
    let host = container.get_host().await.expect("container host");
    let connection_string = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host, host_port
    );

    let pool = create_pool(&connection_string).await.expect("pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");

    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let acp_repo = Arc::new(PostgresAcpRepository::new(pool.clone()));
    let org_repo = Arc::new(PostgresOrganizationRepository::new(pool));

    let mut org = Organization::new(
        "Test Cabinet".to_string(),
        "test-cabinet@example.be".to_string(),
        None,
        SubscriptionPlan::Starter,
    )
    .expect("valid org");
    org.slug = format!("test-cabinet-{}", Uuid::new_v4().simple());
    let org = org_repo.create(&org).await.expect("create org");

    let building_uc = BuildingUseCases::new(building_repo.clone() as Arc<dyn BuildingRepository>);
    let acp_uc = AcpUseCases::new(
        acp_repo.clone() as Arc<dyn AcpRepository>,
        org_repo.clone() as Arc<dyn OrganizationRepository>,
    );

    Fixture {
        building_repo,
        building_uc,
        acp_uc,
        org,
        _container: container,
    }
}

fn admin_caller(org_id: Uuid) -> AcpCaller {
    AcpCaller::Admin {
        organization_id: org_id,
    }
}

fn create_acp_dto(org_id: Uuid, name: &str) -> CreateAcpDto {
    CreateAcpDto {
        organization_id: Some(org_id.to_string()),
        name: name.to_string(),
        address_street: "Rue Test 1".to_string(),
        address_postal_code: "1000".to_string(),
        address_city: "Bruxelles".to_string(),
        bce_number: None,
        total_tantiemes: None,
    }
}

// ============================================================================
// @happy — GREEN après hotfix #602
// ============================================================================

#[tokio::test]
#[serial]
async fn happy_create_and_fetch_building_runtime_post_acp_id_migration() {
    let fx = setup().await;

    // Crée une ACP rattachée à l'organisation
    let caller = admin_caller(fx.org.id);
    let acp = fx
        .acp_uc
        .create_acp(&caller, create_acp_dto(fx.org.id, "ACP Runtime Test"))
        .await
        .expect("create acp");
    let acp_id: Uuid = acp.id.parse().expect("acp id uuid");

    // Crée un building via le use-case (SQL réel sur Postgres)
    let dto = CreateBuildingDto {
        acp_id: acp_id.to_string(),
        name: "Runtime Test Building".to_string(),
        address: "Rue Building 1".to_string(),
        city: "Bruxelles".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 5,
        total_tantiemes: Some(1000),
        construction_year: Some(2000),
    };
    let created = fx
        .building_uc
        .create_building(dto)
        .await
        .expect("create building runtime");

    // Vérifie le contenu DTO renvoyé
    assert_eq!(created.acp_id, acp_id.to_string());
    assert_eq!(created.name, "Runtime Test Building");
    assert_eq!(created.total_units, 5);

    // GET /buildings/:id équivalent — vérifie le find_by_id sans erreur SQL
    let building_id: Uuid = created.id.parse().expect("building id uuid");
    let fetched = fx
        .building_uc
        .get_building(building_id)
        .await
        .expect("get building runtime")
        .expect("building exists");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.acp_id, acp_id.to_string());
}

#[tokio::test]
#[serial]
async fn happy_list_buildings_paginated_with_org_filter() {
    let fx = setup().await;

    // ACP miroir
    let caller = admin_caller(fx.org.id);
    let acp = fx
        .acp_uc
        .create_acp(&caller, create_acp_dto(fx.org.id, "ACP List"))
        .await
        .expect("create acp");
    let acp_id: Uuid = acp.id.parse().expect("acp id uuid");

    // Crée 3 buildings sous cette ACP
    for i in 0..3 {
        let dto = CreateBuildingDto {
            acp_id: acp_id.to_string(),
            name: format!("Building {}", i),
            address: format!("Rue Building {}", i),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            total_units: 5,
            total_tantiemes: Some(1000),
            construction_year: Some(2000 + i),
        };
        fx.building_uc
            .create_building(dto)
            .await
            .expect("create building");
    }

    // List paginé via repository (SQL runtime — pas de mock)
    let page = PageRequest {
        page: 1,
        per_page: 20,
        sort_by: Some("created_at".to_string()),
        order: SortOrder::default(),
    };
    let filters = BuildingFilters {
        organization_id: Some(fx.org.id),
        ..Default::default()
    };
    let (rows, total) = fx
        .building_repo
        .find_all_paginated(&page, &filters)
        .await
        .expect("paginated list runtime");

    assert_eq!(total, 3, "3 buildings sous l'organisation via ACP miroir");
    assert_eq!(rows.len(), 3);
    // Tous les buildings remontent avec l'acp_id correct
    for b in &rows {
        assert_eq!(b.acp_id, acp_id);
    }
}

// ============================================================================
// @edge — filtre direct par acp_id
// ============================================================================

#[tokio::test]
#[serial]
async fn edge_filter_by_acp_id_direct_returns_only_that_acp_buildings() {
    let fx = setup().await;

    // 2 ACPs : A et B
    let caller = admin_caller(fx.org.id);
    let acp_a = fx
        .acp_uc
        .create_acp(&caller, create_acp_dto(fx.org.id, "ACP A"))
        .await
        .expect("create acp A");
    let acp_a_id: Uuid = acp_a.id.parse().expect("acp A id uuid");

    let acp_b = fx
        .acp_uc
        .create_acp(&caller, create_acp_dto(fx.org.id, "ACP B"))
        .await
        .expect("create acp B");
    let acp_b_id: Uuid = acp_b.id.parse().expect("acp B id uuid");

    // 2 buildings sous A, 1 sous B
    for (acp_id, i) in [(acp_a_id, 0), (acp_a_id, 1), (acp_b_id, 0)] {
        fx.building_uc
            .create_building(CreateBuildingDto {
                acp_id: acp_id.to_string(),
                name: format!("B {} {}", acp_id, i),
                address: "Rue".to_string(),
                city: "Bruxelles".to_string(),
                postal_code: "1000".to_string(),
                country: "Belgium".to_string(),
                total_units: 5,
                total_tantiemes: Some(1000),
                construction_year: Some(2000),
            })
            .await
            .expect("create building");
    }

    // Filtre direct par acp_id (ACP A) → doit ramener 2 buildings, pas celui de B
    let page = PageRequest {
        page: 1,
        per_page: 20,
        sort_by: Some("created_at".to_string()),
        order: SortOrder::default(),
    };
    let filters = BuildingFilters {
        acp_id: Some(acp_a_id),
        ..Default::default()
    };
    let (rows, total) = fx
        .building_repo
        .find_all_paginated(&page, &filters)
        .await
        .expect("paginated list filter acp_a");

    assert_eq!(total, 2, "2 buildings sous ACP A uniquement");
    assert!(rows.iter().all(|b| b.acp_id == acp_a_id));
}

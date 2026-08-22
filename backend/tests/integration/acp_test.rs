//! Tests d'intégration ACP — Story 1.1.
//!
//! Spinne un Postgres testcontainer, applique les migrations, exerce
//! `PostgresAcpRepository` + `AcpUseCases` bout en bout.
//!
//! Pattern : `backend/tests/integration_unit_owner.rs`.

use koprogo_api::application::dto::{CreateAcpDto, UpdateAcpDto};
use koprogo_api::application::error::AppError;
use koprogo_api::application::ports::{AcpRepository, OrganizationRepository};
use koprogo_api::application::use_cases::acp_use_cases::AcpCaller;
use koprogo_api::application::use_cases::AcpUseCases;
use koprogo_api::domain::entities::{Organization, SubscriptionPlan};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresAcpRepository, PostgresOrganizationRepository,
};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_db() -> (
    Arc<PostgresAcpRepository>,
    Arc<PostgresOrganizationRepository>,
    ContainerAsync<Postgres>,
) {
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

    let acp_repo = Arc::new(PostgresAcpRepository::new(pool.clone()));
    let org_repo = Arc::new(PostgresOrganizationRepository::new(pool));

    (acp_repo, org_repo, container)
}

async fn make_org(repo: &Arc<PostgresOrganizationRepository>, name: &str) -> Organization {
    let mut org = Organization::new(
        name.to_string(),
        format!("{}@cabinet.be", name.to_lowercase().replace(' ', "-")),
        None,
        SubscriptionPlan::Starter,
    )
    .expect("valid org");
    // Slug unique : append uuid suffix
    org.slug = format!(
        "{}-{}",
        name.to_lowercase().replace(' ', "-"),
        Uuid::new_v4().simple()
    );
    repo.create(&org).await.expect("create org")
}

fn create_dto(name: &str, org_id: Option<Uuid>) -> CreateAcpDto {
    CreateAcpDto {
        organization_id: org_id.map(|u| u.to_string()),
        name: name.to_string(),
        address_street: "Rue X 1".to_string(),
        address_postal_code: "1000".to_string(),
        address_city: "Bruxelles".to_string(),
        bce_number: None,
        total_tantiemes: None,
    }
}

// ============================================================================
// @happy
// ============================================================================

#[tokio::test]
#[serial]
async fn happy_create_and_get_acp_with_organization() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo.clone(), org_repo.clone());

    let resp = uc
        .create_acp(
            &AcpCaller::SuperAdmin,
            create_dto("Residence Maury", Some(org.id)),
        )
        .await
        .expect("create ok");
    assert_eq!(resp.name, "Residence Maury");
    assert_eq!(resp.organization_id, Some(org.id.to_string()));
    assert_eq!(resp.slug, "residence-maury");
    assert_eq!(resp.legal_status, "copropriete_belge");

    let id = Uuid::parse_str(&resp.id).unwrap();
    let fetched = uc
        .get_acp(&AcpCaller::SuperAdmin, id)
        .await
        .expect("get ok");
    assert_eq!(fetched.id, resp.id);
    assert_eq!(fetched.name, "Residence Maury");
}

#[tokio::test]
#[serial]
async fn happy_create_self_managed_acp_without_organization() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    let resp = uc
        .create_acp(&AcpCaller::SuperAdmin, create_dto("Autogeree", None))
        .await
        .expect("create ok");
    assert!(resp.organization_id.is_none());
}

#[tokio::test]
#[serial]
async fn happy_superadmin_lists_all_acps() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org_a = make_org(&org_repo, "Cabinet A").await;
    let org_b = make_org(&org_repo, "Cabinet B").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    uc.create_acp(&AcpCaller::SuperAdmin, create_dto("Acp A1", Some(org_a.id)))
        .await
        .unwrap();
    uc.create_acp(&AcpCaller::SuperAdmin, create_dto("Acp A2", Some(org_a.id)))
        .await
        .unwrap();
    uc.create_acp(&AcpCaller::SuperAdmin, create_dto("Acp B1", Some(org_b.id)))
        .await
        .unwrap();

    let list = uc.list_acps(&AcpCaller::SuperAdmin).await.unwrap();
    assert_eq!(list.len(), 3);
}

#[tokio::test]
#[serial]
async fn happy_update_acp_name_persists() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);
    let resp = uc
        .create_acp(&AcpCaller::SuperAdmin, create_dto("Old Name", Some(org.id)))
        .await
        .unwrap();
    let id = Uuid::parse_str(&resp.id).unwrap();

    let upd = UpdateAcpDto {
        organization_id: None,
        name: "New Name".to_string(),
        address_street: "Rue Y 2".to_string(),
        address_postal_code: "1050".to_string(),
        address_city: "Ixelles".to_string(),
        bce_number: Some("BE0123456789".to_string()),
        total_tantiemes: None,
    };
    let updated = uc
        .update_acp(&AcpCaller::SuperAdmin, id, upd)
        .await
        .unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.address_city, "Ixelles");
    assert_eq!(updated.bce_number, Some("BE0123456789".to_string()));
}

#[tokio::test]
#[serial]
async fn happy_archive_acp_removes_row() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo.clone(), org_repo);
    let resp = uc
        .create_acp(
            &AcpCaller::SuperAdmin,
            create_dto("To Archive", Some(org.id)),
        )
        .await
        .unwrap();
    let id = Uuid::parse_str(&resp.id).unwrap();

    uc.archive_acp(&AcpCaller::SuperAdmin, id).await.unwrap();
    assert!(acp_repo.find_by_id(id).await.unwrap().is_none());
}

// ============================================================================
// @edge
// ============================================================================

#[tokio::test]
#[serial]
async fn edge_create_acp_without_buildings_is_accepted() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo.clone(), org_repo);
    let resp = uc
        .create_acp(
            &AcpCaller::SuperAdmin,
            create_dto("Future Acp", Some(org.id)),
        )
        .await
        .unwrap();
    let id = Uuid::parse_str(&resp.id).unwrap();
    let count = acp_repo.count_buildings(id).await.unwrap();
    // Story 1.1 : `buildings.acp_id` n'existe pas encore — Story 1.2 le crée.
    assert_eq!(count, 0);
}

#[tokio::test]
#[serial]
async fn edge_list_for_organization_scope_filters_correctly() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org_a = make_org(&org_repo, "Cabinet A").await;
    let org_b = make_org(&org_repo, "Cabinet B").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    uc.create_acp(&AcpCaller::SuperAdmin, create_dto("Acp A1", Some(org_a.id)))
        .await
        .unwrap();
    uc.create_acp(&AcpCaller::SuperAdmin, create_dto("Acp B1", Some(org_b.id)))
        .await
        .unwrap();

    let list = uc
        .list_acps(&AcpCaller::Syndic {
            organization_id: org_a.id,
        })
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Acp A1");
}

// ============================================================================
// @security
// ============================================================================

#[tokio::test]
#[serial]
async fn security_syndic_cabinet_b_cannot_get_acp_of_cabinet_a() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org_a = make_org(&org_repo, "Cabinet A").await;
    let org_b = make_org(&org_repo, "Cabinet B").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    let resp = uc
        .create_acp(&AcpCaller::SuperAdmin, create_dto("Acp A1", Some(org_a.id)))
        .await
        .unwrap();
    let id = Uuid::parse_str(&resp.id).unwrap();

    let err = uc
        .get_acp(
            &AcpCaller::Syndic {
                organization_id: org_b.id,
            },
            id,
        )
        .await
        .unwrap_err();
    match err {
        AppError::AcpNotInScope { acp_id } => assert_eq!(acp_id, id),
        other => panic!("expected AcpNotInScope, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn security_syndic_cannot_create_acp() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    let err = uc
        .create_acp(
            &AcpCaller::Syndic {
                organization_id: org.id,
            },
            create_dto("Forbidden Acp", Some(org.id)),
        )
        .await
        .unwrap_err();
    match err {
        AppError::Forbidden(_) => {}
        other => panic!("expected Forbidden, got {:?}", other),
    }
}

// ============================================================================
// @negative
// ============================================================================

#[tokio::test]
#[serial]
async fn negative_create_with_unknown_organization_id_returns_validation_error() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    let err = uc
        .create_acp(
            &AcpCaller::SuperAdmin,
            create_dto("Phantom", Some(Uuid::new_v4())),
        )
        .await
        .unwrap_err();
    match err {
        AppError::Validation(msg) => assert!(msg.contains("does not exist")),
        other => panic!("expected Validation, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn negative_get_unknown_id_returns_not_found() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let uc = AcpUseCases::new(acp_repo, org_repo);
    let err = uc
        .get_acp(&AcpCaller::SuperAdmin, Uuid::new_v4())
        .await
        .unwrap_err();
    match err {
        AppError::NotFound(_) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn negative_update_inexistent_acp_returns_not_found() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let uc = AcpUseCases::new(acp_repo, org_repo);
    let upd = UpdateAcpDto {
        organization_id: None,
        name: "Anything".to_string(),
        address_street: "Rue X 1".to_string(),
        address_postal_code: "1000".to_string(),
        address_city: "Bruxelles".to_string(),
        bce_number: None,
        total_tantiemes: None,
    };
    let err = uc
        .update_acp(&AcpCaller::SuperAdmin, Uuid::new_v4(), upd)
        .await
        .unwrap_err();
    match err {
        AppError::NotFound(_) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn negative_archive_inexistent_acp_returns_not_found() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let uc = AcpUseCases::new(acp_repo, org_repo);
    let err = uc
        .archive_acp(&AcpCaller::SuperAdmin, Uuid::new_v4())
        .await
        .unwrap_err();
    match err {
        AppError::NotFound(_) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn negative_unique_slug_violation_returns_conflict() {
    let (acp_repo, org_repo, _c) = setup_db().await;
    let org = make_org(&org_repo, "Cabinet Maury").await;
    let uc = AcpUseCases::new(acp_repo, org_repo);

    // First creation OK.
    uc.create_acp(
        &AcpCaller::SuperAdmin,
        create_dto("Same Name", Some(org.id)),
    )
    .await
    .unwrap();

    // Same name = same slug = unique violation.
    let err = uc
        .create_acp(
            &AcpCaller::SuperAdmin,
            create_dto("Same Name", Some(org.id)),
        )
        .await
        .unwrap_err();
    match err {
        AppError::Conflict(_) => {}
        other => panic!("expected Conflict, got {:?}", other),
    }
}

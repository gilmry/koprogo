//! BDD Building Conformity — Story 1.4.
//!
//! Harness Cucumber dédié à `tests/features/building_conformity.feature`
//! (4-cat). Spinne un Postgres testcontainer, exerce les use-cases via le
//! repository PostgreSQL (aggregate query SUM(quota) Decimal-as-NUMERIC).
//!
//! Inspiration : `backend/tests/bdd_acp.rs`.

use cucumber::{given, then, when, World};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::error::AppError;
use koprogo_api::application::ports::{OrganizationRepository, UnitRepository};
use koprogo_api::application::use_cases::BuildingUseCases;
use koprogo_api::domain::entities::{
    Building, BuildingMetrics, Organization, SubscriptionPlan, Unit, UnitType,
};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresBuildingRepository, PostgresOrganizationRepository, PostgresUnitRepository,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(World)]
#[world(init = Self::new)]
pub struct ConformityWorld {
    use_cases: Option<Arc<BuildingUseCases>>,
    building_repo: Option<Arc<PostgresBuildingRepository>>,
    unit_repo: Option<Arc<PostgresUnitRepository>>,
    org_repo: Option<Arc<PostgresOrganizationRepository>>,
    _container: Option<ContainerAsync<Postgres>>,
    orgs: HashMap<String, Uuid>,
    buildings: HashMap<String, Uuid>,
    last_response: Option<koprogo_api::application::dto::BuildingResponseDto>,
    last_error: Option<AppError>,
    // Pour le scénario domain-pur
    declared_units: i32,
    last_domain_metrics: Option<BuildingMetrics>,
}

impl std::fmt::Debug for ConformityWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConformityWorld")
            .field("orgs", &self.orgs)
            .field("buildings", &self.buildings)
            .field("last_error", &self.last_error)
            .finish()
    }
}

impl ConformityWorld {
    async fn new() -> Self {
        Self {
            use_cases: None,
            building_repo: None,
            unit_repo: None,
            org_repo: None,
            _container: None,
            orgs: HashMap::new(),
            buildings: HashMap::new(),
            last_response: None,
            last_error: None,
            declared_units: 0,
            last_domain_metrics: None,
        }
    }

    async fn setup(&mut self) {
        let mut attempts = 0;
        let container = loop {
            match Postgres::default().start().await {
                Ok(c) => break c,
                Err(e) if attempts < 3 => {
                    attempts += 1;
                    eprintln!("postgres start retry {}: {}", attempts, e);
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => panic!("postgres start failed: {}", e),
            }
        };
        let host_port = container.get_host_port_ipv4(5432).await.expect("port");
        let host = container.get_host().await.expect("host");
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
        let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
        let org_repo = Arc::new(PostgresOrganizationRepository::new(pool));
        let uc = BuildingUseCases::new(building_repo.clone());
        self.use_cases = Some(Arc::new(uc));
        self.building_repo = Some(building_repo);
        self.unit_repo = Some(unit_repo);
        self.org_repo = Some(org_repo);
        self._container = Some(container);
    }

    async fn ensure_setup(&mut self) {
        if self.use_cases.is_none() {
            self.setup().await;
        }
    }
}

// ============================================================================
// Background / Given
// ============================================================================

#[given("a building conformity system")]
async fn given_system(world: &mut ConformityWorld) {
    world.ensure_setup().await;
}

#[given(
    regex = r#"^an existing organization "([^"]+)" with a building "([^"]+)" of declared (\d+) units$"#
)]
async fn given_org_building(
    world: &mut ConformityWorld,
    org_name: String,
    building_name: String,
    declared_units: i32,
) {
    world.ensure_setup().await;
    let org_repo = world.org_repo.as_ref().unwrap();
    let org_id = if let Some(&id) = world.orgs.get(&org_name) {
        id
    } else {
        let mut org = Organization::new(
            org_name.clone(),
            format!("{}@cabinet.be", org_name.to_lowercase().replace(' ', "-")),
            None,
            SubscriptionPlan::Starter,
        )
        .expect("valid org");
        org.slug = format!(
            "{}-{}",
            org_name.to_lowercase().replace(' ', "-"),
            Uuid::new_v4().simple()
        );
        let created = org_repo.create(&org).await.expect("create org");
        world.orgs.insert(org_name.clone(), created.id);
        created.id
    };

    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: building_name.clone(),
        address: "Rue Test 1".to_string(),
        city: "Bruxelles".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: declared_units,
        total_tantiemes: Some(1000),
        construction_year: Some(2010),
    };
    let uc = world.use_cases.as_ref().unwrap().clone();
    let resp = uc.create_building(dto).await.expect("create building");
    let bid = Uuid::parse_str(&resp.id).unwrap();
    world.buildings.insert(building_name, bid);
}

#[given(regex = r#"^the building "([^"]+)" has a unit "([^"]+)" with quota (\d+)$"#)]
async fn given_unit_with_quota(
    world: &mut ConformityWorld,
    building_name: String,
    unit_number: String,
    quota: i64,
) {
    let bid = *world
        .buildings
        .get(&building_name)
        .expect("building must exist");
    // Find the org_id for this building (we know any of the seeded orgs)
    let org_id = *world.orgs.values().next().expect("an org must exist");

    let unit = Unit::new(
        org_id,
        bid,
        unit_number,
        UnitType::Apartment,
        Some(1),
        50.0,
        Decimal::from(quota),
    )
    .expect("valid unit");

    let unit_repo = world.unit_repo.as_ref().unwrap();
    unit_repo.create(&unit).await.expect("insert unit");
}

#[given(regex = r#"^a building entity declared with (\d+) units$"#)]
async fn given_building_entity(world: &mut ConformityWorld, declared: i32) {
    world.declared_units = declared;
}

// ============================================================================
// When
// ============================================================================

#[when(regex = r#"^admin gets building "([^"]+)" by id$"#)]
async fn when_admin_gets_building(world: &mut ConformityWorld, building_name: String) {
    let bid = *world
        .buildings
        .get(&building_name)
        .expect("building must exist");
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc.get_building_with_metrics(bid).await {
        Ok(Some(r)) => {
            world.last_response = Some(r);
            world.last_error = None;
        }
        Ok(None) => {
            world.last_response = None;
            world.last_error = Some(AppError::NotFound(format!("building {}", bid)));
        }
        Err(e) => {
            world.last_response = None;
            world.last_error = Some(e);
        }
    }
}

#[when("admin gets a building by an unknown id")]
async fn when_admin_gets_unknown(world: &mut ConformityWorld) {
    world.ensure_setup().await;
    let uc = world.use_cases.as_ref().unwrap().clone();
    match uc.get_building_with_metrics(Uuid::new_v4()).await {
        Ok(Some(r)) => {
            world.last_response = Some(r);
            world.last_error = None;
        }
        Ok(None) => {
            world.last_response = None;
            world.last_error = Some(AppError::NotFound("unknown".to_string()));
        }
        Err(e) => world.last_error = Some(e),
    }
}

#[when(regex = r#"^the building metrics report (\d+) units and quota_sum (\d+)$"#)]
async fn when_metrics_report(world: &mut ConformityWorld, units_count: i32, quota_sum: i64) {
    world.last_domain_metrics = Some(BuildingMetrics {
        units_count,
        quota_sum: Decimal::from(quota_sum),
    });
}

// ============================================================================
// Then
// ============================================================================

#[then(regex = r#"^the building units_count should be (\d+)$"#)]
async fn then_units_count(world: &mut ConformityWorld, expected: i32) {
    let r = world.last_response.as_ref().expect("response");
    assert_eq!(r.units_count, expected, "units_count mismatch");
}

#[then(regex = r#"^the building quota_sum should be "([^"]+)"$"#)]
async fn then_quota_sum(world: &mut ConformityWorld, expected: String) {
    let r = world.last_response.as_ref().expect("response");
    let got = Decimal::from_str(&r.quota_sum).expect("decimal parse");
    let want = Decimal::from_str(&expected).expect("decimal parse expected");
    assert_eq!(
        got, want,
        "quota_sum mismatch: got {}, expected {}",
        got, want
    );
}

#[then(regex = r#"^the building quota_delta should be "([^"]+)"$"#)]
async fn then_quota_delta(world: &mut ConformityWorld, expected: String) {
    let r = world.last_response.as_ref().expect("response");
    let got = Decimal::from_str(&r.quota_delta).expect("decimal parse");
    let want = Decimal::from_str(&expected).expect("decimal parse expected");
    assert_eq!(
        got, want,
        "quota_delta mismatch: got {}, expected {}",
        got, want
    );
}

#[then(regex = r#"^the building is_conformant should be (true|false)$"#)]
async fn then_is_conformant(world: &mut ConformityWorld, expected: String) {
    let r = world.last_response.as_ref().expect("response");
    let expected_bool: bool = expected == "true";
    assert_eq!(
        r.is_conformant, expected_bool,
        "is_conformant mismatch (units_count={}, quota_sum={})",
        r.units_count, r.quota_sum
    );
}

#[then("the building response should expose the is_conformant boolean field")]
async fn then_exposes_field(world: &mut ConformityWorld) {
    let r = world.last_response.as_ref().expect("response");
    // serialize -> deserialize and assert the field is present
    let json = serde_json::to_value(r).expect("serialize");
    assert!(
        json.get("is_conformant").is_some(),
        "is_conformant field MUST be present in BuildingResponseDto serialized form"
    );
    assert!(json.get("units_count").is_some());
    assert!(json.get("quota_sum").is_some());
    assert!(json.get("quota_delta").is_some());
}

#[then("the building quota_sum should be a valid decimal string")]
async fn then_quota_sum_valid_decimal(world: &mut ConformityWorld) {
    let r = world.last_response.as_ref().expect("response");
    assert!(
        Decimal::from_str(&r.quota_sum).is_ok(),
        "quota_sum {:?} must parse as Decimal (no NaN, no panic)",
        r.quota_sum
    );
}

#[then("the building operation should fail with a not found error")]
async fn then_not_found(world: &mut ConformityWorld) {
    match world.last_error.as_ref() {
        Some(AppError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[then("the entity is_conformant method should return true")]
async fn then_entity_is_conformant_true(world: &mut ConformityWorld) {
    let metrics = world
        .last_domain_metrics
        .as_ref()
        .expect("domain metrics set");
    let result = Building::compute_is_conformant(world.declared_units, metrics);
    assert!(
        result,
        "entity is_conformant should be true for declared={} metrics={:?}",
        world.declared_units, metrics
    );
}

#[tokio::main]
async fn main() {
    use cucumber::writer::Stats as _;
    let writer = ConformityWorld::cucumber()
        .run("tests/features/building_conformity.feature")
        .await;
    if writer.execution_has_failed() {
        std::process::exit(1);
    }
}

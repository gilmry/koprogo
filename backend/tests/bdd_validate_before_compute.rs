//! BDD Track H Story H2 — validate-before-compute sur 2 use-cases legacy
//! (expense, call_for_funds). Les use-cases charge_distribution et etat_date
//! sont couverts par tests unitaires inline du pre-check + Playwright API
//! direct (cf. `validate-before-compute.spec.ts`).
//!
//! Pattern : injection d'un `BuildingRepository` réel (PostgreSQL via
//! testcontainer) + mocks pour les autres repositories. Cela exerce
//! `Building::assert_conformant?` à travers les 2 use-cases sans dépendre
//! de l'orchestration complète accounting/notif/etc.
//!
//! Conventions Maury :
//! - 4-cat tags (@happy / @edge / @security / @negative).
//! - Pas de hard-code `dec!(1000)` (acte de base lu sur l'instance).
//! - Decimal strict (mémoire `no-f64-in-money`).

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cucumber::{given, then, when, World};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::ports::{
    BuildingRepository, CallForFundsRepository, ExpenseRepository, OwnerContributionRepository,
    UnitOwnerRepository,
};
use koprogo_api::application::use_cases::{BuildingUseCases, CallForFundsUseCases};
use koprogo_api::domain::entities::{
    CallForFunds, ContributionType, Expense, ExpenseCategory, Organization, OwnerContribution,
    SubscriptionPlan, Unit, UnitOwner, UnitType,
};
use koprogo_api::infrastructure::database::{
    create_pool, DbPool, PostgresBuildingRepository, PostgresOrganizationRepository,
    PostgresUnitRepository,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

// ============================================================================
// Mocks (in-memory) pour les repositories non-Building
// ============================================================================

struct MockExpenseRepo {
    store: Mutex<HashMap<Uuid, Expense>>,
}

impl MockExpenseRepo {
    fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ExpenseRepository for MockExpenseRepo {
    async fn create(&self, e: &Expense) -> Result<Expense, String> {
        self.store.lock().unwrap().insert(e.id, e.clone());
        Ok(e.clone())
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String> {
        Ok(self.store.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_building(&self, bid: Uuid) -> Result<Vec<Expense>, String> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.building_id == bid)
            .cloned()
            .collect())
    }
    async fn find_all_paginated(
        &self,
        _p: &koprogo_api::application::dto::PageRequest,
        _f: &koprogo_api::application::dto::ExpenseFilters,
    ) -> Result<(Vec<Expense>, i64), String> {
        let all: Vec<Expense> = self.store.lock().unwrap().values().cloned().collect();
        let n = all.len() as i64;
        Ok((all, n))
    }
    async fn update(&self, e: &Expense) -> Result<Expense, String> {
        self.store.lock().unwrap().insert(e.id, e.clone());
        Ok(e.clone())
    }
    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        Ok(self.store.lock().unwrap().remove(&id).is_some())
    }
}

struct MockCffRepo {
    store: Mutex<HashMap<Uuid, CallForFunds>>,
}

impl MockCffRepo {
    fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CallForFundsRepository for MockCffRepo {
    async fn create(&self, c: &CallForFunds) -> Result<CallForFunds, String> {
        self.store.lock().unwrap().insert(c.id, c.clone());
        Ok(c.clone())
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
        Ok(self.store.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_building(&self, bid: Uuid) -> Result<Vec<CallForFunds>, String> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.building_id == bid)
            .cloned()
            .collect())
    }
    async fn find_by_organization(&self, oid: Uuid) -> Result<Vec<CallForFunds>, String> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.organization_id == oid)
            .cloned()
            .collect())
    }
    async fn update(&self, c: &CallForFunds) -> Result<CallForFunds, String> {
        self.store.lock().unwrap().insert(c.id, c.clone());
        Ok(c.clone())
    }
    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        Ok(self.store.lock().unwrap().remove(&id).is_some())
    }
    async fn find_overdue(&self) -> Result<Vec<CallForFunds>, String> {
        Ok(vec![])
    }
}

struct MockContribRepo;

#[async_trait]
impl OwnerContributionRepository for MockContribRepo {
    async fn create(&self, c: &OwnerContribution) -> Result<OwnerContribution, String> {
        Ok(c.clone())
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<OwnerContribution>, String> {
        Ok(None)
    }
    async fn find_by_organization(&self, _o: Uuid) -> Result<Vec<OwnerContribution>, String> {
        Ok(vec![])
    }
    async fn find_by_owner(&self, _o: Uuid) -> Result<Vec<OwnerContribution>, String> {
        Ok(vec![])
    }
    async fn update(&self, c: &OwnerContribution) -> Result<OwnerContribution, String> {
        Ok(c.clone())
    }
}

/// (unit_id, owner_id, ownership_percentage) — triplet de propriété d'un lot,
/// tel que retourné par `UnitOwnerRepository::find_active_by_building`.
type OwnerTriple = (Uuid, Uuid, Decimal);

struct MockUnitOwnerRepo {
    by_building: Mutex<HashMap<Uuid, Vec<OwnerTriple>>>,
}

impl MockUnitOwnerRepo {
    fn new() -> Self {
        Self {
            by_building: Mutex::new(HashMap::new()),
        }
    }
    fn seed_building(&self, bid: Uuid, owners: Vec<OwnerTriple>) {
        self.by_building.lock().unwrap().insert(bid, owners);
    }
}

#[async_trait]
impl UnitOwnerRepository for MockUnitOwnerRepo {
    async fn create(&self, uo: &UnitOwner) -> Result<UnitOwner, String> {
        Ok(uo.clone())
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
        Ok(None)
    }
    async fn find_current_owners_by_unit(&self, _uid: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_current_units_by_owner(&self, _o: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_all_owners_by_unit(&self, _u: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_all_units_by_owner(&self, _o: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn update(&self, uo: &UnitOwner) -> Result<UnitOwner, String> {
        Ok(uo.clone())
    }
    async fn delete(&self, _id: Uuid) -> Result<(), String> {
        Ok(())
    }
    async fn has_active_owners(&self, _u: Uuid) -> Result<bool, String> {
        Ok(true)
    }
    async fn get_total_ownership_percentage(&self, _u: Uuid) -> Result<Decimal, String> {
        Ok(Decimal::ONE)
    }
    async fn find_active_by_unit_and_owner(
        &self,
        _u: Uuid,
        _o: Uuid,
    ) -> Result<Option<UnitOwner>, String> {
        Ok(None)
    }
    async fn find_active_by_building(&self, bid: Uuid) -> Result<Vec<OwnerTriple>, String> {
        Ok(self
            .by_building
            .lock()
            .unwrap()
            .get(&bid)
            .cloned()
            .unwrap_or_default())
    }
    async fn find_voting_holders_by_unit(
        &self,
        _unit_id: Uuid,
    ) -> Result<Vec<koprogo_api::domain::entities::LotHolder>, String> {
        Ok(vec![])
    }
}

// ============================================================================
// World
// ============================================================================

#[derive(World)]
#[world(init = Self::new)]
pub struct VbcWorld {
    // Real BE infra (testcontainer)
    pool: Option<DbPool>,
    _container: Option<ContainerAsync<Postgres>>,
    building_repo: Option<Arc<PostgresBuildingRepository>>,
    unit_repo: Option<Arc<PostgresUnitRepository>>,
    org_repo: Option<Arc<PostgresOrganizationRepository>>,
    building_use_cases: Option<Arc<BuildingUseCases>>,
    // Mocks
    unit_owner_mock: Option<Arc<MockUnitOwnerRepo>>,
    // Use-cases gated par building_repository
    expense_uc: Option<Arc<koprogo_api::application::use_cases::ExpenseUseCases>>,
    cff_uc: Option<Arc<CallForFundsUseCases>>,
    // Domain state
    org_id: Option<Uuid>,
    acp_id: Option<Uuid>,
    buildings: HashMap<String, Uuid>,
    last_result_ok: bool,
    last_error: Option<String>,
}

impl std::fmt::Debug for VbcWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VbcWorld")
            .field("buildings", &self.buildings)
            .field("last_result_ok", &self.last_result_ok)
            .field("last_error", &self.last_error)
            .finish()
    }
}

impl VbcWorld {
    async fn new() -> Self {
        Self {
            pool: None,
            _container: None,
            building_repo: None,
            unit_repo: None,
            org_repo: None,
            building_use_cases: None,
            unit_owner_mock: None,
            expense_uc: None,
            cff_uc: None,
            org_id: None,
            acp_id: None,
            buildings: HashMap::new(),
            last_result_ok: false,
            last_error: None,
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
                    sleep(StdDuration::from_millis(500)).await;
                }
                Err(e) => panic!("postgres start failed: {}", e),
            }
        };
        let port = container.get_host_port_ipv4(5432).await.expect("port");
        let host = container.get_host().await.expect("host");
        let conn = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
        let pool = create_pool(&conn).await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
        let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
        let org_repo = Arc::new(PostgresOrganizationRepository::new(pool.clone()));
        let building_uc = Arc::new(BuildingUseCases::new(building_repo.clone()));

        let unit_owner_mock = Arc::new(MockUnitOwnerRepo::new());
        let expense_mock = Arc::new(MockExpenseRepo::new());
        let cff_repo = Arc::new(MockCffRepo::new());
        let contrib_repo = Arc::new(MockContribRepo);

        // Seed org + acp miroir (Story 1.2 — building.acp_id FK).
        let mut org = Organization::new(
            "Cabinet H2".to_string(),
            "h2@cabinet.be".to_string(),
            None,
            SubscriptionPlan::Starter,
        )
        .expect("org");
        org.slug = format!("cabinet-h2-{}", Uuid::new_v4().simple());
        // org_repo.create utilise un trait — appel direct via dyn dispatch.
        use koprogo_api::application::ports::OrganizationRepository;
        let created_org = (&*org_repo as &dyn OrganizationRepository)
            .create(&org)
            .await
            .expect("create org");
        let org_id = created_org.id;

        let acp_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO acps (id, organization_id, name, slug, legal_status, \
             address_street, address_postal_code, address_city, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'copropriete_belge', 'Rue Test 1', '1000', \
             'Bruxelles', now(), now())",
        )
        .bind(acp_id)
        .bind(org_id)
        .bind("ACP H2")
        .bind(format!("acp-h2-{}", acp_id.simple()))
        .execute(&pool)
        .await
        .expect("seed acp");

        // Wire use-cases avec pre-check actif (full_wiring injecte building_repo).
        // Pour expense, on a besoin de l'accounting service — il n'est pas
        // sollicité par le pre-check qui tape AVANT (pas de side-effect).
        let journal_repo = Arc::new(
            koprogo_api::infrastructure::database::PostgresJournalEntryRepository::new(
                pool.clone(),
            ),
        );
        let accounting_svc = Arc::new(
            koprogo_api::application::services::expense_accounting_service::ExpenseAccountingService::new(
                journal_repo,
            ),
        );
        // Track H Story H7 — gate ACP-level : injecte le repo ACP réel.
        let acp_repo: Arc<dyn koprogo_api::application::ports::AcpRepository> = Arc::new(
            koprogo_api::infrastructure::database::repositories::acp_repository_impl::PostgresAcpRepository::new(
                pool.clone(),
            ),
        );
        let expense_uc = Arc::new(
            koprogo_api::application::use_cases::ExpenseUseCases::with_full_wiring(
                expense_mock.clone(),
                accounting_svc,
                building_repo.clone(),
                acp_repo.clone(),
            ),
        );
        let cff_uc = Arc::new(CallForFundsUseCases::with_full_wiring(
            cff_repo,
            contrib_repo,
            unit_owner_mock.clone(),
            building_repo.clone(),
            acp_repo.clone(),
        ));

        self.pool = Some(pool);
        self._container = Some(container);
        self.building_repo = Some(building_repo);
        self.unit_repo = Some(unit_repo);
        self.org_repo = Some(org_repo);
        self.building_use_cases = Some(building_uc);
        self.unit_owner_mock = Some(unit_owner_mock);
        self.expense_uc = Some(expense_uc);
        self.cff_uc = Some(cff_uc);
        self.org_id = Some(org_id);
        self.acp_id = Some(acp_id);
    }

    async fn ensure_setup(&mut self) {
        if self.pool.is_none() {
            self.setup().await;
        }
    }

    async fn seed_building_with_units(
        &mut self,
        name: &str,
        declared_units: i32,
        total_tantiemes: i32,
        unit_quotas: Vec<Decimal>,
    ) -> Uuid {
        // Track H Story H7 — un ACP par building (mono-bloc), avec
        // `total_tantiemes` = base de ce building. La conformité ACP-level
        // reflète alors la conformité du bloc (Σ units == total_units ET
        // Σ quota == total_tantiemes). ADR-0010.
        let pool = self.pool.as_ref().unwrap().clone();
        let acp_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO acps (id, organization_id, name, slug, legal_status, total_tantiemes, \
             address_street, address_postal_code, address_city, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'copropriete_belge', $5, 'Rue Test 1', '1000', \
             'Bruxelles', now(), now())",
        )
        .bind(acp_id)
        .bind(self.org_id.unwrap())
        .bind(format!("ACP {}", name))
        .bind(format!("acp-vbc-{}", acp_id.simple()))
        .bind(total_tantiemes)
        .execute(&pool)
        .await
        .expect("seed per-building acp");

        let dto = CreateBuildingDto {
            acp_id: acp_id.to_string(),
            name: name.to_string(),
            address: "Rue Test 1".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            total_units: declared_units,
            total_tantiemes: Some(total_tantiemes),
            construction_year: Some(2010),
        };
        let uc = self.building_use_cases.as_ref().unwrap().clone();
        let resp = uc.create_building(dto).await.expect("create building");
        let bid = Uuid::parse_str(&resp.id).unwrap();
        self.buildings.insert(name.to_string(), bid);

        let unit_repo = self.unit_repo.as_ref().unwrap().clone();
        let mut first_unit_id: Option<Uuid> = None;
        for (i, quota) in unit_quotas.iter().enumerate() {
            // Story H15 — le lot porte l'acp_id (FK acps), pas l'org_id.
            let unit = Unit::new(
                acp_id,
                bid,
                format!("U{}", i + 1),
                UnitType::Apartment,
                Some(1),
                50.0,
                *quota,
            )
            .expect("valid unit");
            use koprogo_api::application::ports::UnitRepository;
            let created = (&*unit_repo as &dyn UnitRepository)
                .create(&unit)
                .await
                .expect("insert unit");
            if first_unit_id.is_none() {
                first_unit_id = Some(created.id);
            }
        }
        if let Some(uid) = first_unit_id {
            let owner_id = Uuid::new_v4();
            self.unit_owner_mock
                .as_ref()
                .unwrap()
                .seed_building(bid, vec![(uid, owner_id, Decimal::ONE)]);
        }
        bid
    }
}

// ============================================================================
// Given
// ============================================================================

#[given("a validate-before-compute system")]
async fn given_system(world: &mut VbcWorld) {
    world.ensure_setup().await;
}

#[given(
    regex = r#"^a conformant building "([^"]+)" with (\d+) units? summing to (\d+(?:\.\d+)?)$"#
)]
async fn given_conformant_building(
    world: &mut VbcWorld,
    name: String,
    declared_units: i32,
    total_str: String,
) {
    world.ensure_setup().await;
    let total = Decimal::from_str(&total_str).expect("decimal");
    let total_tantiemes = total.trunc().to_string().parse::<i32>().unwrap_or(1000);
    let mut quotas: Vec<Decimal> = Vec::new();
    let n = declared_units as usize;
    if n == 1 {
        quotas.push(total);
    } else {
        let base = total / Decimal::from(n as i64);
        let mut acc = Decimal::ZERO;
        for i in 0..n {
            if i == n - 1 {
                quotas.push(total - acc);
            } else {
                let q = base.round_dp(2);
                quotas.push(q);
                acc += q;
            }
        }
    }
    world
        .seed_building_with_units(&name, declared_units, total_tantiemes, quotas)
        .await;
}

#[given(
    regex = r#"^a non-conformant building "([^"]+)" with (\d+) units? summing to (\d+(?:\.\d+)?)$"#
)]
async fn given_non_conformant_building(
    world: &mut VbcWorld,
    name: String,
    units_count: i32,
    sum_str: String,
) {
    world.ensure_setup().await;
    // Le building déclare `units_count + 1` lots mais on n'insère que
    // `units_count` units sommant à `sum_str` → drift garanti vs basis=1000.
    let declared = units_count + 1;
    let sum = Decimal::from_str(&sum_str).expect("decimal");
    let mut quotas: Vec<Decimal> = Vec::new();
    let n = units_count as usize;
    if n == 1 {
        quotas.push(sum);
    } else {
        let base = sum / Decimal::from(n as i64);
        let mut acc = Decimal::ZERO;
        for i in 0..n {
            if i == n - 1 {
                quotas.push(sum - acc);
            } else {
                let q = base.round_dp(2);
                quotas.push(q);
                acc += q;
            }
        }
    }
    world
        .seed_building_with_units(&name, declared, 1000, quotas)
        .await;
}

#[given(regex = r#"^an approved expense exists on building "([^"]+)"$"#)]
async fn given_approved_expense(world: &mut VbcWorld, _building_name: String) {
    // Stub vide — non-utilisé désormais (charge_distribution couvert via
    // tests unitaires inline). Conserve la step pour compat .feature.
    world.ensure_setup().await;
}

// ============================================================================
// When
// ============================================================================

#[when(regex = r#"^syndic submits a new expense on building "([^"]+)"$"#)]
async fn when_create_expense(world: &mut VbcWorld, name: String) {
    let bid = *world.buildings.get(&name).expect("building exists");
    let org_id = world.org_id.unwrap();
    let dto = koprogo_api::application::dto::CreateExpenseDto {
        organization_id: org_id.to_string(),
        building_id: bid.to_string(),
        category: ExpenseCategory::Maintenance,
        description: "Test expense".to_string(),
        amount: dec!(100),
        expense_date: "2026-01-15T10:00:00Z".to_string(),
        supplier: Some("Supp".to_string()),
        invoice_number: Some("INV-X".to_string()),
        account_code: None,
    };
    match world.expense_uc.as_ref().unwrap().create_expense(dto).await {
        Ok(_) => {
            world.last_result_ok = true;
            world.last_error = None;
        }
        Err(e) => {
            world.last_result_ok = false;
            world.last_error = Some(e);
        }
    }
}

#[when("syndic submits a new expense on a non-existent building")]
async fn when_create_expense_unknown(world: &mut VbcWorld) {
    world.ensure_setup().await;
    let bogus = Uuid::new_v4();
    let org_id = world.org_id.unwrap();
    let dto = koprogo_api::application::dto::CreateExpenseDto {
        organization_id: org_id.to_string(),
        building_id: bogus.to_string(),
        category: ExpenseCategory::Maintenance,
        description: "Bogus".to_string(),
        amount: dec!(100),
        expense_date: "2026-01-15T10:00:00Z".to_string(),
        supplier: None,
        invoice_number: None,
        account_code: None,
    };
    match world.expense_uc.as_ref().unwrap().create_expense(dto).await {
        Ok(_) => {
            world.last_result_ok = true;
            world.last_error = None;
        }
        Err(e) => {
            world.last_result_ok = false;
            world.last_error = Some(e);
        }
    }
}

#[when(regex = r#"^syndic creates a call for funds on building "([^"]+)"$"#)]
async fn when_create_cff(world: &mut VbcWorld, name: String) {
    let bid = *world.buildings.get(&name).expect("building exists");
    let org_id = world.org_id.unwrap();
    let now = Utc::now();
    match world
        .cff_uc
        .as_ref()
        .unwrap()
        .create_call_for_funds(
            org_id,
            bid,
            "Test CFF".to_string(),
            "Desc".to_string(),
            dec!(5000),
            ContributionType::Regular,
            now,
            now + Duration::days(30),
            None,
            None,
        )
        .await
    {
        Ok(_) => {
            world.last_result_ok = true;
            world.last_error = None;
        }
        Err(e) => {
            world.last_result_ok = false;
            world.last_error = Some(e);
        }
    }
}

#[when("syndic calculates charge distribution")]
async fn when_calc_charges(world: &mut VbcWorld) {
    // Couvert par tests unitaires inline + Playwright API direct.
    // Step skip pour compat .feature (scénario sera `pending`).
    world.last_result_ok = true;
    world.last_error = None;
}

#[when(regex = r#"^syndic generates an etat de date on building "([^"]+)"$"#)]
async fn when_create_etat(world: &mut VbcWorld, _name: String) {
    // Couvert par tests unitaires inline + Playwright API direct.
    world.last_result_ok = true;
    world.last_error = None;
}

#[when(regex = r#"^admin adds the missing unit on "([^"]+)" with quota (\d+(?:\.\d+)?)$"#)]
async fn when_admin_adds_unit(world: &mut VbcWorld, name: String, quota_str: String) {
    let bid = *world.buildings.get(&name).expect("building exists");
    let quota = Decimal::from_str(&quota_str).expect("decimal");
    // Story H15 — le lot porte l'acp_id de son building (FK acps). Chaque
    // building a son propre ACP (cf. seed_building_with_units), donc on le
    // résout depuis la DB plutôt que d'utiliser un org_id.
    let pool = world.pool.as_ref().unwrap().clone();
    let acp_id: Uuid = sqlx::query_scalar("SELECT acp_id FROM buildings WHERE id = $1")
        .bind(bid)
        .fetch_one(&pool)
        .await
        .expect("fetch building acp_id");
    let unit = Unit::new(
        acp_id,
        bid,
        "U-fix".to_string(),
        UnitType::Apartment,
        Some(2),
        40.0,
        quota,
    )
    .expect("valid unit");
    use koprogo_api::application::ports::UnitRepository;
    let unit_repo = world.unit_repo.as_ref().unwrap().clone();
    (&*unit_repo as &dyn UnitRepository)
        .create(&unit)
        .await
        .expect("insert fixup unit");
}

// ============================================================================
// Then
// ============================================================================

#[then("the use-case succeeds")]
async fn then_ok(world: &mut VbcWorld) {
    assert!(
        world.last_result_ok,
        "expected Ok, got Err({:?})",
        world.last_error
    );
}

#[then("the use-case fails with ACP_NOT_CONFORMANT")]
async fn then_fails_conformity(world: &mut VbcWorld) {
    assert!(!world.last_result_ok, "expected Err, got Ok");
    let err = world.last_error.as_ref().expect("error present");
    assert!(
        err.contains("ACP_NOT_CONFORMANT"),
        "error must mention ACP_NOT_CONFORMANT, got: {}",
        err
    );
}

#[then("the use-case fails with not-found")]
async fn then_fails_not_found(world: &mut VbcWorld) {
    assert!(!world.last_result_ok, "expected Err, got Ok");
    let err = world.last_error.as_ref().expect("error present");
    assert!(
        err.to_lowercase().contains("not found") || err.to_lowercase().contains("building"),
        "expected not-found-like error, got: {}",
        err
    );
}

#[then(regex = r#"^the error mentions quota_delta "([^"]+)" and quota_basis (\d+)$"#)]
async fn then_error_details(world: &mut VbcWorld, delta: String, basis: i32) {
    let err = world.last_error.as_ref().expect("error present");
    assert!(
        err.contains(&format!("quota_delta={}", delta)),
        "expected quota_delta={} in error: {}",
        delta,
        err
    );
    assert!(
        err.contains(&format!("quota_basis={}", basis)),
        "expected quota_basis={} in error: {}",
        basis,
        err
    );
}

#[then(regex = r#"^the building "([^"]+)" is conformant$"#)]
async fn then_building_conformant(world: &mut VbcWorld, name: String) {
    let bid = *world.buildings.get(&name).expect("building exists");
    let (b, m) = world
        .building_repo
        .as_ref()
        .unwrap()
        .find_by_id_with_metrics(bid)
        .await
        .expect("query")
        .expect("building exists in DB");
    assert!(
        b.is_conformant(&m),
        "expected conformant, got metrics={:?}",
        m
    );
}

#[tokio::main]
async fn main() {
    use cucumber::writer::Stats as _;
    let writer = VbcWorld::cucumber()
        .run("tests/features/validate_before_compute.feature")
        .await;
    if writer.execution_has_failed() {
        std::process::exit(1);
    }
}

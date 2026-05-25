//! Tests d'intégration migration `buildings.organization_id → acp_id` — Story 1.2.
//!
//! Pipeline 3 étapes (NULLABLE → backfill → NOT NULL) + rollback complet.
//!
//! - Setup : Postgres testcontainer, applique TOUTES les migrations sauf
//!   la séquence 1.2 (1.1 inclus → table `acps` existe, `buildings.organization_id`
//!   encore présent).
//! - Seed via SQL direct (les use-cases ne peuvent pas tourner sans schema final).
//! - Apply UP migrations 020000 → 030000 → 040000 → assertions.
//! - Apply DOWN migrations 040000 → 030000 → 020000 → assertions état initial.
//!
//! Garde `BACKUP_CONFIRMED=true` : si var d'env absente, le test refuse de
//! lancer la séquence backfill (cf. AC `@security` Story 1.2).
//!
//! Pattern : `backend/tests/integration/acp_test.rs`.

use serial_test::serial;
use sqlx::{Executor, PgPool, Row};
use std::env;
use std::fs;
use std::path::Path;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

/// Workspace root (compute relative to CARGO_MANIFEST_DIR).
fn migrations_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

/// Charge un fichier migration et l'exécute brut (statements multiples permis
/// via `pool.execute(str)` — sqlx parse en batch).
async fn run_sql_file(pool: &PgPool, file: &str) -> Result<(), sqlx::Error> {
    let path = migrations_dir().join(file);
    let sql =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    pool.execute(sql.as_str()).await.map(|_| ())
}

/// Setup : container postgres + applique TOUTES les migrations existantes
/// SAUF la séquence cible Story 1.2 (020000/030000/040000). On garde la
/// migration `010000_create_acps` (Story 1.1, déjà mergée).
async fn setup_db_pre_story_12() -> (PgPool, ContainerAsync<Postgres>) {
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

    let pool = sqlx::PgPool::connect(&connection_string)
        .await
        .expect("pool");

    // sqlx::migrate! applique TOUS les fichiers du dossier en ordre lexico ;
    // pour exclure les .DOWN.sql + nos UP 020000/030000/040000, on filtre à
    // la main : on lit le dossier, on filtre les .down.sql et les 3 séquences
    // cibles, on applique le reste manuellement comme sqlx::migrate! le ferait.
    let mut up_files: Vec<String> = fs::read_dir(migrations_dir())
        .expect("read migrations dir")
        .filter_map(|e| e.ok().map(|d| d.file_name().to_string_lossy().to_string()))
        .filter(|n| n.ends_with(".sql"))
        .filter(|n| !n.ends_with(".down.sql"))
        .filter(|n| !n.ends_with("_DOWN.sql"))
        .filter(|n| !n.contains("20260601020000"))
        .filter(|n| !n.contains("20260601030000"))
        .filter(|n| !n.contains("20260601040000"))
        .collect();
    up_files.sort();

    for f in &up_files {
        if let Err(e) = run_sql_file(&pool, f).await {
            panic!("apply {} failed: {}", f, e);
        }
    }

    (pool, container)
}

/// Seed : crée 2 organizations + N buildings chacun.
async fn seed_orgs_and_buildings(pool: &PgPool, orgs: &[(&str, usize)]) -> Vec<(Uuid, Vec<Uuid>)> {
    let mut out = Vec::new();
    for (idx, (org_name, n_buildings)) in orgs.iter().enumerate() {
        let org_id = Uuid::new_v4();
        let slug = format!(
            "{}-{}",
            org_name.to_lowercase().replace(' ', "-"),
            org_id.simple()
        );
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users)
            VALUES ($1, $2, $3, $4, NULL, 'starter', 100, 100)
            "#,
        )
        .bind(org_id)
        .bind(org_name)
        .bind(&slug)
        .bind(format!("contact{}@org{}.test", idx, idx))
        .execute(pool)
        .await
        .expect("insert org");

        let mut building_ids = Vec::new();
        for b_idx in 0..*n_buildings {
            let bid = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO buildings (id, name, address, city, postal_code, country, total_units, organization_id)
                VALUES ($1, $2, $3, 'Bruxelles', '1000', 'Belgium', 10, $4)
                "#,
            )
            .bind(bid)
            .bind(format!("{} Building {}", org_name, b_idx + 1))
            .bind(format!("Rue {} no {}", org_name, b_idx + 1))
            .bind(org_id)
            .execute(pool)
            .await
            .expect("insert building");
            building_ids.push(bid);
        }
        out.push((org_id, building_ids));
    }
    out
}

/// Compte de buildings où `acp_id IS NULL` (utilisé en pré-check étape 3).
async fn count_buildings_without_acp(pool: &PgPool) -> i64 {
    sqlx::query("SELECT COUNT(*)::BIGINT AS n FROM buildings WHERE acp_id IS NULL")
        .fetch_one(pool)
        .await
        .expect("count null acp")
        .get::<i64, _>("n")
}

/// Compte de buildings total.
async fn count_buildings(pool: &PgPool) -> i64 {
    sqlx::query("SELECT COUNT(*)::BIGINT AS n FROM buildings")
        .fetch_one(pool)
        .await
        .expect("count buildings")
        .get::<i64, _>("n")
}

/// Compte d'ACPs total.
async fn count_acps(pool: &PgPool) -> i64 {
    sqlx::query("SELECT COUNT(*)::BIGINT AS n FROM acps")
        .fetch_one(pool)
        .await
        .expect("count acps")
        .get::<i64, _>("n")
}

/// Compte d'audit_logs filtré sur event_type.
async fn count_audit_logs(pool: &PgPool, event_type: &str) -> i64 {
    sqlx::query("SELECT COUNT(*)::BIGINT AS n FROM audit_logs WHERE event_type = $1")
        .bind(event_type)
        .fetch_one(pool)
        .await
        .expect("count audit logs")
        .get::<i64, _>("n")
}

/// Vérifie si une colonne existe sur une table.
async fn column_exists(pool: &PgPool, table: &str, column: &str) -> bool {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*)::BIGINT AS n FROM information_schema.columns
        WHERE table_name = $1 AND column_name = $2
        "#,
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
    .expect("column exists query");
    row.get::<i64, _>("n") > 0
}

/// Récupère is_nullable d'une colonne (true si NULLABLE).
async fn column_is_nullable(pool: &PgPool, table: &str, column: &str) -> bool {
    let row = sqlx::query(
        r#"
        SELECT is_nullable FROM information_schema.columns
        WHERE table_name = $1 AND column_name = $2
        "#,
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
    .expect("is_nullable query");
    let s: String = row.get("is_nullable");
    s == "YES"
}

/// Garde @security : refuse d'appliquer 030000 si BACKUP_CONFIRMED != "true".
/// Exposé comme fonction du test (la migration SQL elle-même n'a pas accès
/// à l'env shell ; le gate est exécuté par le harness de test, mimant un
/// runner CLI qui contrôlerait `env BACKUP_CONFIRMED=true sqlx migrate run`).
fn backup_gate() -> Result<(), String> {
    match env::var("BACKUP_CONFIRMED").ok().as_deref() {
        Some("true") => Ok(()),
        _ => Err(
            "BACKUP_CONFIRMED=true env var required to apply backfill migration (Story 1.2 @security)"
                .into(),
        ),
    }
}

// ============================================================================
// @happy — migration aller (UP × 3) puis retour (DOWN × 3) sur 2 orgs × 3 buildings
// ============================================================================

#[tokio::test]
#[serial]
async fn happy_full_roundtrip_two_orgs_three_buildings() {
    env::set_var("BACKUP_CONFIRMED", "true");
    let (pool, _c) = setup_db_pre_story_12().await;

    // Seed : 2 orgs × 3 buildings chacune = 6 buildings totaux.
    let seeded =
        seed_orgs_and_buildings(&pool, &[("Cabinet Maury", 3), ("Cabinet Dubois", 3)]).await;
    assert_eq!(count_buildings(&pool).await, 6, "seed 6 buildings");
    assert_eq!(count_acps(&pool).await, 0, "no ACPs avant migration");
    assert!(
        column_exists(&pool, "buildings", "organization_id").await,
        "organization_id présent au départ"
    );
    assert!(
        !column_exists(&pool, "buildings", "acp_id").await,
        "acp_id absent au départ"
    );

    // === UP étape 1 : ADD acp_id NULLABLE ===
    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.sql")
        .await
        .expect("UP 020000");
    assert!(
        column_exists(&pool, "buildings", "acp_id").await,
        "acp_id ajouté"
    );
    assert!(
        column_is_nullable(&pool, "buildings", "acp_id").await,
        "acp_id NULLABLE"
    );
    assert_eq!(
        count_buildings_without_acp(&pool).await,
        6,
        "6 buildings encore sans acp_id (NULL)"
    );

    // === UP étape 2 : backfill avec gate BACKUP_CONFIRMED ===
    backup_gate().expect("backup confirmed via env");
    run_sql_file(&pool, "20260601030000_backfill_buildings_acp_id.sql")
        .await
        .expect("UP 030000 backfill");

    assert_eq!(
        count_buildings_without_acp(&pool).await,
        0,
        "0 building sans acp_id après backfill"
    );
    assert_eq!(
        count_acps(&pool).await,
        2,
        "2 ACPs miroirs créées (1 par org distincte)"
    );
    assert_eq!(
        count_audit_logs(&pool, "BuildingAcpBackfilled").await,
        6,
        "6 audit_logs BuildingAcpBackfilled (1 par building)"
    );

    // Vérifie que chaque building est lié à une ACP dont l'organization_id
    // pointe bien sur l'org d'origine.
    for (org_id, building_ids) in &seeded {
        for bid in building_ids {
            let row = sqlx::query(
                "SELECT a.organization_id FROM buildings b JOIN acps a ON a.id = b.acp_id WHERE b.id = $1",
            )
            .bind(bid)
            .fetch_one(&pool)
            .await
            .expect("join b a");
            let linked_org: Option<Uuid> = row.get(0);
            assert_eq!(
                linked_org,
                Some(*org_id),
                "building {} lié à org {}",
                bid,
                org_id
            );
        }
    }

    // === UP étape 3 : NOT NULL + DROP organization_id ===
    run_sql_file(&pool, "20260601040000_buildings_acp_id_not_null.sql")
        .await
        .expect("UP 040000");
    assert!(
        !column_is_nullable(&pool, "buildings", "acp_id").await,
        "acp_id NOT NULL"
    );
    assert!(
        !column_exists(&pool, "buildings", "organization_id").await,
        "organization_id supprimée"
    );

    // === DOWN étape 3 : ré-ajoute organization_id + remplit + NOT NULL acp_id off ===
    run_sql_file(&pool, "20260601040000_buildings_acp_id_not_null.down.sql")
        .await
        .expect("DOWN 040000");
    assert!(
        column_exists(&pool, "buildings", "organization_id").await,
        "organization_id ré-ajoutée"
    );
    assert!(
        column_is_nullable(&pool, "buildings", "acp_id").await,
        "acp_id redevient NULLABLE"
    );
    // Restaure organization_id depuis ACP miroir : chaque building doit
    // pointer sur son org d'origine.
    for (org_id, building_ids) in &seeded {
        for bid in building_ids {
            let row = sqlx::query("SELECT organization_id FROM buildings WHERE id = $1")
                .bind(bid)
                .fetch_one(&pool)
                .await
                .expect("read building.organization_id");
            let restored: Option<Uuid> = row.get(0);
            assert_eq!(
                restored,
                Some(*org_id),
                "organization_id restauré pour {}",
                bid
            );
        }
    }

    // === DOWN étape 2 : delete ACPs miroirs sans building + nullify acp_id ===
    run_sql_file(&pool, "20260601030000_backfill_buildings_acp_id.down.sql")
        .await
        .expect("DOWN 030000");
    assert_eq!(
        count_buildings_without_acp(&pool).await,
        6,
        "tous les acp_id remis à NULL après DOWN backfill"
    );
    assert_eq!(
        count_acps(&pool).await,
        0,
        "ACPs miroirs sans building supprimées"
    );

    // === DOWN étape 1 : DROP acp_id + DROP INDEX ===
    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.down.sql")
        .await
        .expect("DOWN 020000");
    assert!(
        !column_exists(&pool, "buildings", "acp_id").await,
        "acp_id supprimée après DOWN final"
    );
    assert!(
        column_exists(&pool, "buildings", "organization_id").await,
        "organization_id présente (état initial restauré)"
    );
    assert_eq!(count_buildings(&pool).await, 6, "6 buildings préservés");

    env::remove_var("BACKUP_CONFIRMED");
}

// ============================================================================
// @edge — organization sans building → ACP miroir créée mais reste vide
// ============================================================================

#[tokio::test]
#[serial]
async fn edge_organization_without_building_yields_empty_mirror_acp() {
    env::set_var("BACKUP_CONFIRMED", "true");
    let (pool, _c) = setup_db_pre_story_12().await;

    // Seed : 1 org avec 2 buildings, 1 org sans building.
    let _seeded =
        seed_orgs_and_buildings(&pool, &[("Cabinet Maury", 2), ("Cabinet Vide", 0)]).await;
    assert_eq!(count_buildings(&pool).await, 2);

    // UP × 2 (pas l'étape 3, on regarde l'effet du backfill).
    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.sql")
        .await
        .expect("UP 020000");
    backup_gate().expect("backup");
    run_sql_file(&pool, "20260601030000_backfill_buildings_acp_id.sql")
        .await
        .expect("UP 030000");

    // Seul Cabinet Maury devrait avoir une ACP miroir (org sans building
    // = pas touchée par le backfill, qui itère sur DISTINCT organization_id
    // PRÉSENT dans `buildings`).
    assert_eq!(
        count_acps(&pool).await,
        1,
        "1 ACP miroir (org Cabinet Maury qui a des buildings)"
    );
    assert_eq!(
        count_buildings_without_acp(&pool).await,
        0,
        "0 building orphelin"
    );

    env::remove_var("BACKUP_CONFIRMED");
}

// ============================================================================
// @security — sans BACKUP_CONFIRMED=true → refus migration backfill
// ============================================================================

#[tokio::test]
#[serial]
async fn security_backfill_refuses_without_backup_confirmed_env() {
    env::remove_var("BACKUP_CONFIRMED");
    let (pool, _c) = setup_db_pre_story_12().await;
    let _seeded = seed_orgs_and_buildings(&pool, &[("Cabinet Maury", 1)]).await;

    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.sql")
        .await
        .expect("UP 020000");

    // Gate doit refuser l'application 030000.
    let res = backup_gate();
    assert!(
        res.is_err(),
        "backup_gate doit refuser sans BACKUP_CONFIRMED=true"
    );
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("BACKUP_CONFIRMED"),
        "message d'erreur explicite : {}",
        err_msg
    );

    // Vérifie que rien n'a été migré.
    assert_eq!(count_acps(&pool).await, 0);
    assert_eq!(count_buildings_without_acp(&pool).await, 1);
}

// ============================================================================
// @negative — interrompue après étape 2 → DOWN restore intégral
// ============================================================================

#[tokio::test]
#[serial]
async fn negative_interrupted_after_step_2_rollback_restores_state() {
    env::set_var("BACKUP_CONFIRMED", "true");
    let (pool, _c) = setup_db_pre_story_12().await;
    let seeded = seed_orgs_and_buildings(&pool, &[("Cabinet Maury", 2)]).await;
    let initial_buildings = count_buildings(&pool).await;
    assert_eq!(initial_buildings, 2);

    // UP étape 1 + 2 (étape 3 jamais lancée — interruption simulée).
    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.sql")
        .await
        .expect("UP 020000");
    backup_gate().expect("backup");
    run_sql_file(&pool, "20260601030000_backfill_buildings_acp_id.sql")
        .await
        .expect("UP 030000");
    assert_eq!(count_acps(&pool).await, 1);
    assert_eq!(count_buildings_without_acp(&pool).await, 0);

    // Rollback : DOWN étape 2 puis DOWN étape 1.
    run_sql_file(&pool, "20260601030000_backfill_buildings_acp_id.down.sql")
        .await
        .expect("DOWN 030000");
    run_sql_file(&pool, "20260601020000_add_buildings_acp_id.down.sql")
        .await
        .expect("DOWN 020000");

    // État doit être identique au départ : org_id NOT NULL, 2 buildings,
    // 0 ACP, organization_id présente.
    assert_eq!(count_buildings(&pool).await, initial_buildings);
    assert_eq!(count_acps(&pool).await, 0);
    assert!(column_exists(&pool, "buildings", "organization_id").await);
    assert!(!column_exists(&pool, "buildings", "acp_id").await);

    // Et chaque building pointe encore sur son org d'origine.
    let (org_id, building_ids) = &seeded[0];
    for bid in building_ids {
        let row = sqlx::query("SELECT organization_id FROM buildings WHERE id = $1")
            .bind(bid)
            .fetch_one(&pool)
            .await
            .expect("read building");
        let restored: Option<Uuid> = row.get(0);
        assert_eq!(restored, Some(*org_id));
    }

    env::remove_var("BACKUP_CONFIRMED");
}

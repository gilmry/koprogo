// Hotfix #602 follow-up — shared helper for BDD/E2E tests that need to
// associate a building with an organization through the new `acps` table.
//
// Post-hotfix #602, `buildings.organization_id` was dropped and replaced by
// `buildings.acp_id` (FK -> `acps.id`). Tests that previously passed
// `org_id` directly into `Building::new(org_id, ...)` now panic at runtime
// with `buildings_acp_id_fkey` violation. This helper materialises an ACP
// for the given org_id (or returns the first existing one) so callers can
// safely pass the returned `acp_id` to `Building::new(acp_id, ...)`.
//
// Each BDD binary is standalone and does NOT pull in `tests/common/mod.rs`
// (which would drag the full AppState setup). We expose this lightweight
// helper via `#[path = "common/acp_test_helper.rs"] mod acp_test_helper;`
// at the top of each BDD file.

use uuid::Uuid;

/// Ensure an ACP exists for `org_id` and return its id.
/// Idempotent — reuses the first existing ACP for the org if any.
#[allow(dead_code)]
pub async fn ensure_default_acp_for_org(pool: &sqlx::PgPool, org_id: Uuid) -> Uuid {
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM acps WHERE organization_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await
    .expect("lookup acp for org");

    if let Some((id,)) = existing {
        return id;
    }

    let acp_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let short = org_id.simple().to_string();
    let short_prefix = &short[..8];
    let acp_name = format!("BDD ACP ({})", short_prefix);
    let acp_slug = format!("bdd-acp-{}", short_prefix);

    sqlx::query(
        r#"INSERT INTO acps (id, organization_id, name, slug, legal_status,
            address_street, address_postal_code, address_city, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
    )
    .bind(acp_id)
    .bind(org_id)
    .bind(&acp_name)
    .bind(&acp_slug)
    .bind("copropriete_belge")
    .bind("Adresse a completer")
    .bind("0000")
    .bind("A completer")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("create default acp for org");

    acp_id
}

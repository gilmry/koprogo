//! `PostgresAcpRepository` — adapter sqlx pour le port `AcpRepository`.
//!
//! Story 1.1. Utilise `sqlx::query` runtime (pas `query_as!` macro) pour
//! cohérence avec le pattern existant `PostgresBuildingRepository` (évite
//! la dépendance compile-time à la DB en CI ; cf. mémoire
//! `use-docker-compose-for-tooling`).

use crate::application::error::AppError;
use crate::application::ports::{AcpRepository, ListScope};
use crate::domain::entities::{Acp, AcpLegalStatus, AcpMetrics};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresAcpRepository {
    pool: DbPool,
}

impl PostgresAcpRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_acp(row: &sqlx::postgres::PgRow) -> Acp {
        let legal_status_str: String = row.get("legal_status");
        Acp {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            slug: row.get("slug"),
            legal_status: AcpLegalStatus::from_db_str(&legal_status_str),
            total_tantiemes: row.get("total_tantiemes"),
            bce_number: row.get("bce_number"),
            address_street: row.get("address_street"),
            address_postal_code: row.get("address_postal_code"),
            address_city: row.get("address_city"),
            // Story H13 — fonds réserve/roulement. `try_get` + défaut : robuste
            // aux SELECT qui n'incluent pas (encore) ces colonnes (ex. branches
            // list() All/Organization), pas de panic.
            reserve_fund_balance: row.try_get("reserve_fund_balance").unwrap_or(Decimal::ZERO),
            working_capital_balance: row
                .try_get("working_capital_balance")
                .unwrap_or(Decimal::ZERO),
            reserve_fund_waived: row.try_get("reserve_fund_waived").unwrap_or(false),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl AcpRepository for PostgresAcpRepository {
    async fn create(&self, acp: &Acp) -> Result<Acp, AppError> {
        sqlx::query(
            r#"
            INSERT INTO acps (
                id, organization_id, name, slug, legal_status, bce_number,
                address_street, address_postal_code, address_city,
                total_tantiemes, created_at, updated_at,
                reserve_fund_balance, working_capital_balance, reserve_fund_waived
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(acp.id)
        .bind(acp.organization_id)
        .bind(&acp.name)
        .bind(&acp.slug)
        .bind(acp.legal_status.as_db_str())
        .bind(&acp.bce_number)
        .bind(&acp.address_street)
        .bind(&acp.address_postal_code)
        .bind(&acp.address_city)
        .bind(acp.total_tantiemes)
        .bind(acp.created_at)
        .bind(acp.updated_at)
        .bind(acp.reserve_fund_balance)
        .bind(acp.working_capital_balance)
        .bind(acp.reserve_fund_waived)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // Conflits unicité (slug ou id en doublon) → Conflict typé.
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict(format!("ACP unique violation: {}", db_err));
                }
            }
            AppError::Database(e.to_string())
        })?;

        Ok(acp.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Acp>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, name, slug, legal_status, bce_number,
                   address_street, address_postal_code, address_city,
                   total_tantiemes, created_at, updated_at,
                   reserve_fund_balance, working_capital_balance, reserve_fund_waived
            FROM acps
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.as_ref().map(Self::row_to_acp))
    }

    async fn find_by_id_with_metrics(
        &self,
        id: Uuid,
    ) -> Result<Option<(Acp, AcpMetrics)>, AppError> {
        // Sous-requêtes indépendantes : évite le fan-out du JOIN
        // (buildings × units multiplierait `total_units`). Chaque agrégat est
        // calculé séparément sur le périmètre de l'ACP. `SUM(quota::NUMERIC)`
        // reste Decimal exact (ADR-0007/0008). Story H6 — ADR-0010.
        let row = sqlx::query(
            r#"
            SELECT
                a.id, a.organization_id, a.name, a.slug, a.legal_status, a.bce_number,
                a.address_street, a.address_postal_code, a.address_city,
                a.total_tantiemes, a.created_at, a.updated_at,
                a.reserve_fund_balance, a.working_capital_balance, a.reserve_fund_waived,
                (SELECT COALESCE(COUNT(u.id), 0)::INT
                   FROM buildings b JOIN units u ON u.building_id = b.id
                   WHERE b.acp_id = a.id)                                   AS units_count,
                (SELECT COALESCE(SUM(u.quota::NUMERIC), 0::NUMERIC)
                   FROM buildings b JOIN units u ON u.building_id = b.id
                   WHERE b.acp_id = a.id)                                   AS quota_sum,
                (SELECT COALESCE(SUM(b.total_units), 0)::INT
                   FROM buildings b WHERE b.acp_id = a.id)                  AS declared_units_total,
                (SELECT COALESCE(COUNT(*), 0)::INT
                   FROM buildings b WHERE b.acp_id = a.id)                  AS buildings_count
            FROM acps a
            WHERE a.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|row| {
            let acp = Self::row_to_acp(&row);
            let metrics = AcpMetrics {
                units_count: row.try_get("units_count").unwrap_or(0),
                declared_units_total: row.try_get("declared_units_total").unwrap_or(0),
                quota_sum: row.try_get("quota_sum").unwrap_or(Decimal::ZERO),
                buildings_count: row.try_get("buildings_count").unwrap_or(0),
            };
            (acp, metrics)
        }))
    }

    async fn list(&self, scope: ListScope) -> Result<Vec<Acp>, AppError> {
        let rows = match scope {
            ListScope::All => sqlx::query(
                r#"
                SELECT id, organization_id, name, slug, legal_status, bce_number,
                       address_street, address_postal_code, address_city,
                       created_at, updated_at
                FROM acps
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?,

            ListScope::Organization(org_id) => sqlx::query(
                r#"
                SELECT id, organization_id, name, slug, legal_status, bce_number,
                       address_street, address_postal_code, address_city,
                       created_at, updated_at
                FROM acps
                WHERE organization_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?,

            ListScope::Owner(user_id) => {
                // Story 1.1 — scope Owner direct via user_role_assignments
                // sur scope='acp'. Le filtrage transitif (via building/unit)
                // arrivera en Story 1.3 (`list_buildings` refacto rôle).
                sqlx::query(
                    r#"
                    SELECT a.id, a.organization_id, a.name, a.slug, a.legal_status,
                           a.bce_number, a.address_street, a.address_postal_code,
                           a.address_city, a.total_tantiemes, a.created_at, a.updated_at
                    FROM acps a
                    INNER JOIN user_role_assignments ura
                        ON ura.scope = 'acp'
                       AND ura.scope_id = a.id
                    WHERE ura.user_id = $1
                    ORDER BY a.created_at DESC
                    "#,
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
                // L'absence de colonnes scope/scope_id sur
                // user_role_assignments (avant migration future Story 3.5)
                // ne doit PAS faire crasher la lecture : on retourne une
                // liste vide en logguant — propre, pas de panic.
                .unwrap_or_else(|e| {
                    log::warn!(
                        "Owner-scope ACP listing fell back to empty (likely \
                         missing user_role_assignments scope/scope_id columns \
                         pending Story 3.5): {}",
                        e
                    );
                    Vec::new()
                })
            }
        };

        Ok(rows.iter().map(Self::row_to_acp).collect())
    }

    async fn update(&self, acp: &Acp) -> Result<Acp, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE acps
            SET organization_id = $2,
                name = $3,
                slug = $4,
                legal_status = $5,
                bce_number = $6,
                address_street = $7,
                address_postal_code = $8,
                address_city = $9,
                total_tantiemes = $10,
                updated_at = $11,
                reserve_fund_balance = $12,
                working_capital_balance = $13,
                reserve_fund_waived = $14
            WHERE id = $1
            "#,
        )
        .bind(acp.id)
        .bind(acp.organization_id)
        .bind(&acp.name)
        .bind(&acp.slug)
        .bind(acp.legal_status.as_db_str())
        .bind(&acp.bce_number)
        .bind(&acp.address_street)
        .bind(&acp.address_postal_code)
        .bind(&acp.address_city)
        .bind(acp.total_tantiemes)
        .bind(acp.updated_at)
        .bind(acp.reserve_fund_balance)
        .bind(acp.working_capital_balance)
        .bind(acp.reserve_fund_waived)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict(format!("ACP unique violation: {}", db_err));
                }
            }
            AppError::Database(e.to_string())
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("ACP {} not found", acp.id)));
        }
        Ok(acp.clone())
    }

    async fn archive(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM acps WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("ACP {} not found", id)));
        }
        Ok(())
    }

    async fn count_buildings(&self, id: Uuid) -> Result<i64, AppError> {
        // Story 1.1 : la colonne `buildings.acp_id` n'existe pas encore
        // (Story 1.2). On répond 0 sans paniquer — implémentation se
        // raffinera en Story 1.2 sans changer la signature du port.
        // Note : on évite un SELECT sur une colonne inexistante (cause
        // une erreur SQL) ; on renvoie 0 directement.
        let _ = id;
        Ok(0)
    }
}

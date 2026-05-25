//! `PostgresAcpRepository` — adapter sqlx pour le port `AcpRepository`.
//!
//! Story 1.1. Utilise `sqlx::query` runtime (pas `query_as!` macro) pour
//! cohérence avec le pattern existant `PostgresBuildingRepository` (évite
//! la dépendance compile-time à la DB en CI ; cf. mémoire
//! `use-docker-compose-for-tooling`).

use crate::application::error::AppError;
use crate::application::ports::{AcpRepository, ListScope};
use crate::domain::entities::{Acp, AcpLegalStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
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
            bce_number: row.get("bce_number"),
            address_street: row.get("address_street"),
            address_postal_code: row.get("address_postal_code"),
            address_city: row.get("address_city"),
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
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
        .bind(acp.created_at)
        .bind(acp.updated_at)
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
                   created_at, updated_at
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
                           a.address_city, a.created_at, a.updated_at
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
                updated_at = $10
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
        .bind(acp.updated_at)
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

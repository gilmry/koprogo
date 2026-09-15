//! PostgreSQL implementation of [`LienNotaireRepository`] (#845 — ADR 0051).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4).

use crate::application::error::AppError;
use crate::application::ports::LienNotaireRepository;
use crate::domain::entities::LienNotaire;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresLienNotaireRepository {
    pool: DbPool,
}

impl PostgresLienNotaireRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_lien(row: &sqlx::postgres::PgRow) -> LienNotaire {
        LienNotaire {
            id: row.get("id"),
            etat_date_id: row.get("etat_date_id"),
            token_hash: row.get("token_hash"),
            emis_par: row.get("emis_par"),
            cree_le: row.get("cree_le"),
            expire_le: row.get("expire_le"),
            revoque_le: row.get("revoque_le"),
            revoque_par: row.get("revoque_par"),
            renouvele_le: row.get("renouvele_le"),
            mis_a_jour_le: row.get("mis_a_jour_le"),
        }
    }
}

#[async_trait]
impl LienNotaireRepository for PostgresLienNotaireRepository {
    async fn save(&self, lien: &LienNotaire) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO liens_notaire (
                id, token_hash, etat_date_id, emis_par,
                cree_le, expire_le, revoque_le, revoque_par,
                renouvele_le, mis_a_jour_le
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(lien.id)
        .bind(&lien.token_hash)
        .bind(lien.etat_date_id)
        .bind(lien.emis_par)
        .bind(lien.cree_le)
        .bind(lien.expire_le)
        .bind(lien.revoque_le)
        .bind(lien.revoque_par)
        .bind(lien.renouvele_le)
        .bind(lien.mis_a_jour_le)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<LienNotaire>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, etat_date_id, emis_par,
                   cree_le, expire_le, revoque_le, revoque_par,
                   renouvele_le, mis_a_jour_le
            FROM liens_notaire
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_lien(&r)))
    }

    async fn find_active_by_etat_date_id(
        &self,
        etat_date_id: Uuid,
    ) -> Result<Option<LienNotaire>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, etat_date_id, emis_par,
                   cree_le, expire_le, revoque_le, revoque_par,
                   renouvele_le, mis_a_jour_le
            FROM liens_notaire
            WHERE etat_date_id = $1 AND revoque_le IS NULL
            ORDER BY cree_le DESC
            LIMIT 1
            "#,
        )
        .bind(etat_date_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_lien(&r)))
    }

    async fn update(&self, lien: &LienNotaire) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE liens_notaire
            SET expire_le = $2,
                revoque_le = $3,
                revoque_par = $4,
                renouvele_le = $5,
                mis_a_jour_le = $6
            WHERE id = $1
            "#,
        )
        .bind(lien.id)
        .bind(lien.expire_le)
        .bind(lien.revoque_le)
        .bind(lien.revoque_par)
        .bind(lien.renouvele_le)
        .bind(lien.mis_a_jour_le)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

//! `PostgresPortfolioRepository` — adapter sqlx pour le port `PortfolioRepository`.
//!
//! Story 2.1. Pattern aligné sur `PostgresAcpRepository` (Story 1.1) :
//! `sqlx::query` runtime (pas `query_as!` macro) — évite la dépendance
//! compile-time à la DB en CI (cf. mémoire `use-docker-compose-for-tooling`).

use crate::application::error::AppError;
use crate::application::ports::{PortfolioBuildingEntry, PortfolioRepository};
use crate::domain::entities::{Portfolio, PortfolioBuilding, PortfolioShare};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresPortfolioRepository {
    pool: DbPool,
}

impl PostgresPortfolioRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_portfolio(row: &sqlx::postgres::PgRow) -> Portfolio {
        Portfolio {
            id: row.get("id"),
            owner_user_id: row.get("owner_user_id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_share(row: &sqlx::postgres::PgRow) -> PortfolioShare {
        PortfolioShare {
            portfolio_id: row.get("portfolio_id"),
            shared_with_user_id: row.get("shared_with_user_id"),
            can_edit: row.get("can_edit"),
            shared_at: row.get("shared_at"),
        }
    }
}

#[async_trait]
impl PortfolioRepository for PostgresPortfolioRepository {
    async fn create(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError> {
        sqlx::query(
            r#"
            INSERT INTO portfolios (
                id, owner_user_id, name, description, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(portfolio.id)
        .bind(portfolio.owner_user_id)
        .bind(&portfolio.name)
        .bind(&portfolio.description)
        .bind(portfolio.created_at)
        .bind(portfolio.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict(format!("Portfolio unique violation: {}", db_err));
                }
                if db_err.is_foreign_key_violation() {
                    return AppError::Validation(format!("Portfolio FK violation: {}", db_err));
                }
            }
            AppError::Database(e.to_string())
        })?;
        Ok(portfolio.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Portfolio>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, owner_user_id, name, description, created_at, updated_at
            FROM portfolios
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_portfolio))
    }

    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Portfolio>, AppError> {
        // UNION : portfolios owned + portfolios shared. DISTINCT pour éviter
        // doublons si un user est shared sur son propre portfolio (cas
        // théorique).
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT p.id, p.owner_user_id, p.name, p.description, p.created_at, p.updated_at
            FROM portfolios p
            LEFT JOIN portfolio_shares s ON s.portfolio_id = p.id
            WHERE p.owner_user_id = $1 OR s.shared_with_user_id = $1
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_portfolio).collect())
    }

    async fn update(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE portfolios
            SET name = $2,
                description = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(portfolio.id)
        .bind(&portfolio.name)
        .bind(&portfolio.description)
        .bind(portfolio.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Portfolio {} not found",
                portfolio.id
            )));
        }
        Ok(portfolio.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM portfolios WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Portfolio {} not found", id)));
        }
        Ok(())
    }

    async fn add_building(
        &self,
        portfolio_id: Uuid,
        building_id: Uuid,
        is_favorite: bool,
    ) -> Result<PortfolioBuilding, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO portfolio_buildings (portfolio_id, building_id, is_favorite, added_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (portfolio_id, building_id)
            DO UPDATE SET is_favorite = EXCLUDED.is_favorite
            RETURNING portfolio_id, building_id, is_favorite, added_at
            "#,
        )
        .bind(portfolio_id)
        .bind(building_id)
        .bind(is_favorite)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_foreign_key_violation() {
                    return AppError::NotFound(format!(
                        "Portfolio or Building not found: {}",
                        db_err
                    ));
                }
            }
            AppError::Database(e.to_string())
        })?;
        Ok(PortfolioBuilding {
            portfolio_id: row.get("portfolio_id"),
            building_id: row.get("building_id"),
            is_favorite: row.get("is_favorite"),
            added_at: row.get("added_at"),
        })
    }

    async fn remove_building(&self, portfolio_id: Uuid, building_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM portfolio_buildings
            WHERE portfolio_id = $1 AND building_id = $2
            "#,
        )
        .bind(portfolio_id)
        .bind(building_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Building {} not in portfolio {}",
                building_id, portfolio_id
            )));
        }
        Ok(())
    }

    async fn list_buildings(
        &self,
        portfolio_id: Uuid,
    ) -> Result<Vec<PortfolioBuildingEntry>, AppError> {
        // Tri Story 2.1 AC @happy : favoris d'abord puis added_at DESC.
        let rows = sqlx::query(
            r#"
            SELECT portfolio_id, building_id, is_favorite, added_at
            FROM portfolio_buildings
            WHERE portfolio_id = $1
            ORDER BY is_favorite DESC, added_at DESC
            "#,
        )
        .bind(portfolio_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PortfolioBuildingEntry {
                portfolio_id: r.get("portfolio_id"),
                building_id: r.get("building_id"),
                is_favorite: r.get("is_favorite"),
            })
            .collect())
    }

    async fn share_with(
        &self,
        portfolio_id: Uuid,
        shared_with_user_id: Uuid,
        can_edit: bool,
    ) -> Result<PortfolioShare, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO portfolio_shares
                (portfolio_id, shared_with_user_id, can_edit, shared_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (portfolio_id, shared_with_user_id)
            DO UPDATE SET can_edit = EXCLUDED.can_edit
            RETURNING portfolio_id, shared_with_user_id, can_edit, shared_at
            "#,
        )
        .bind(portfolio_id)
        .bind(shared_with_user_id)
        .bind(can_edit)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_foreign_key_violation() {
                    return AppError::NotFound(format!("Portfolio or User not found: {}", db_err));
                }
            }
            AppError::Database(e.to_string())
        })?;
        Ok(Self::row_to_share(&row))
    }

    async fn unshare(&self, portfolio_id: Uuid, shared_with_user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM portfolio_shares
            WHERE portfolio_id = $1 AND shared_with_user_id = $2
            "#,
        )
        .bind(portfolio_id)
        .bind(shared_with_user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Share not found for user {} on portfolio {}",
                shared_with_user_id, portfolio_id
            )));
        }
        Ok(())
    }

    async fn list_shares(&self, portfolio_id: Uuid) -> Result<Vec<PortfolioShare>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT portfolio_id, shared_with_user_id, can_edit, shared_at
            FROM portfolio_shares
            WHERE portfolio_id = $1
            ORDER BY shared_at DESC
            "#,
        )
        .bind(portfolio_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_share).collect())
    }
}

//! Adaptateur PostgreSQL du registre de modules — Story 5.1 (#585).
//!
//! **Requêtes vérifiées à l'exécution, pas à la compilation.** Le reste du
//! dépôt emploie surtout les macros `sqlx::query!`, qui exigent le cache
//! `.sqlx`. Ce cache est suivi par git et `sqlx prepare` le vide avant de le
//! refaire : le régénérer pour trois requêtes ferait porter à cette story le
//! risque de perdre les entrées de tout le monde. Les requêtes ici sont
//! simples (une table, deux colonnes) et leur typage est couvert par les
//! tests d'intégration testcontainers, qui exécutent le SQL pour de vrai.
//! Précédent dans le dépôt : `charge_distribution_repository_impl.rs`.
//!
//! INV-27 — rien n'est jamais supprimé : `archived_at` est posée puis
//! retirée. Réactiver retrouve la ligne, donc les données du module.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::ModuleRegistry;
use crate::domain::entities::Module;

pub struct PostgresModuleRegistry {
    pool: PgPool,
}

impl PostgresModuleRegistry {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn erreur_sql(e: sqlx::Error) -> AppError {
    AppError::Database(e.to_string())
}

#[async_trait]
impl ModuleRegistry for PostgresModuleRegistry {
    async fn list_enabled(&self, acp_id: Uuid) -> Result<Vec<Module>, AppError> {
        let noms: Vec<String> = sqlx::query_scalar(
            "SELECT module FROM acp_enabled_modules \
             WHERE acp_id = $1 AND archived_at IS NULL \
             ORDER BY module",
        )
        .bind(acp_id)
        .fetch_all(&self.pool)
        .await
        .map_err(erreur_sql)?;

        // Un nom en base que l'enum ne connaît pas signifie que la contrainte
        // SQL et l'enum Rust ont divergé. On refuse plutôt que de filtrer en
        // silence : filtrer rendrait une liste incomplète qui se lirait comme
        // « ce module est éteint ». `garde_enum_contre_contrainte` empêche
        // normalement d'en arriver là ; ceci est la ceinture.
        noms.into_iter()
            .map(|nom| {
                Module::depuis_nom(&nom).ok_or_else(|| {
                    AppError::Database(format!(
                        "Module « {nom} » présent en base mais inconnu de l'enum Rust : \
                         la contrainte SQL et le domaine ont divergé"
                    ))
                })
            })
            .collect()
    }

    async fn enable(&self, acp_id: Uuid, module: Module) -> Result<(), AppError> {
        // Idempotent : le conflit sur (acp_id, module) rallume la ligne
        // existante au lieu d'en créer une seconde. C'est ce qui fait que le
        // cycle activer/désactiver/réactiver laisse les données intactes.
        sqlx::query(
            "INSERT INTO acp_enabled_modules (acp_id, module) VALUES ($1, $2) \
             ON CONFLICT (acp_id, module) \
             DO UPDATE SET archived_at = NULL, enabled_at = NOW()",
        )
        .bind(acp_id)
        .bind(module.as_str())
        .execute(&self.pool)
        .await
        .map_err(erreur_sql)?;
        Ok(())
    }

    async fn disable(&self, acp_id: Uuid, module: Module) -> Result<(), AppError> {
        // Pas de DELETE. Éteindre un module doit rester réversible (INV-27).
        // `archived_at IS NULL` dans le WHERE rend l'appel idempotent sans
        // écraser la date d'archivage d'une extinction précédente.
        sqlx::query(
            "UPDATE acp_enabled_modules SET archived_at = NOW() \
             WHERE acp_id = $1 AND module = $2 AND archived_at IS NULL",
        )
        .bind(acp_id)
        .bind(module.as_str())
        .execute(&self.pool)
        .await
        .map_err(erreur_sql)?;
        Ok(())
    }

    async fn is_enabled(&self, acp_id: Uuid, module: Module) -> Result<bool, AppError> {
        let actif: Option<bool> = sqlx::query_scalar(
            "SELECT TRUE FROM acp_enabled_modules \
             WHERE acp_id = $1 AND module = $2 AND archived_at IS NULL",
        )
        .bind(acp_id)
        .bind(module.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(erreur_sql)?;
        Ok(actif.unwrap_or(false))
    }
}

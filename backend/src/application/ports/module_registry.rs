//! Port (trait) du registre de modules — Story 5.1 (#585), ADR-0015.
//!
//! Hexagonal : le trait vit côté application, l'implémentation PostgreSQL
//! dans `infrastructure/database/repositories/module_registry_impl.rs`.
//!
//! Toutes les méthodes retournent `Result<_, AppError>` (CRITICAL.md §4).

use crate::application::error::AppError;
use crate::domain::entities::Module;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ModuleRegistry: Send + Sync {
    /// Les modules actifs (`archived_at IS NULL`) d'une ACP.
    ///
    /// Une ACP inconnue rend une liste vide, pas une erreur : la garde de
    /// portée s'exécute **avant** et c'est elle qui décide si l'appelant a
    /// le droit de savoir que l'ACP n'existe pas.
    async fn list_enabled(&self, acp_id: Uuid) -> Result<Vec<Module>, AppError>;

    /// Allume un module. Idempotent : rallumer un module déjà actif remet
    /// simplement `archived_at` à `NULL` et ne duplique pas la ligne — c'est
    /// ce qui permet le cycle activer/désactiver/réactiver d'INV-27 sans
    /// perdre les données.
    async fn enable(&self, acp_id: Uuid, module: Module) -> Result<(), AppError>;

    /// Éteint un module. Idempotent également. **Ne supprime rien** : pose
    /// `archived_at`. Le refus d'éteindre `identity` est une règle métier et
    /// vit dans le use-case, pas ici.
    async fn disable(&self, acp_id: Uuid, module: Module) -> Result<(), AppError>;

    /// Chemin chaud : appelé par `ModuleGuard` à chaque requête gardée.
    async fn is_enabled(&self, acp_id: Uuid, module: Module) -> Result<bool, AppError>;
}

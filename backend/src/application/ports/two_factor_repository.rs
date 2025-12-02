use crate::domain::entities::TwoFactorSecret;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository port for two-factor authentication secrets
#[async_trait]
pub trait TwoFactorRepository: Send + Sync {
    /// Create a new 2FA secret for a user
    async fn create(&self, secret: &TwoFactorSecret) -> Result<TwoFactorSecret, String>;

    /// Find 2FA secret by user ID
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<TwoFactorSecret>, String>;

    /// Update 2FA secret (enable, disable, mark used, etc.)
    async fn update(&self, secret: &TwoFactorSecret) -> Result<TwoFactorSecret, String>;

    /// Delete 2FA secret (when user disables 2FA)
    async fn delete(&self, user_id: Uuid) -> Result<(), String>;

    /// Find all users with enabled 2FA that need reverification (not used in 90 days)
    async fn find_needing_reverification(&self) -> Result<Vec<TwoFactorSecret>, String>;

    /// Find all users with low backup codes (< 3 remaining)
    async fn find_with_low_backup_codes(&self) -> Result<Vec<TwoFactorSecret>, String>;
}

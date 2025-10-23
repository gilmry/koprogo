use crate::domain::entities::RefreshToken;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create(&self, refresh_token: &RefreshToken) -> Result<RefreshToken, String>;
    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, String>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, String>;
    async fn revoke(&self, token: &str) -> Result<bool, String>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, String>;
    async fn delete_expired(&self) -> Result<u64, String>;
}

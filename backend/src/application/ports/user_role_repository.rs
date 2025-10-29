use crate::domain::entities::UserRoleAssignment;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRoleRepository: Send + Sync {
    async fn create(&self, assignment: &UserRoleAssignment) -> Result<UserRoleAssignment, String>;
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, String>;
    async fn set_primary_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<UserRoleAssignment, String>;
}

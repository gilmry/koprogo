use crate::domain::entities::UserRoleAssignment;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait UserRoleRepository: Send + Sync {
    async fn create(&self, assignment: &UserRoleAssignment) -> Result<UserRoleAssignment, String>;
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>, String>;
    async fn list_for_users(
        &self,
        user_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<UserRoleAssignment>>, String>;
    async fn replace_all(
        &self,
        user_id: Uuid,
        assignments: &[UserRoleAssignment],
    ) -> Result<(), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, String>;
    async fn set_primary_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<UserRoleAssignment, String>;
}

#[cfg(test)]
pub use tests::MockUserRoleRepo;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub UserRoleRepo {}

        #[async_trait]
        impl UserRoleRepository for UserRoleRepo {
            async fn create(&self, assignment: &UserRoleAssignment) -> Result<UserRoleAssignment, String>;
            async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>, String>;
            async fn list_for_users(
                &self,
                user_ids: &[Uuid],
            ) -> Result<HashMap<Uuid, Vec<UserRoleAssignment>>, String>;
            async fn replace_all(
                &self,
                user_id: Uuid,
                assignments: &[UserRoleAssignment],
            ) -> Result<(), String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, String>;
            async fn set_primary_role(
                &self,
                user_id: Uuid,
                role_id: Uuid,
            ) -> Result<UserRoleAssignment, String>;
        }
    }
}

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
    /// Delete a single role assignment by id.
    ///
    /// Story B0bis — gap fill for Story 3.1: the CRUD REST endpoint
    /// `DELETE /users/{user_id}/role-assignments/{id}` revokes a single row
    /// without rewriting the whole set. Returns `true` if a row was deleted,
    /// `false` if no row matched.
    async fn delete_by_id(&self, id: Uuid) -> Result<bool, String>;
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
            async fn delete_by_id(&self, id: Uuid) -> Result<bool, String>;
        }
    }
}

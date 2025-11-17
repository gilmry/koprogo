use crate::domain::entities::User;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
    async fn find_all(&self) -> Result<Vec<User>, String>;
    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
    async fn update(&self, user: &User) -> Result<User, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub UserRepo {}

        #[async_trait]
        impl UserRepository for UserRepo {
            async fn create(&self, user: &User) -> Result<User, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
            async fn find_all(&self) -> Result<Vec<User>, String>;
            async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
            async fn update(&self, user: &User) -> Result<User, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
        }
    }
}

#[cfg(test)]
pub use tests::MockUserRepo;

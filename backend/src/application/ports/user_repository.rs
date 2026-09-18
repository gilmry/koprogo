use crate::domain::entities::User;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
    async fn find_all(&self) -> Result<Vec<User>, String>;

    /// Une page d'utilisateurs, filtrée par `recherche` si elle est fournie.
    ///
    /// `find_all` existe toujours, mais `GET /users` ne l'appelle plus : elle
    /// rendait la table entière — **2 399 187 octets en 667 ms** pour 4 120
    /// lignes, mesuré le 2026-09-18 sur la recette (#953). Le même écran
    /// après pagination de `/organizations` la veille rendait 6 190 octets en
    /// 18,8 ms.
    ///
    /// `Option<String>` et non `Option<&str>` : mockall ne sait pas exprimer
    /// la durée de vie ici, et un port n'a pas à se tordre pour un détail de
    /// doublure. Le coût est une allocation par appel.
    async fn find_page(
        &self,
        recherche: Option<String>,
        role: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, String>;

    /// Le total correspondant au même filtre, pour que la pagination sache
    /// combien de pages annoncer.
    async fn count_matching(
        &self,
        recherche: Option<String>,
        role: Option<String>,
    ) -> Result<i64, String>;
    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
    async fn update(&self, user: &User) -> Result<User, String>;
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<bool, String>;
    async fn activate(&self, id: Uuid) -> Result<Option<User>, String>;
    async fn deactivate(&self, id: Uuid) -> Result<Option<User>, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
}

#[cfg(test)]
pub use tests::MockUserRepo;

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
            async fn find_page(
                &self,
                recherche: Option<String>,
                role: Option<String>,
                limit: i64,
                offset: i64,
            ) -> Result<Vec<User>, String>;
            async fn count_matching(
                &self,
                recherche: Option<String>,
                role: Option<String>,
            ) -> Result<i64, String>;
            async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
            async fn update(&self, user: &User) -> Result<User, String>;
            async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<bool, String>;
            async fn activate(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn deactivate(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
        }
    }
}

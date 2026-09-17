use crate::domain::entities::Organization;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, org: &Organization) -> Result<Organization, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, String>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, String>;
    async fn find_all(&self) -> Result<Vec<Organization>, String>;

    /// Une PAGE d'organisations, filtrée par une recherche libre.
    ///
    /// `find_all` rendait la table entière — 2743 lignes sur la recette au
    /// 2026-09-17, d'où 8,2 s d'écran blanc sur `/admin/acps` (#943). Il
    /// reste pour les appelants internes qui ont réellement besoin de tout.
    ///
    /// `recherche` porte sur le nom ET le slug, sans distinction de casse.
    /// `None` ou une chaîne vide ne filtre rien.
    async fn find_page(
        &self,
        recherche: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Organization>, String>;

    /// Le nombre total correspondant à la même recherche, pour que
    /// l'appelant sache combien il en reste. Sans lui, une page est un
    /// fragment dont on ignore la taille du tout.
    async fn count_matching(&self, recherche: Option<String>) -> Result<i64, String>;
    async fn update(&self, org: &Organization) -> Result<Organization, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn count_buildings(&self, org_id: Uuid) -> Result<i64, String>;
}

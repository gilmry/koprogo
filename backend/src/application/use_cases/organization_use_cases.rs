use crate::application::ports::OrganizationRepository;
use crate::domain::entities::{Organization, SubscriptionPlan};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct OrganizationUseCases {
    repo: Arc<dyn OrganizationRepository>,
}

impl OrganizationUseCases {
    pub fn new(repo: Arc<dyn OrganizationRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_all(&self) -> Result<Vec<Organization>, String> {
        self.repo.find_all().await
    }

    pub async fn create(
        &self,
        name: String,
        slug: String,
        contact_email: String,
        contact_phone: Option<String>,
        subscription_plan: String,
    ) -> Result<Organization, String> {
        let plan = subscription_plan
            .parse::<SubscriptionPlan>()
            .map_err(|_| "invalid_plan".to_string())?;

        let (max_buildings, max_users) = plan_limits(&plan);

        let org = Organization {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            slug: slug.trim().to_lowercase(),
            contact_email: contact_email.trim().to_lowercase(),
            contact_phone,
            subscription_plan: plan,
            max_buildings,
            max_users,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        org.validate()
            .map_err(|e| format!("validation_error:{}", e))?;

        self.repo.create(&org).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: String,
        slug: String,
        contact_email: String,
        contact_phone: Option<String>,
        subscription_plan: String,
    ) -> Result<Organization, String> {
        let mut org = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "not_found".to_string())?;

        let plan = subscription_plan
            .parse::<SubscriptionPlan>()
            .map_err(|_| "invalid_plan".to_string())?;

        let (max_buildings, max_users) = plan_limits(&plan);

        org.name = name.trim().to_string();
        org.slug = slug.trim().to_lowercase();
        org.contact_email = contact_email.trim().to_lowercase();
        org.contact_phone = contact_phone;
        org.subscription_plan = plan;
        org.max_buildings = max_buildings;
        org.max_users = max_users;
        org.updated_at = Utc::now();

        org.validate()
            .map_err(|e| format!("validation_error:{}", e))?;

        self.repo.update(&org).await
    }

    pub async fn activate(&self, id: Uuid) -> Result<Organization, String> {
        let mut org = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "not_found".to_string())?;
        org.activate();
        self.repo.update(&org).await
    }

    pub async fn suspend(&self, id: Uuid) -> Result<Organization, String> {
        let mut org = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "not_found".to_string())?;
        org.deactivate();
        self.repo.update(&org).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, String> {
        self.repo.delete(id).await
    }
}

fn plan_limits(plan: &SubscriptionPlan) -> (i32, i32) {
    match plan {
        SubscriptionPlan::Free => (1, 3),
        SubscriptionPlan::Starter => (5, 10),
        SubscriptionPlan::Professional => (20, 50),
        SubscriptionPlan::Enterprise => (999, 999),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;

    struct MockOrgRepository {
        orgs: Vec<Organization>,
    }

    fn make_org(name: &str) -> Organization {
        Organization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            contact_email: "test@test.com".to_string(),
            contact_phone: None,
            subscription_plan: SubscriptionPlan::Free,
            max_buildings: 1,
            max_users: 3,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[async_trait]
    impl OrganizationRepository for MockOrgRepository {
        async fn create(&self, org: &Organization) -> Result<Organization, String> {
            Ok(org.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, String> {
            Ok(self.orgs.iter().find(|o| o.id == id).cloned())
        }
        async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, String> {
            Ok(self.orgs.iter().find(|o| o.slug == slug).cloned())
        }
        async fn find_all(&self) -> Result<Vec<Organization>, String> {
            Ok(self.orgs.clone())
        }
        async fn update(&self, org: &Organization) -> Result<Organization, String> {
            Ok(org.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            Ok(true)
        }
        async fn count_buildings(&self, _org_id: Uuid) -> Result<i64, String> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_list_all() {
        let org = make_org("TestOrg");
        let repo = Arc::new(MockOrgRepository { orgs: vec![org] });
        let uc = OrganizationUseCases::new(repo);
        let result = uc.list_all().await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_create_invalid_plan_returns_error() {
        let repo = Arc::new(MockOrgRepository { orgs: vec![] });
        let uc = OrganizationUseCases::new(repo);
        let result = uc
            .create(
                "Test".to_string(),
                "test".to_string(),
                "a@b.com".to_string(),
                None,
                "invalid_plan".to_string(),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid_plan");
    }

    #[tokio::test]
    async fn test_activate_not_found_returns_error() {
        let repo = Arc::new(MockOrgRepository { orgs: vec![] });
        let uc = OrganizationUseCases::new(repo);
        let result = uc.activate(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "not_found");
    }

    #[tokio::test]
    async fn test_suspend_org() {
        let org = make_org("ActiveOrg");
        let id = org.id;
        let repo = Arc::new(MockOrgRepository { orgs: vec![org] });
        let uc = OrganizationUseCases::new(repo);
        let result = uc.suspend(id).await.unwrap();
        assert!(!result.is_active);
    }
}

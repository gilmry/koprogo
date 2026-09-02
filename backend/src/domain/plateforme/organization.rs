use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubscriptionPlan {
    Free,
    Starter,
    Professional,
    Enterprise,
}

impl std::fmt::Display for SubscriptionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionPlan::Free => write!(f, "free"),
            SubscriptionPlan::Starter => write!(f, "starter"),
            SubscriptionPlan::Professional => write!(f, "professional"),
            SubscriptionPlan::Enterprise => write!(f, "enterprise"),
        }
    }
}

impl std::str::FromStr for SubscriptionPlan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(SubscriptionPlan::Free),
            "starter" => Ok(SubscriptionPlan::Starter),
            "professional" => Ok(SubscriptionPlan::Professional),
            "enterprise" => Ok(SubscriptionPlan::Enterprise),
            _ => Err(format!("Invalid subscription plan: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Organization {
    pub id: Uuid,

    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,

    pub slug: String,

    #[validate(email(message = "Contact email must be valid"))]
    pub contact_email: String,

    pub contact_phone: Option<String>,

    pub subscription_plan: SubscriptionPlan,

    pub max_buildings: i32,

    pub max_users: i32,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn new(
        name: String,
        contact_email: String,
        contact_phone: Option<String>,
        subscription_plan: SubscriptionPlan,
    ) -> Result<Self, String> {
        let slug = Self::generate_slug(&name);

        let (max_buildings, max_users) = Self::get_limits_for_plan(&subscription_plan);

        let org = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            slug,
            contact_email: contact_email.to_lowercase().trim().to_string(),
            contact_phone,
            subscription_plan,
            max_buildings,
            max_users,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        org.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(org)
    }

    fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }

    fn get_limits_for_plan(plan: &SubscriptionPlan) -> (i32, i32) {
        match plan {
            SubscriptionPlan::Free => (1, 3),
            SubscriptionPlan::Starter => (5, 10),
            SubscriptionPlan::Professional => (20, 50),
            SubscriptionPlan::Enterprise => (i32::MAX, i32::MAX),
        }
    }

    pub fn upgrade_plan(&mut self, new_plan: SubscriptionPlan) {
        self.subscription_plan = new_plan;
        let (max_buildings, max_users) = Self::get_limits_for_plan(&self.subscription_plan);
        self.max_buildings = max_buildings;
        self.max_users = max_users;
        self.updated_at = Utc::now();
    }

    pub fn update_contact(&mut self, email: String, phone: Option<String>) -> Result<(), String> {
        self.contact_email = email.to_lowercase().trim().to_string();
        self.contact_phone = phone;
        self.updated_at = Utc::now();

        self.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn can_add_building(&self, current_count: i32) -> bool {
        self.is_active && current_count < self.max_buildings
    }

    pub fn can_add_user(&self, current_count: i32) -> bool {
        self.is_active && current_count < self.max_users
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_organization_success() {
        let org = Organization::new(
            "Test Company".to_string(),
            "contact@test.com".to_string(),
            Some("+33123456789".to_string()),
            SubscriptionPlan::Professional,
        );

        assert!(org.is_ok());
        let org = org.unwrap();
        assert_eq!(org.name, "Test Company");
        assert_eq!(org.slug, "test-company");
        assert_eq!(org.max_buildings, 20);
        assert_eq!(org.max_users, 50);
        assert!(org.is_active);
    }

    #[test]
    fn test_generate_slug() {
        let org = Organization::new(
            "My Super Company!!!".to_string(),
            "contact@test.com".to_string(),
            None,
            SubscriptionPlan::Free,
        )
        .unwrap();

        assert_eq!(org.slug, "my-super-company");
    }

    #[test]
    fn test_subscription_limits() {
        let free_org = Organization::new(
            "Free Org".to_string(),
            "free@test.com".to_string(),
            None,
            SubscriptionPlan::Free,
        )
        .unwrap();
        assert_eq!(free_org.max_buildings, 1);
        assert_eq!(free_org.max_users, 3);

        let starter_org = Organization::new(
            "Starter Org".to_string(),
            "starter@test.com".to_string(),
            None,
            SubscriptionPlan::Starter,
        )
        .unwrap();
        assert_eq!(starter_org.max_buildings, 5);
        assert_eq!(starter_org.max_users, 10);
    }

    #[test]
    fn test_upgrade_plan() {
        let mut org = Organization::new(
            "Test Org".to_string(),
            "test@test.com".to_string(),
            None,
            SubscriptionPlan::Free,
        )
        .unwrap();

        assert_eq!(org.max_buildings, 1);

        org.upgrade_plan(SubscriptionPlan::Professional);
        assert_eq!(org.subscription_plan, SubscriptionPlan::Professional);
        assert_eq!(org.max_buildings, 20);
        assert_eq!(org.max_users, 50);
    }

    #[test]
    fn test_can_add_building() {
        let org = Organization::new(
            "Test Org".to_string(),
            "test@test.com".to_string(),
            None,
            SubscriptionPlan::Starter,
        )
        .unwrap();

        assert!(org.can_add_building(0));
        assert!(org.can_add_building(4));
        assert!(!org.can_add_building(5));
    }

    #[test]
    fn test_deactivate_prevents_adding() {
        let mut org = Organization::new(
            "Test Org".to_string(),
            "test@test.com".to_string(),
            None,
            SubscriptionPlan::Professional,
        )
        .unwrap();

        org.deactivate();
        assert!(!org.can_add_building(0));
        assert!(!org.can_add_user(0));
    }
}

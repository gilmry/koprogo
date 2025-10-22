use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    SuperAdmin,
    Syndic,
    Accountant,
    Owner,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Syndic => write!(f, "syndic"),
            UserRole::Accountant => write!(f, "accountant"),
            UserRole::Owner => write!(f, "owner"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "superadmin" => Ok(UserRole::SuperAdmin),
            "syndic" => Ok(UserRole::Syndic),
            "accountant" => Ok(UserRole::Accountant),
            "owner" => Ok(UserRole::Owner),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,

    #[validate(email(message = "Email must be valid"))]
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
    pub first_name: String,

    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: String,

    pub role: UserRole,

    pub organization_id: Option<Uuid>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        password_hash: String,
        first_name: String,
        last_name: String,
        role: UserRole,
        organization_id: Option<Uuid>,
    ) -> Result<Self, String> {
        let user = Self {
            id: Uuid::new_v4(),
            email: email.to_lowercase().trim().to_string(),
            password_hash,
            first_name: first_name.trim().to_string(),
            last_name: last_name.trim().to_string(),
            role,
            organization_id,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        user.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(user)
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn update_profile(&mut self, first_name: String, last_name: String) -> Result<(), String> {
        self.first_name = first_name.trim().to_string();
        self.last_name = last_name.trim().to_string();
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

    pub fn can_access_building(&self, building_org_id: Option<Uuid>) -> bool {
        match self.role {
            UserRole::SuperAdmin => true,
            _ => self.organization_id == building_org_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_success() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            Some(Uuid::new_v4()),
        );

        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name(), "John Doe");
        assert!(user.is_active);
    }

    #[test]
    fn test_create_user_invalid_email() {
        let user = User::new(
            "invalid-email".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        );

        assert!(user.is_err());
    }

    #[test]
    fn test_update_profile() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        )
        .unwrap();

        let result = user.update_profile("Jane".to_string(), "Smith".to_string());
        assert!(result.is_ok());
        assert_eq!(user.full_name(), "Jane Smith");
    }

    #[test]
    fn test_deactivate_user() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        )
        .unwrap();

        user.deactivate();
        assert!(!user.is_active);
    }

    #[test]
    fn test_superadmin_can_access_all_buildings() {
        let user = User::new(
            "admin@example.com".to_string(),
            "hashed_password".to_string(),
            "Admin".to_string(),
            "User".to_string(),
            UserRole::SuperAdmin,
            None,
        )
        .unwrap();

        assert!(user.can_access_building(Some(Uuid::new_v4())));
        assert!(user.can_access_building(None));
    }

    #[test]
    fn test_regular_user_access_control() {
        let org_id = Uuid::new_v4();
        let user = User::new(
            "syndic@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Syndic".to_string(),
            UserRole::Syndic,
            Some(org_id),
        )
        .unwrap();

        assert!(user.can_access_building(Some(org_id)));
        assert!(!user.can_access_building(Some(Uuid::new_v4())));
    }
}

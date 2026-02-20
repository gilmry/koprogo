use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    SuperAdmin,
    Syndic,
    Accountant,
    BoardMember, // Membre du conseil de copropriété
    Owner,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Syndic => write!(f, "syndic"),
            UserRole::Accountant => write!(f, "accountant"),
            UserRole::BoardMember => write!(f, "board_member"),
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
            "board_member" => Ok(UserRole::BoardMember),
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

    // GDPR Article 18: Right to Restriction of Processing
    pub processing_restricted: bool,
    pub processing_restricted_at: Option<DateTime<Utc>>,

    // GDPR Article 21: Right to Object (Marketing opt-out)
    pub marketing_opt_out: bool,
    pub marketing_opt_out_at: Option<DateTime<Utc>>,

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
            processing_restricted: false,
            processing_restricted_at: None,
            marketing_opt_out: false,
            marketing_opt_out_at: None,
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

    // GDPR Article 16: Right to Rectification
    // Users can correct inaccurate personal data
    pub fn rectify_data(
        &mut self,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<(), String> {
        // At least one field must be provided
        if email.is_none() && first_name.is_none() && last_name.is_none() {
            return Err("No fields provided for rectification".to_string());
        }

        // Validate email format BEFORE modifying anything
        if let Some(ref new_email) = email {
            let email_normalized = new_email.to_lowercase().trim().to_string();
            if !email_normalized.contains('@') || email_normalized.len() < 3 {
                return Err(format!("Invalid email format: {}", new_email));
            }
        }

        // Validate names are not empty BEFORE modifying
        if let Some(ref new_first_name) = first_name {
            if new_first_name.trim().is_empty() {
                return Err("First name cannot be empty".to_string());
            }
        }
        if let Some(ref new_last_name) = last_name {
            if new_last_name.trim().is_empty() {
                return Err("Last name cannot be empty".to_string());
            }
        }

        // Only apply changes after validation passes
        if let Some(new_email) = email {
            self.email = new_email.to_lowercase().trim().to_string();
        }
        if let Some(new_first_name) = first_name {
            self.first_name = new_first_name.trim().to_string();
        }
        if let Some(new_last_name) = last_name {
            self.last_name = new_last_name.trim().to_string();
        }

        self.updated_at = Utc::now();

        // Final validation with full validator
        self.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(())
    }

    // GDPR Article 18: Right to Restriction of Processing
    // Users can request temporary limitation of data processing
    pub fn restrict_processing(&mut self) -> Result<(), String> {
        if self.processing_restricted {
            return Err("Processing is already restricted for this user".to_string());
        }

        self.processing_restricted = true;
        self.processing_restricted_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    // GDPR Article 18: Unrestrict processing (admin action or legal requirement met)
    pub fn unrestrict_processing(&mut self) {
        self.processing_restricted = false;
        // Keep processing_restricted_at for audit trail
        self.updated_at = Utc::now();
    }

    // GDPR Article 21: Right to Object (Marketing opt-out)
    // Users can object to marketing communications and profiling
    pub fn set_marketing_opt_out(&mut self, opt_out: bool) {
        if opt_out && !self.marketing_opt_out {
            // User is opting out
            self.marketing_opt_out = true;
            self.marketing_opt_out_at = Some(Utc::now());
        } else if !opt_out && self.marketing_opt_out {
            // User is opting back in
            self.marketing_opt_out = false;
            // Keep marketing_opt_out_at for audit trail
        }

        self.updated_at = Utc::now();
    }

    // Helper to check if user data processing is allowed
    pub fn can_process_data(&self) -> bool {
        !self.processing_restricted
    }

    // Helper to check if marketing communications are allowed
    pub fn can_send_marketing(&self) -> bool {
        !self.marketing_opt_out
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

    // GDPR Article 16 Tests
    #[test]
    fn test_rectify_data_success() {
        let mut user = User::new(
            "old@example.com".to_string(),
            "hashed_password".to_string(),
            "OldFirst".to_string(),
            "OldLast".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(
            Some("new@example.com".to_string()),
            Some("NewFirst".to_string()),
            Some("NewLast".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.first_name, "NewFirst");
        assert_eq!(user.last_name, "NewLast");
    }

    #[test]
    fn test_rectify_data_partial() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(None, Some("Jane".to_string()), None);

        assert!(result.is_ok());
        assert_eq!(user.email, "test@example.com"); // unchanged
        assert_eq!(user.first_name, "Jane"); // changed
        assert_eq!(user.last_name, "Doe"); // unchanged
    }

    #[test]
    fn test_rectify_data_invalid_email() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(Some("invalid-email".to_string()), None, None);

        assert!(result.is_err());
        assert_eq!(user.email, "test@example.com"); // unchanged on error
    }

    // GDPR Article 18 Tests
    #[test]
    fn test_restrict_processing_success() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        assert!(!user.processing_restricted);
        assert!(user.can_process_data());

        let result = user.restrict_processing();

        assert!(result.is_ok());
        assert!(user.processing_restricted);
        assert!(user.processing_restricted_at.is_some());
        assert!(!user.can_process_data());
    }

    #[test]
    fn test_restrict_processing_already_restricted() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.restrict_processing().unwrap();

        let result = user.restrict_processing();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Processing is already restricted"));
    }

    #[test]
    fn test_unrestrict_processing() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.restrict_processing().unwrap();
        assert!(!user.can_process_data());

        let restriction_timestamp = user.processing_restricted_at;

        user.unrestrict_processing();

        assert!(!user.processing_restricted);
        assert!(user.can_process_data());
        assert_eq!(user.processing_restricted_at, restriction_timestamp); // Audit trail preserved
    }

    // GDPR Article 21 Tests
    #[test]
    fn test_set_marketing_opt_out() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        assert!(!user.marketing_opt_out);
        assert!(user.can_send_marketing());

        user.set_marketing_opt_out(true);

        assert!(user.marketing_opt_out);
        assert!(user.marketing_opt_out_at.is_some());
        assert!(!user.can_send_marketing());
    }

    #[test]
    fn test_set_marketing_opt_in_after_opt_out() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.set_marketing_opt_out(true);
        assert!(!user.can_send_marketing());

        let opt_out_timestamp = user.marketing_opt_out_at;

        user.set_marketing_opt_out(false);

        assert!(!user.marketing_opt_out);
        assert!(user.can_send_marketing());
        assert_eq!(user.marketing_opt_out_at, opt_out_timestamp); // Audit trail preserved
    }

    #[test]
    fn test_gdpr_defaults_on_new_user() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        // GDPR defaults
        assert!(!user.processing_restricted);
        assert!(user.processing_restricted_at.is_none());
        assert!(!user.marketing_opt_out);
        assert!(user.marketing_opt_out_at.is_none());

        // Helper methods
        assert!(user.can_process_data());
        assert!(user.can_send_marketing());
    }
}

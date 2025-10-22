use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Représente un copropriétaire
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Owner {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Owner {
    pub fn new(
        first_name: String,
        last_name: String,
        email: String,
        phone: Option<String>,
        address: String,
        city: String,
        postal_code: String,
        country: String,
    ) -> Result<Self, String> {
        if first_name.is_empty() {
            return Err("First name cannot be empty".to_string());
        }
        if last_name.is_empty() {
            return Err("Last name cannot be empty".to_string());
        }
        if !Self::is_valid_email(&email) {
            return Err("Invalid email format".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email,
            phone,
            address,
            city,
            postal_code,
            country,
            created_at: now,
            updated_at: now,
        })
    }

    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn update_contact(&mut self, email: String, phone: Option<String>) -> Result<(), String> {
        if !Self::is_valid_email(&email) {
            return Err("Invalid email format".to_string());
        }
        self.email = email;
        self.phone = phone;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_owner_success() {
        let owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean.dupont@example.com".to_string(),
            Some("+33612345678".to_string()),
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );

        assert!(owner.is_ok());
        let owner = owner.unwrap();
        assert_eq!(owner.full_name(), "Jean Dupont");
    }

    #[test]
    fn test_create_owner_invalid_email_fails() {
        let owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "invalid-email".to_string(),
            None,
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );

        assert!(owner.is_err());
        assert_eq!(owner.unwrap_err(), "Invalid email format");
    }

    #[test]
    fn test_update_contact() {
        let mut owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean.dupont@example.com".to_string(),
            None,
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        )
        .unwrap();

        let result = owner.update_contact(
            "new.email@example.com".to_string(),
            Some("+33699999999".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(owner.email, "new.email@example.com");
    }
}

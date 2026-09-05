use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a JWT refresh token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RefreshToken {
    /// Create a new refresh token with 7 days expiration
    pub fn new(user_id: Uuid, token: String) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(7);

        Self {
            id: Uuid::new_v4(),
            user_id,
            token,
            expires_at,
            revoked: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.revoked
    }

    /// Revoke the token
    pub fn revoke(&mut self) {
        self.revoked = true;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_refresh_token() {
        let user_id = Uuid::new_v4();
        let token_string = "test_refresh_token".to_string();

        let token = RefreshToken::new(user_id, token_string.clone());

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.token, token_string);
        assert!(!token.revoked);
        assert!(token.is_valid());
    }

    #[test]
    fn test_revoke_token() {
        let user_id = Uuid::new_v4();
        let mut token = RefreshToken::new(user_id, "test_token".to_string());

        assert!(token.is_valid());

        token.revoke();

        assert!(token.revoked);
        assert!(!token.is_valid());
    }

    #[test]
    fn test_expired_token_is_invalid() {
        let user_id = Uuid::new_v4();
        let mut token = RefreshToken::new(user_id, "test_token".to_string());

        // Set expiration to past
        token.expires_at = Utc::now() - Duration::hours(1);

        assert!(token.is_expired());
        assert!(!token.is_valid());
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Two-factor authentication secret for TOTP (Time-based One-Time Password)
/// Stores encrypted TOTP secret and backup codes for account recovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwoFactorSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub secret_encrypted: String, // Base32-encoded TOTP secret (AES-256 encrypted at application level)
    pub backup_codes_encrypted: Vec<String>, // 10 backup codes (bcrypt hashed)
    pub is_enabled: bool,
    pub verified_at: Option<DateTime<Utc>>, // First successful verification
    pub last_used_at: Option<DateTime<Utc>>, // Last successful TOTP verification
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TwoFactorSecret {
    /// Create a new 2FA secret (not yet enabled)
    pub fn new(user_id: Uuid, secret_encrypted: String) -> Result<Self, String> {
        // Validate secret is non-empty
        if secret_encrypted.trim().is_empty() {
            return Err("Secret cannot be empty".to_string());
        }

        // Generate 10 backup codes (will be encrypted by caller)
        let backup_codes = Self::generate_backup_codes_placeholders();

        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            secret_encrypted,
            backup_codes_encrypted: backup_codes,
            is_enabled: false,
            verified_at: None,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Set backup codes (encrypted/hashed by caller)
    pub fn with_backup_codes(
        mut self,
        backup_codes_encrypted: Vec<String>,
    ) -> Result<Self, String> {
        if backup_codes_encrypted.len() != 10 {
            return Err("Must provide exactly 10 backup codes".to_string());
        }

        self.backup_codes_encrypted = backup_codes_encrypted;
        Ok(self)
    }

    /// Enable 2FA after successful verification
    pub fn enable(&mut self) -> Result<(), String> {
        if self.is_enabled {
            return Err("2FA is already enabled".to_string());
        }

        self.is_enabled = true;
        self.verified_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Disable 2FA
    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.updated_at = Utc::now();
    }

    /// Mark TOTP as used (update last_used_at)
    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Regenerate backup codes (caller must hash/encrypt new codes)
    pub fn regenerate_backup_codes(
        &mut self,
        new_backup_codes_encrypted: Vec<String>,
    ) -> Result<(), String> {
        if new_backup_codes_encrypted.len() != 10 {
            return Err("Must provide exactly 10 backup codes".to_string());
        }

        self.backup_codes_encrypted = new_backup_codes_encrypted;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Remove a used backup code (caller must identify which code was used)
    pub fn remove_backup_code(&mut self, code_index: usize) -> Result<(), String> {
        if code_index >= self.backup_codes_encrypted.len() {
            return Err("Invalid backup code index".to_string());
        }

        self.backup_codes_encrypted.remove(code_index);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if backup codes are exhausted (< 3 remaining)
    pub fn backup_codes_low(&self) -> bool {
        self.backup_codes_encrypted.len() < 3
    }

    /// Check if 2FA needs re-verification (not used in 90 days)
    pub fn needs_reverification(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        match self.last_used_at {
            Some(last_used) => {
                let days_since_use = (Utc::now() - last_used).num_days();
                days_since_use > 90
            }
            None => false, // Never used yet
        }
    }

    /// Generate placeholder backup codes (will be replaced with real codes)
    fn generate_backup_codes_placeholders() -> Vec<String> {
        vec!["placeholder".to_string(); 10]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user_id() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    }

    #[test]
    fn test_create_two_factor_secret_success() {
        let secret = TwoFactorSecret::new(
            sample_user_id(),
            "JBSWY3DPEHPK3PXP".to_string(), // Base32 encoded secret
        );

        assert!(secret.is_ok());
        let s = secret.unwrap();
        assert_eq!(s.user_id, sample_user_id());
        assert_eq!(s.secret_encrypted, "JBSWY3DPEHPK3PXP");
        assert!(!s.is_enabled);
        assert!(s.verified_at.is_none());
        assert_eq!(s.backup_codes_encrypted.len(), 10);
    }

    #[test]
    fn test_create_two_factor_secret_empty() {
        let secret = TwoFactorSecret::new(sample_user_id(), "".to_string());

        assert!(secret.is_err());
        assert!(secret.unwrap_err().contains("Secret cannot be empty"));
    }

    #[test]
    fn test_with_backup_codes() {
        let codes: Vec<String> = (0..10).map(|i| format!("CODE{}", i)).collect();

        let secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string())
            .unwrap()
            .with_backup_codes(codes.clone());

        assert!(secret.is_ok());
        let s = secret.unwrap();
        assert_eq!(s.backup_codes_encrypted.len(), 10);
        assert_eq!(s.backup_codes_encrypted[0], "CODE0");
    }

    #[test]
    fn test_with_backup_codes_invalid_count() {
        let codes: Vec<String> = vec!["CODE1".to_string(), "CODE2".to_string()]; // Only 2 codes

        let secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string())
            .unwrap()
            .with_backup_codes(codes);

        assert!(secret.is_err());
        assert!(secret
            .unwrap_err()
            .contains("Must provide exactly 10 backup codes"));
    }

    #[test]
    fn test_enable_2fa() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        assert!(!secret.is_enabled);
        assert!(secret.verified_at.is_none());

        let result = secret.enable();
        assert!(result.is_ok());
        assert!(secret.is_enabled);
        assert!(secret.verified_at.is_some());
        assert!(secret.verified_at.unwrap() <= Utc::now());
    }

    #[test]
    fn test_enable_2fa_already_enabled() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        secret.enable().unwrap();

        let result = secret.enable();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already enabled"));
    }

    #[test]
    fn test_disable_2fa() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        secret.enable().unwrap();
        assert!(secret.is_enabled);

        secret.disable();
        assert!(!secret.is_enabled);
    }

    #[test]
    fn test_mark_used() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        assert!(secret.last_used_at.is_none());

        secret.mark_used();
        assert!(secret.last_used_at.is_some());
        assert!(secret.last_used_at.unwrap() <= Utc::now());
    }

    #[test]
    fn test_regenerate_backup_codes() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        let new_codes: Vec<String> = (0..10).map(|i| format!("NEWCODE{}", i)).collect();

        let result = secret.regenerate_backup_codes(new_codes.clone());
        assert!(result.is_ok());
        assert_eq!(secret.backup_codes_encrypted.len(), 10);
        assert_eq!(secret.backup_codes_encrypted[0], "NEWCODE0");
    }

    #[test]
    fn test_regenerate_backup_codes_invalid_count() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        let new_codes: Vec<String> = vec!["CODE1".to_string()]; // Only 1 code

        let result = secret.regenerate_backup_codes(new_codes);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_backup_code() {
        let codes: Vec<String> = (0..10).map(|i| format!("CODE{}", i)).collect();
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string())
            .unwrap()
            .with_backup_codes(codes)
            .unwrap();

        assert_eq!(secret.backup_codes_encrypted.len(), 10);

        let result = secret.remove_backup_code(0);
        assert!(result.is_ok());
        assert_eq!(secret.backup_codes_encrypted.len(), 9);
        assert_eq!(secret.backup_codes_encrypted[0], "CODE1"); // CODE0 removed, CODE1 is now first
    }

    #[test]
    fn test_remove_backup_code_invalid_index() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        let result = secret.remove_backup_code(20); // Invalid index
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid backup code index"));
    }

    #[test]
    fn test_backup_codes_low() {
        let codes: Vec<String> = (0..10).map(|i| format!("CODE{}", i)).collect();
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string())
            .unwrap()
            .with_backup_codes(codes)
            .unwrap();

        assert!(!secret.backup_codes_low());

        // Remove 8 codes (leaving 2)
        for _ in 0..8 {
            secret.remove_backup_code(0).unwrap();
        }

        assert_eq!(secret.backup_codes_encrypted.len(), 2);
        assert!(secret.backup_codes_low());
    }

    #[test]
    fn test_needs_reverification() {
        let mut secret = TwoFactorSecret::new(sample_user_id(), "SECRET123".to_string()).unwrap();

        // Not enabled
        assert!(!secret.needs_reverification());

        // Enable
        secret.enable().unwrap();
        assert!(!secret.needs_reverification()); // Never used yet

        // Mark as used recently
        secret.mark_used();
        assert!(!secret.needs_reverification());

        // Simulate 91 days ago (needs reverification)
        secret.last_used_at = Some(Utc::now() - chrono::Duration::days(91));
        assert!(secret.needs_reverification());
    }
}

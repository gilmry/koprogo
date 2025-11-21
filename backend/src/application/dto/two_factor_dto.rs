use crate::domain::entities::TwoFactorSecret;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ========================================
// 2FA Setup DTOs
// ========================================

/// DTO for initiating 2FA setup (returns QR code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup2FAResponseDto {
    pub secret: String, // Base32-encoded secret (ONLY returned during setup, never again)
    pub qr_code_data_url: String, // Data URL for QR code image (data:image/png;base64,...)
    pub backup_codes: Vec<String>, // 10 plaintext backup codes (ONLY shown once, user must save)
    pub issuer: String, // "KoproGo"
    pub account_name: String, // User email or username
}

/// DTO for enabling 2FA (requires TOTP verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enable2FADto {
    pub totp_code: String, // 6-digit TOTP code from authenticator app
}

/// DTO for 2FA enable response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enable2FAResponseDto {
    pub success: bool,
    pub message: String,
    pub enabled_at: DateTime<Utc>,
}

// ========================================
// 2FA Verification DTOs
// ========================================

/// DTO for verifying TOTP code during login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verify2FADto {
    pub totp_code: String, // 6-digit TOTP code OR 8-character backup code
}

/// DTO for 2FA verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verify2FAResponseDto {
    pub success: bool,
    pub message: String,
    pub backup_code_used: bool, // True if backup code was used instead of TOTP
    pub backup_codes_remaining: Option<usize>, // Number of backup codes remaining (if backup code used)
}

// ========================================
// 2FA Management DTOs
// ========================================

/// DTO for disabling 2FA (requires current password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disable2FADto {
    pub current_password: String, // Require password to disable 2FA
}

/// DTO for regenerating backup codes (requires TOTP verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateBackupCodesDto {
    pub totp_code: String, // Must verify with TOTP before regenerating
}

/// DTO for regenerate backup codes response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateBackupCodesResponseDto {
    pub backup_codes: Vec<String>, // 10 new plaintext backup codes
    pub regenerated_at: DateTime<Utc>,
}

/// DTO for 2FA status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorStatusDto {
    pub is_enabled: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub backup_codes_remaining: usize,
    pub backup_codes_low: bool, // True if < 3 codes remaining
    pub needs_reverification: bool, // True if not used in 90 days
}

impl From<TwoFactorSecret> for TwoFactorStatusDto {
    fn from(secret: TwoFactorSecret) -> Self {
        Self {
            is_enabled: secret.is_enabled,
            verified_at: secret.verified_at,
            last_used_at: secret.last_used_at,
            backup_codes_remaining: secret.backup_codes_encrypted.len(),
            backup_codes_low: secret.backup_codes_low(),
            needs_reverification: secret.needs_reverification(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_two_factor_status_dto_from_entity() {
        let mut secret = TwoFactorSecret::new(
            Uuid::new_v4(),
            "JBSWY3DPEHPK3PXP".to_string(),
        )
        .unwrap();

        secret.enable().unwrap();

        let dto: TwoFactorStatusDto = secret.into();

        assert!(dto.is_enabled);
        assert!(dto.verified_at.is_some());
        assert_eq!(dto.backup_codes_remaining, 10);
        assert!(!dto.backup_codes_low);
        assert!(!dto.needs_reverification);
    }

    #[test]
    fn test_two_factor_status_dto_backup_codes_low() {
        let codes: Vec<String> = vec!["CODE1".to_string(), "CODE2".to_string()]; // Only 2 codes
        let mut secret = TwoFactorSecret::new(
            Uuid::new_v4(),
            "JBSWY3DPEHPK3PXP".to_string(),
        )
        .unwrap();

        // Remove 8 codes to leave 2
        for _ in 0..8 {
            secret.remove_backup_code(0).unwrap();
        }

        let dto: TwoFactorStatusDto = secret.into();

        assert_eq!(dto.backup_codes_remaining, 2);
        assert!(dto.backup_codes_low);
    }
}

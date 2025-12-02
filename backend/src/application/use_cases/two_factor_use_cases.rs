use crate::application::dto::{
    Disable2FADto, Enable2FADto, Enable2FAResponseDto, RegenerateBackupCodesDto,
    RegenerateBackupCodesResponseDto, Setup2FAResponseDto, TwoFactorStatusDto, Verify2FADto,
    Verify2FAResponseDto,
};
use crate::application::ports::{TwoFactorRepository, UserRepository};
use crate::domain::entities::TwoFactorSecret;
use crate::infrastructure::audit::{log_audit_event, AuditEventType};
use crate::infrastructure::totp::TotpGenerator;
use std::sync::Arc;
use uuid::Uuid;

/// Use cases for two-factor authentication (TOTP)
pub struct TwoFactorUseCases {
    two_factor_repo: Arc<dyn TwoFactorRepository>,
    user_repo: Arc<dyn UserRepository>,
    encryption_key: [u8; 32],
}

impl TwoFactorUseCases {
    pub fn new(
        two_factor_repo: Arc<dyn TwoFactorRepository>,
        user_repo: Arc<dyn UserRepository>,
        encryption_key: [u8; 32],
    ) -> Self {
        Self {
            two_factor_repo,
            user_repo,
            encryption_key,
        }
    }

    /// Setup 2FA for a user (returns QR code + backup codes)
    /// This does NOT enable 2FA yet - user must verify with TOTP code first
    pub async fn setup_2fa(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Setup2FAResponseDto, String> {
        // Check if user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or("User not found")?;

        // Check if 2FA already exists for this user
        if let Some(existing) = self.two_factor_repo.find_by_user_id(user_id).await? {
            if existing.is_enabled {
                return Err("2FA is already enabled for this user".to_string());
            }
            // Delete existing setup if not enabled (allow re-setup)
            self.two_factor_repo.delete(user_id).await?;
        }

        // Generate TOTP secret
        let secret = Self::generate_totp_secret();
        let secret_encrypted = self.encrypt_secret(&secret)?;

        // Generate 10 backup codes
        let backup_codes = Self::generate_backup_codes();
        let backup_codes_encrypted: Vec<String> = backup_codes
            .iter()
            .map(|code| Self::hash_backup_code(code))
            .collect::<Result<Vec<_>, _>>()?;

        // Create TwoFactorSecret entity
        let two_factor_secret = TwoFactorSecret::new(user_id, secret_encrypted)?
            .with_backup_codes(backup_codes_encrypted)?;

        // Save to database
        self.two_factor_repo.create(&two_factor_secret).await?;

        // Generate QR code
        let issuer = "KoproGo".to_string();
        let account_name = user.email.clone();
        let qr_code_data_url = Self::generate_qr_code(&secret, &issuer, &account_name)?;

        // Audit log
        log_audit_event(
            AuditEventType::TwoFactorSetupInitiated,
            Some(user_id),
            Some(organization_id),
            Some(format!("User {} initiated 2FA setup", user.email)),
            None,
        )
        .await;

        Ok(Setup2FAResponseDto {
            secret: secret.clone(), // ONLY returned during setup (never again)
            qr_code_data_url,
            backup_codes: backup_codes.clone(), // ONLY shown once (user must save)
            issuer,
            account_name,
        })
    }

    /// Enable 2FA after user verifies TOTP code
    pub async fn enable_2fa(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: Enable2FADto,
    ) -> Result<Enable2FAResponseDto, String> {
        // Find 2FA secret
        let mut secret = self
            .two_factor_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or("2FA setup not found. Please run setup first.")?;

        if secret.is_enabled {
            return Err("2FA is already enabled".to_string());
        }

        // Decrypt secret
        let decrypted_secret = self.decrypt_secret(&secret.secret_encrypted)?;

        // Verify TOTP code
        if !Self::verify_totp_code(&decrypted_secret, &dto.totp_code)? {
            // Audit log failed verification
            log_audit_event(
                AuditEventType::TwoFactorVerificationFailed,
                Some(user_id),
                Some(organization_id),
                Some("Failed TOTP verification during enable".to_string()),
                None,
            )
            .await;

            return Err("Invalid TOTP code".to_string());
        }

        // Enable 2FA
        secret.enable()?;
        secret.mark_used(); // Mark as used immediately

        // Update database
        self.two_factor_repo.update(&secret).await?;

        // Audit log successful enable
        log_audit_event(
            AuditEventType::TwoFactorEnabled,
            Some(user_id),
            Some(organization_id),
            Some("2FA successfully enabled".to_string()),
            None,
        )
        .await;

        Ok(Enable2FAResponseDto {
            success: true,
            message: "2FA successfully enabled".to_string(),
            enabled_at: secret.verified_at.unwrap(),
        })
    }

    /// Verify TOTP code or backup code during login
    pub async fn verify_2fa(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: Verify2FADto,
    ) -> Result<Verify2FAResponseDto, String> {
        // Find 2FA secret
        let mut secret = self
            .two_factor_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or("2FA not enabled for this user")?;

        if !secret.is_enabled {
            return Err("2FA is not enabled".to_string());
        }

        // Decrypt secret
        let decrypted_secret = self.decrypt_secret(&secret.secret_encrypted)?;

        // Try TOTP code first
        if Self::verify_totp_code(&decrypted_secret, &dto.totp_code)? {
            // TOTP verification successful
            secret.mark_used();
            self.two_factor_repo.update(&secret).await?;

            // Audit log
            log_audit_event(
                AuditEventType::TwoFactorVerified,
                Some(user_id),
                Some(organization_id),
                Some("TOTP verification successful".to_string()),
                None,
            )
            .await;

            return Ok(Verify2FAResponseDto {
                success: true,
                message: "2FA verification successful".to_string(),
                backup_code_used: false,
                backup_codes_remaining: None,
            });
        }

        // TOTP failed, try backup codes
        if let Some(code_index) = Self::find_matching_backup_code(&secret, &dto.totp_code)? {
            // Backup code matched
            secret.remove_backup_code(code_index)?;
            secret.mark_used();
            self.two_factor_repo.update(&secret).await?;

            let backup_codes_remaining = secret.backup_codes_encrypted.len();

            // Audit log
            log_audit_event(
                AuditEventType::BackupCodeUsed,
                Some(user_id),
                Some(organization_id),
                Some(format!(
                    "Backup code used. {} codes remaining",
                    backup_codes_remaining
                )),
                None,
            )
            .await;

            // Warn if backup codes are low
            if secret.backup_codes_low() {
                log_audit_event(
                    AuditEventType::TwoFactorReverificationRequired,
                    Some(user_id),
                    Some(organization_id),
                    Some(format!(
                        "Warning: Only {} backup codes remaining",
                        backup_codes_remaining
                    )),
                    None,
                )
                .await;
            }

            return Ok(Verify2FAResponseDto {
                success: true,
                message: "Backup code verification successful".to_string(),
                backup_code_used: true,
                backup_codes_remaining: Some(backup_codes_remaining),
            });
        }

        // Both TOTP and backup code failed
        log_audit_event(
            AuditEventType::TwoFactorVerificationFailed,
            Some(user_id),
            Some(organization_id),
            Some("Invalid TOTP code and no matching backup code".to_string()),
            None,
        )
        .await;

        Err("Invalid TOTP code or backup code".to_string())
    }

    /// Disable 2FA (requires current password verification)
    pub async fn disable_2fa(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: Disable2FADto,
    ) -> Result<(), String> {
        // Verify user password first (implementation depends on password hashing)
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or("User not found")?;

        if !Self::verify_password(&user.password_hash, &dto.current_password)? {
            return Err("Invalid password".to_string());
        }

        // Delete 2FA configuration
        self.two_factor_repo.delete(user_id).await?;

        // Audit log
        log_audit_event(
            AuditEventType::TwoFactorDisabled,
            Some(user_id),
            Some(organization_id),
            Some("2FA disabled by user".to_string()),
            None,
        )
        .await;

        Ok(())
    }

    /// Regenerate backup codes (requires TOTP verification)
    pub async fn regenerate_backup_codes(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: RegenerateBackupCodesDto,
    ) -> Result<RegenerateBackupCodesResponseDto, String> {
        // Find 2FA secret
        let mut secret = self
            .two_factor_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or("2FA not enabled")?;

        if !secret.is_enabled {
            return Err("2FA is not enabled".to_string());
        }

        // Verify TOTP code
        let decrypted_secret = self.decrypt_secret(&secret.secret_encrypted)?;
        if !Self::verify_totp_code(&decrypted_secret, &dto.totp_code)? {
            return Err("Invalid TOTP code".to_string());
        }

        // Generate new backup codes
        let backup_codes = Self::generate_backup_codes();
        let backup_codes_encrypted: Vec<String> = backup_codes
            .iter()
            .map(|code| Self::hash_backup_code(code))
            .collect::<Result<Vec<_>, _>>()?;

        // Update secret
        secret.regenerate_backup_codes(backup_codes_encrypted)?;
        self.two_factor_repo.update(&secret).await?;

        // Audit log
        log_audit_event(
            AuditEventType::BackupCodesRegenerated,
            Some(user_id),
            Some(organization_id),
            Some("Backup codes regenerated".to_string()),
            None,
        )
        .await;

        Ok(RegenerateBackupCodesResponseDto {
            backup_codes: backup_codes.clone(),
            regenerated_at: chrono::Utc::now(),
        })
    }

    /// Get 2FA status for a user
    pub async fn get_2fa_status(&self, user_id: Uuid) -> Result<TwoFactorStatusDto, String> {
        match self.two_factor_repo.find_by_user_id(user_id).await? {
            Some(secret) => Ok(secret.into()),
            None => Ok(TwoFactorStatusDto {
                is_enabled: false,
                verified_at: None,
                last_used_at: None,
                backup_codes_remaining: 0,
                backup_codes_low: false,
                needs_reverification: false,
            }),
        }
    }

    // ========================================
    // Private helper methods
    // ========================================

    /// Generate TOTP secret (Base32 encoded)
    fn generate_totp_secret() -> String {
        TotpGenerator::generate_secret()
    }

    /// Encrypt TOTP secret (AES-256-GCM)
    fn encrypt_secret(&self, secret: &str) -> Result<String, String> {
        TotpGenerator::encrypt_secret(secret, &self.encryption_key)
    }

    /// Decrypt TOTP secret (AES-256-GCM)
    fn decrypt_secret(&self, encrypted: &str) -> Result<String, String> {
        TotpGenerator::decrypt_secret(encrypted, &self.encryption_key)
    }

    /// Generate QR code data URL
    fn generate_qr_code(secret: &str, issuer: &str, account_name: &str) -> Result<String, String> {
        TotpGenerator::generate_qr_code(secret, issuer, account_name)
    }

    /// Verify TOTP code (6 digits)
    fn verify_totp_code(secret: &str, code: &str) -> Result<bool, String> {
        TotpGenerator::verify_code(secret, code)
    }

    /// Generate 10 backup codes (8-char alphanumeric, uppercase)
    fn generate_backup_codes() -> Vec<String> {
        TotpGenerator::generate_backup_codes()
    }

    /// Hash backup code (bcrypt)
    fn hash_backup_code(code: &str) -> Result<String, String> {
        TotpGenerator::hash_backup_code(code)
    }

    /// Find matching backup code in array
    fn find_matching_backup_code(
        secret: &TwoFactorSecret,
        code: &str,
    ) -> Result<Option<usize>, String> {
        for (index, stored_hash) in secret.backup_codes_encrypted.iter().enumerate() {
            if TotpGenerator::verify_backup_code(code, stored_hash)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// Verify user password (bcrypt)
    fn verify_password(password_hash: &str, password: &str) -> Result<bool, String> {
        bcrypt::verify(password, password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_backup_codes() {
        let codes = TwoFactorUseCases::generate_backup_codes();

        assert_eq!(codes.len(), 10);

        // All codes should be unique
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), 10);
    }

    #[test]
    fn test_verify_totp_code_invalid_format() {
        let result = TwoFactorUseCases::verify_totp_code("SECRET", "12345"); // 5 digits
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result2 = TwoFactorUseCases::verify_totp_code("SECRET", "1234567"); // 7 digits
        assert!(result2.is_ok());
        assert!(!result2.unwrap());

        let result3 = TwoFactorUseCases::verify_totp_code("SECRET", "ABCDEF"); // Non-digits
        assert!(result3.is_ok());
        assert!(!result3.unwrap());
    }
}

-- Two-Factor Authentication (2FA) TOTP Secrets - Issue #78 (Security Hardening)
-- Stores encrypted TOTP secrets and backup codes for account recovery

-- ========================================
-- Table: two_factor_secrets
-- ========================================
CREATE TABLE two_factor_secrets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secret_encrypted TEXT NOT NULL, -- Base32-encoded TOTP secret (AES-256 encrypted at application level)
    backup_codes_encrypted TEXT[] NOT NULL DEFAULT '{}', -- Array of 10 backup codes (bcrypt hashed)
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    verified_at TIMESTAMPTZ, -- First successful TOTP verification
    last_used_at TIMESTAMPTZ, -- Last successful TOTP/backup code use
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for two_factor_secrets
CREATE UNIQUE INDEX idx_two_factor_secrets_user ON two_factor_secrets(user_id); -- One 2FA config per user
CREATE INDEX idx_two_factor_secrets_enabled ON two_factor_secrets(is_enabled) WHERE is_enabled = true;
-- Note: Reverification index removed (NOW() is not immutable for partial index predicates)
-- Application layer will filter for reverification queries using last_used_at column
CREATE INDEX idx_two_factor_secrets_last_used ON two_factor_secrets(last_used_at) WHERE is_enabled = true;

-- ========================================
-- Triggers
-- ========================================

-- Update two_factor_secrets.updated_at on modification
CREATE OR REPLACE FUNCTION update_two_factor_secret_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_two_factor_secrets_updated
    BEFORE UPDATE ON two_factor_secrets
    FOR EACH ROW
    EXECUTE FUNCTION update_two_factor_secret_timestamp();

-- ========================================
-- Comments for Documentation
-- ========================================

COMMENT ON TABLE two_factor_secrets IS 'Two-factor authentication TOTP secrets and backup codes';
COMMENT ON COLUMN two_factor_secrets.secret_encrypted IS 'Base32-encoded TOTP secret (AES-256 encrypted at application level)';
COMMENT ON COLUMN two_factor_secrets.backup_codes_encrypted IS 'Array of 10 bcrypt-hashed backup codes for account recovery';
COMMENT ON COLUMN two_factor_secrets.is_enabled IS 'Whether 2FA is enabled for this user (requires first successful verification)';
COMMENT ON COLUMN two_factor_secrets.verified_at IS 'Timestamp of first successful TOTP verification (enables 2FA)';
COMMENT ON COLUMN two_factor_secrets.last_used_at IS 'Last successful TOTP or backup code verification (for reverification tracking)';

-- ========================================
-- Security Notes
-- ========================================
-- 1. TOTP Secret Storage:
--    - Secret is Base32-encoded (standard TOTP format)
--    - Encrypted at application level with AES-256 (separate encryption key in environment)
--    - Never logged or exposed in API responses (except during initial setup QR code)
--
-- 2. Backup Codes:
--    - 10 codes generated during 2FA setup
--    - Each code is 8 characters (alphanumeric, uppercase)
--    - Bcrypt hashed (cost factor 12) for secure storage
--    - Used for account recovery if TOTP device is lost
--    - Removed from array after use (one-time use only)
--
-- 3. Rate Limiting:
--    - Login endpoint already has rate limiting (5 attempts per 15 minutes)
--    - Additional 2FA verification rate limiting: 3 attempts per 5 minutes per user
--    - Prevents brute-force attacks on TOTP codes
--
-- 4. Reverification:
--    - If 2FA not used for 90 days, require reverification
--    - Prevents stale 2FA configurations from being exploited
--
-- 5. Audit Logging:
--    - All 2FA events logged to audit_logs table:
--      - TwoFactorEnabled, TwoFactorDisabled
--      - TwoFactorVerified, TwoFactorVerificationFailed
--      - BackupCodeUsed, BackupCodesRegenerated
--
-- 6. GDPR Compliance:
--    - 2FA secrets cascade delete when user is anonymized (ON DELETE CASCADE)
--    - Backup codes are hashed (not reversible)
--    - Last usage timestamp tracked for compliance reporting

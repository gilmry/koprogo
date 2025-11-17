-- Add GDPR Complementary Articles fields to users table
-- Issue #90: GDPR Articles 16 (Rectification), 18 (Restriction), 21 (Objection)

-- Article 18: Right to Restriction of Processing
-- Users can request to limit data processing temporarily
ALTER TABLE users
ADD COLUMN processing_restricted BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN processing_restricted_at TIMESTAMPTZ;

-- Article 21: Right to Object (Marketing opt-out)
-- Users can object to marketing/profiling communications
ALTER TABLE users
ADD COLUMN marketing_opt_out BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN marketing_opt_out_at TIMESTAMPTZ;

-- Create index for querying users with processing restrictions (admin views)
CREATE INDEX idx_users_processing_restricted
ON users(processing_restricted)
WHERE processing_restricted = TRUE;

-- Create index for querying users who opted out of marketing
CREATE INDEX idx_users_marketing_opt_out
ON users(marketing_opt_out)
WHERE marketing_opt_out = TRUE;

-- Comments for documentation
COMMENT ON COLUMN users.processing_restricted IS 'GDPR Article 18: User requested restriction of data processing';
COMMENT ON COLUMN users.processing_restricted_at IS 'GDPR Article 18: Timestamp when processing restriction was requested';
COMMENT ON COLUMN users.marketing_opt_out IS 'GDPR Article 21: User objected to marketing communications';
COMMENT ON COLUMN users.marketing_opt_out_at IS 'GDPR Article 21: Timestamp when user opted out of marketing';

-- Note: Article 16 (Rectification) does not require new columns
-- It uses existing user fields (email, first_name, last_name) with update_profile() method
-- All rectifications are logged in audit_logs table

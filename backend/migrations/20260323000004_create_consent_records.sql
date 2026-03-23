-- Create consent_records table for GDPR Art. 13-14 - Privacy Policy compliance
-- Tracks user consent to privacy policy and terms of service
-- Issue #315

CREATE TABLE IF NOT EXISTS consent_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    consent_type VARCHAR(50) NOT NULL CHECK (consent_type IN ('privacy_policy', 'terms')),
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    policy_version VARCHAR(20) NOT NULL DEFAULT '1.0',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on user_id for finding user consents
CREATE INDEX idx_consent_records_user_id ON consent_records(user_id);

-- Index on organization_id for finding organization consents
CREATE INDEX idx_consent_records_organization_id ON consent_records(organization_id);

-- Index on consent_type for filtering by type
CREATE INDEX idx_consent_records_type ON consent_records(consent_type);

-- Composite index for finding latest consent per user per type
CREATE INDEX idx_consent_records_user_type_latest ON consent_records(user_id, consent_type, accepted_at DESC);

-- Add comment for documentation
COMMENT ON TABLE consent_records IS 'GDPR Art. 13-14 compliance: Records user consent to privacy policy and terms of service with audit trail (IP address, user agent, timestamp)';
COMMENT ON COLUMN consent_records.consent_type IS 'Type of consent: privacy_policy (GDPR Art. 13-14) or terms (contractual terms)';
COMMENT ON COLUMN consent_records.policy_version IS 'Version of the policy accepted (e.g., 1.0, 1.1) to track policy updates';

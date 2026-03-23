-- Issue #280: Energy group buying extensions
-- Support for non-copropriétaire individuals joining energy campaigns (Art. 22 RED II)

-- Add audience_type to energy_campaigns
ALTER TABLE energy_campaigns
    ADD COLUMN IF NOT EXISTS audience_type VARCHAR(30) NOT NULL DEFAULT 'CoProprietiesOnly'
    CONSTRAINT chk_audience_type CHECK (audience_type IN ('CoProprietiesOnly', 'OpenToIndividuals', 'Public'));

-- IndividualMember table for non-copropriétaire participants
CREATE TABLE IF NOT EXISTS individual_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES energy_campaigns(id) ON DELETE CASCADE,
    email VARCHAR(200) NOT NULL,
    postal_code VARCHAR(10) NOT NULL,
    has_gdpr_consent BOOLEAN NOT NULL DEFAULT FALSE,
    consent_at TIMESTAMPTZ,
    annual_consumption_kwh NUMERIC(12,2),
    current_provider VARCHAR(100),
    ean_code VARCHAR(30),
    unsubscribed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(campaign_id, email)
);

-- Indexes for individual_members
CREATE INDEX IF NOT EXISTS idx_individual_members_campaign ON individual_members(campaign_id);
CREATE INDEX IF NOT EXISTS idx_individual_members_email ON individual_members(email);
CREATE INDEX IF NOT EXISTS idx_individual_members_postal ON individual_members(postal_code);
CREATE INDEX IF NOT EXISTS idx_individual_members_unsubscribed ON individual_members(unsubscribed_at);
CREATE INDEX IF NOT EXISTS idx_individual_members_consent ON individual_members(campaign_id, has_gdpr_consent) WHERE has_gdpr_consent = TRUE;

-- Comments for documentation
COMMENT ON TABLE individual_members IS 'Non-copropriétaire members joining energy campaigns — Issue #280 Art. 22 RED II (Renewable Energy Directive II)';
COMMENT ON COLUMN individual_members.has_gdpr_consent IS 'GDPR consent for campaign participation and data processing';
COMMENT ON COLUMN individual_members.consent_at IS 'Timestamp when member granted consent';
COMMENT ON COLUMN individual_members.annual_consumption_kwh IS 'Annual energy consumption in kWh (from uploaded bills)';
COMMENT ON COLUMN individual_members.ean_code IS 'Belgian EAN identifier for electricity meter';
COMMENT ON COLUMN individual_members.unsubscribed_at IS 'When member withdrew from campaign (GDPR right to erasure)';
COMMENT ON COLUMN energy_campaigns.audience_type IS 'Audience eligibility: CoProprietiesOnly / OpenToIndividuals / Public — Issue #280';

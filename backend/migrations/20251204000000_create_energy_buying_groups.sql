-- Create Energy Buying Groups Tables (Issue #ENERGY - Achats Groupés d'Énergie)
--
-- This migration creates tables for the energy buying groups feature,
-- enabling condominiums to collectively negotiate energy contracts
-- while maintaining GDPR compliance through encryption and anonymization.
--
-- Key features:
-- - Energy campaigns (buying groups)
-- - Encrypted energy bill uploads with GDPR consent
-- - Provider offers comparison
-- - Aggregated building-level statistics (k-anonymity)
-- - Auto-deletion based on retention policies

-- ============================================================================
-- Energy Campaigns Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS energy_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID REFERENCES buildings(id) ON DELETE CASCADE,

    -- Méta
    campaign_name VARCHAR(255) NOT NULL,
    campaign_type VARCHAR(50) NOT NULL CHECK (campaign_type IN ('BuyingGroup', 'CollectiveSwitch')),
    status VARCHAR(50) NOT NULL DEFAULT 'Draft' CHECK (status IN (
        'Draft',
        'AwaitingAGVote',
        'CollectingData',
        'Negotiating',
        'AwaitingFinalVote',
        'Finalized',
        'Completed',
        'Cancelled'
    )),

    -- Timeline
    deadline_participation TIMESTAMPTZ NOT NULL,
    deadline_vote TIMESTAMPTZ,
    contract_start_date TIMESTAMPTZ,

    -- Configuration
    energy_types TEXT[] NOT NULL, -- ['Electricity', 'Gas', 'Both']
    contract_duration_months INTEGER NOT NULL DEFAULT 12 CHECK (contract_duration_months > 0),
    contract_type VARCHAR(50) NOT NULL DEFAULT 'Fixed' CHECK (contract_type IN ('Fixed', 'Variable')),

    -- Agrégation (données anonymes)
    total_participants INTEGER NOT NULL DEFAULT 0 CHECK (total_participants >= 0),
    total_kwh_electricity DOUBLE PRECISION CHECK (total_kwh_electricity >= 0),
    total_kwh_gas DOUBLE PRECISION CHECK (total_kwh_gas >= 0),
    avg_kwh_per_unit DOUBLE PRECISION CHECK (avg_kwh_per_unit >= 0),

    -- Résultats négociation
    selected_offer_id UUID, -- Foreign key to provider_offers
    estimated_savings_pct DOUBLE PRECISION CHECK (estimated_savings_pct >= 0 AND estimated_savings_pct <= 100),

    -- Audit
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT deadline_before_contract CHECK (deadline_participation < COALESCE(contract_start_date, deadline_participation + INTERVAL '1 year')),
    CONSTRAINT at_least_one_energy_type CHECK (cardinality(energy_types) > 0)
);

-- Indexes for energy_campaigns
CREATE INDEX idx_energy_campaigns_organization ON energy_campaigns(organization_id);
CREATE INDEX idx_energy_campaigns_building ON energy_campaigns(building_id);
CREATE INDEX idx_energy_campaigns_status ON energy_campaigns(status);
CREATE INDEX idx_energy_campaigns_deadline ON energy_campaigns(deadline_participation);

-- ============================================================================
-- Provider Offers Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS provider_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES energy_campaigns(id) ON DELETE CASCADE,

    -- Provider details
    provider_name VARCHAR(255) NOT NULL,
    price_kwh_electricity DOUBLE PRECISION CHECK (price_kwh_electricity >= 0),
    price_kwh_gas DOUBLE PRECISION CHECK (price_kwh_gas >= 0),
    fixed_monthly_fee DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (fixed_monthly_fee >= 0),

    -- Green energy percentage (0-100)
    green_energy_pct DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (green_energy_pct >= 0 AND green_energy_pct <= 100),

    -- Contract details
    contract_duration_months INTEGER NOT NULL CHECK (contract_duration_months > 0),
    estimated_savings_pct DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (estimated_savings_pct >= 0),

    -- Validity
    offer_valid_until TIMESTAMPTZ NOT NULL,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT at_least_one_price CHECK (
        price_kwh_electricity IS NOT NULL OR price_kwh_gas IS NOT NULL
    )
);

-- Indexes for provider_offers
CREATE INDEX idx_provider_offers_campaign ON provider_offers(campaign_id);
CREATE INDEX idx_provider_offers_valid_until ON provider_offers(offer_valid_until);
CREATE INDEX idx_provider_offers_green_pct ON provider_offers(green_energy_pct DESC);

-- Add foreign key from energy_campaigns to provider_offers
ALTER TABLE energy_campaigns ADD CONSTRAINT fk_selected_offer
    FOREIGN KEY (selected_offer_id) REFERENCES provider_offers(id) ON DELETE SET NULL;

-- ============================================================================
-- Energy Bill Uploads Table (GDPR-compliant with encryption)
-- ============================================================================

CREATE TABLE IF NOT EXISTS energy_bill_uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES energy_campaigns(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Données extraites (chiffrées avec AES-256-GCM)
    bill_period_start TIMESTAMPTZ NOT NULL,
    bill_period_end TIMESTAMPTZ NOT NULL,
    total_kwh_encrypted BYTEA NOT NULL, -- Encrypted consumption data
    energy_type VARCHAR(50) NOT NULL CHECK (energy_type IN ('Electricity', 'Gas', 'Both')),
    provider VARCHAR(255),
    postal_code VARCHAR(4) NOT NULL, -- Belgian postal code (4 digits)

    -- Authentification facture (SHA-256 hash)
    file_hash VARCHAR(64) NOT NULL, -- SHA-256 produces 64 hex chars
    file_path_encrypted TEXT NOT NULL, -- S3 path (encrypted)
    ocr_confidence DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (ocr_confidence >= 0 AND ocr_confidence <= 100),
    manually_verified BOOLEAN NOT NULL DEFAULT FALSE,

    -- Upload metadata
    uploaded_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ,
    verified_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- GDPR Consent (Article 6.1.a - Consentement explicite)
    consent_timestamp TIMESTAMPTZ NOT NULL,
    consent_ip VARCHAR(45) NOT NULL, -- IPv6 max length
    consent_user_agent TEXT NOT NULL,
    consent_signature_hash VARCHAR(32) NOT NULL, -- MD5 hash (32 hex chars)

    -- Privacy & Retention (GDPR Article 5.1.e)
    anonymized BOOLEAN NOT NULL DEFAULT FALSE,
    retention_until TIMESTAMPTZ NOT NULL, -- Auto-delete after this date
    deleted_at TIMESTAMPTZ, -- Soft delete for GDPR compliance

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT bill_period_valid CHECK (bill_period_start < bill_period_end),
    CONSTRAINT postal_code_length CHECK (char_length(postal_code) = 4),
    CONSTRAINT one_upload_per_campaign_unit UNIQUE (campaign_id, unit_id)
);

-- Indexes for energy_bill_uploads
CREATE INDEX idx_energy_bill_uploads_campaign ON energy_bill_uploads(campaign_id);
CREATE INDEX idx_energy_bill_uploads_unit ON energy_bill_uploads(unit_id);
CREATE INDEX idx_energy_bill_uploads_building ON energy_bill_uploads(building_id);
CREATE INDEX idx_energy_bill_uploads_organization ON energy_bill_uploads(organization_id);
CREATE INDEX idx_energy_bill_uploads_verified ON energy_bill_uploads(manually_verified);
CREATE INDEX idx_energy_bill_uploads_anonymized ON energy_bill_uploads(anonymized);
CREATE INDEX idx_energy_bill_uploads_retention ON energy_bill_uploads(retention_until);
CREATE INDEX idx_energy_bill_uploads_deleted ON energy_bill_uploads(deleted_at);

-- ============================================================================
-- Triggers for Aggregation (Building-level anonymization)
-- ============================================================================

-- Trigger function to aggregate building energy consumption
CREATE OR REPLACE FUNCTION aggregate_building_energy()
RETURNS TRIGGER AS $$
DECLARE
    v_total_participants INTEGER;
    v_total_kwh_electricity DOUBLE PRECISION;
    v_total_kwh_gas DOUBLE PRECISION;
    v_avg_kwh DOUBLE PRECISION;
BEGIN
    -- Only aggregate if bill is verified and not deleted
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') AND
       NEW.manually_verified = TRUE AND
       NEW.deleted_at IS NULL THEN

        -- Count total participants
        SELECT COUNT(*)
        INTO v_total_participants
        FROM energy_bill_uploads
        WHERE campaign_id = NEW.campaign_id
        AND manually_verified = TRUE
        AND deleted_at IS NULL;

        -- Note: We cannot decrypt total_kwh_encrypted here without the encryption key,
        -- so aggregation will be done at the application layer.
        -- This trigger only updates participant count.

        -- Update campaign statistics
        UPDATE energy_campaigns
        SET
            total_participants = v_total_participants,
            updated_at = NOW()
        WHERE id = NEW.campaign_id;

    ELSIF TG_OP = 'DELETE' OR (TG_OP = 'UPDATE' AND NEW.deleted_at IS NOT NULL) THEN
        -- Decrement participant count when bill is deleted
        SELECT COUNT(*)
        INTO v_total_participants
        FROM energy_bill_uploads
        WHERE campaign_id = COALESCE(NEW.campaign_id, OLD.campaign_id)
        AND manually_verified = TRUE
        AND deleted_at IS NULL;

        UPDATE energy_campaigns
        SET
            total_participants = v_total_participants,
            updated_at = NOW()
        WHERE id = COALESCE(NEW.campaign_id, OLD.campaign_id);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to energy_bill_uploads
CREATE TRIGGER trigger_aggregate_building_energy
AFTER INSERT OR UPDATE OR DELETE ON energy_bill_uploads
FOR EACH ROW
EXECUTE FUNCTION aggregate_building_energy();

-- ============================================================================
-- Auto-delete expired bills (GDPR retention policy)
-- ============================================================================

-- Function to auto-delete bills past retention period
CREATE OR REPLACE FUNCTION auto_delete_expired_bills()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    UPDATE energy_bill_uploads
    SET
        deleted_at = NOW(),
        updated_at = NOW()
    WHERE retention_until < NOW()
    AND deleted_at IS NULL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Note: This function should be called by a cron job (e.g., pg_cron)
-- Example: SELECT cron.schedule('cleanup-energy-bills', '0 2 * * *', 'SELECT auto_delete_expired_bills();');

-- ============================================================================
-- Updated_at trigger for all energy tables
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach updated_at triggers
CREATE TRIGGER trigger_energy_campaigns_updated_at
BEFORE UPDATE ON energy_campaigns
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_provider_offers_updated_at
BEFORE UPDATE ON provider_offers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_energy_bill_uploads_updated_at
BEFORE UPDATE ON energy_bill_uploads
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Audit Events for Energy Buying Groups
-- ============================================================================

-- Add energy-related audit event types to the audit_logs table
-- (Assuming audit_logs table already exists from previous migrations)

COMMENT ON TABLE energy_campaigns IS 'Energy buying group campaigns for collective energy contract negotiation (GDPR-compliant)';
COMMENT ON TABLE provider_offers IS 'Energy provider offers for comparison and voting';
COMMENT ON TABLE energy_bill_uploads IS 'Encrypted energy bill uploads with GDPR consent tracking and k-anonymity aggregation';

COMMENT ON COLUMN energy_bill_uploads.total_kwh_encrypted IS 'AES-256-GCM encrypted consumption data (kWh). Decryption requires master encryption key.';
COMMENT ON COLUMN energy_bill_uploads.file_hash IS 'SHA-256 hash of original PDF bill for authenticity verification';
COMMENT ON COLUMN energy_bill_uploads.consent_signature_hash IS 'MD5 hash of consent data (unit_id|kwh|ip|timestamp) for non-repudiation';
COMMENT ON COLUMN energy_bill_uploads.retention_until IS 'GDPR retention deadline (90 days post-campaign). Bills are auto-deleted after this date.';
COMMENT ON COLUMN energy_bill_uploads.anonymized IS 'True if consumption has been aggregated to building level (k-anonymity)';

COMMENT ON COLUMN energy_campaigns.total_participants IS 'Count of verified, non-deleted bill uploads (k-anonymity: min 5 required)';
COMMENT ON COLUMN provider_offers.green_energy_pct IS 'Percentage of renewable energy (0-100). Used for green score calculation (≥100=10pts, ≥50=5pts, <50=0pts)';

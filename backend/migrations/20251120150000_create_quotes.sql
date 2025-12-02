-- Migration: Create quotes table (Issue #91 - Contractor Quotes Module)
-- Belgian legal requirement: 3 quotes mandatory for works >5000€

-- Create custom ENUM for quote status
CREATE TYPE quote_status AS ENUM (
    'Requested',      -- Quote requested from contractor
    'Received',       -- Contractor submitted quote
    'UnderReview',    -- Syndic reviewing/comparing quotes
    'Accepted',       -- Quote accepted (winner)
    'Rejected',       -- Quote rejected (loser or unqualified)
    'Expired',        -- Validity date passed
    'Withdrawn'       -- Contractor withdrew quote
);

-- Create quotes table
CREATE TABLE quotes (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    contractor_id UUID NOT NULL, -- Will reference contractors table (future)
    project_title VARCHAR(255) NOT NULL,
    project_description TEXT NOT NULL,

    -- Quote amounts
    amount_excl_vat DECIMAL(15, 2) NOT NULL,
    vat_rate DECIMAL(5, 4) NOT NULL, -- Belgian VAT: 21% = 0.2100
    amount_incl_vat DECIMAL(15, 2) NOT NULL,
    validity_date TIMESTAMPTZ NOT NULL,
    estimated_start_date TIMESTAMPTZ,
    estimated_duration_days INT NOT NULL,

    -- Scoring factors (Belgian best practices)
    warranty_years INT NOT NULL DEFAULT 2, -- 2 years (apparent defects), 10 years (structural)
    contractor_rating INT CHECK (contractor_rating >= 0 AND contractor_rating <= 100),

    -- Status & workflow
    status quote_status NOT NULL DEFAULT 'Requested',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_at TIMESTAMPTZ,
    reviewed_at TIMESTAMPTZ,
    decision_at TIMESTAMPTZ,
    decision_by UUID, -- References users(id) - Syndic who made decision
    decision_notes TEXT,

    -- Audit trail
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_amount_positive CHECK (amount_excl_vat > 0),
    CONSTRAINT check_amount_incl_vat CHECK (amount_incl_vat > 0),
    CONSTRAINT check_duration_positive CHECK (estimated_duration_days > 0),
    CONSTRAINT check_warranty_nonnegative CHECK (warranty_years >= 0),
    CONSTRAINT check_validity_future CHECK (validity_date > created_at)
);

-- Indexes for performance
CREATE INDEX idx_quotes_building_id ON quotes(building_id);
CREATE INDEX idx_quotes_contractor_id ON quotes(contractor_id);
CREATE INDEX idx_quotes_status ON quotes(status);
CREATE INDEX idx_quotes_validity_date ON quotes(validity_date);
CREATE INDEX idx_quotes_project_title ON quotes(project_title);
CREATE INDEX idx_quotes_created_at ON quotes(created_at DESC);

-- Composite index for comparison queries (Belgian law: 3 quotes minimum)
CREATE INDEX idx_quotes_building_status ON quotes(building_id, status);

-- Index for background job (find expired quotes)
CREATE INDEX idx_quotes_expired ON quotes(validity_date, status)
WHERE status NOT IN ('Accepted', 'Rejected', 'Expired', 'Withdrawn');

-- Column comments for documentation
COMMENT ON TABLE quotes IS 'Contractor quotes for works (Belgian legal requirement: 3 quotes >5000€)';
COMMENT ON COLUMN quotes.amount_excl_vat IS 'Quote amount excluding VAT (Belgian: 21% standard)';
COMMENT ON COLUMN quotes.vat_rate IS 'VAT rate (e.g., 0.21 for 21%)';
COMMENT ON COLUMN quotes.amount_incl_vat IS 'Quote amount including VAT (auto-calculated: amount_excl_vat * (1 + vat_rate))';
COMMENT ON COLUMN quotes.validity_date IS 'Date until which quote is valid (Belgian typical: 30 days)';
COMMENT ON COLUMN quotes.warranty_years IS 'Warranty duration in years (Belgian: 2 years apparent defects, 10 years structural)';
COMMENT ON COLUMN quotes.contractor_rating IS 'Contractor historical rating (0-100) for scoring algorithm';
COMMENT ON COLUMN quotes.status IS 'Quote workflow status (7 states)';
COMMENT ON COLUMN quotes.decision_by IS 'User ID (Syndic) who made accept/reject decision';
COMMENT ON COLUMN quotes.decision_notes IS 'Notes for accept/reject decision (e.g., "Best value for money")';

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_quotes_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_quotes_updated_at
    BEFORE UPDATE ON quotes
    FOR EACH ROW
    EXECUTE FUNCTION update_quotes_updated_at();

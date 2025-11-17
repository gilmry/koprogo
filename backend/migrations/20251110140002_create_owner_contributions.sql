-- Migration: Create owner_contributions table for tracking revenue (appels de fonds)
-- This table tracks payments FROM owners TO the ACP (incoming money = revenue)
-- Complements expenses table which tracks payments FROM ACP TO suppliers (outgoing money = charges)

-- Owner contributions (Appels de fonds / Cotisations)
-- Represents payments made by owners to the ACP
-- Maps to PCMN classe 7 (Produits/Revenue)
CREATE TABLE owner_contributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    unit_id UUID REFERENCES units(id) ON DELETE SET NULL,

    -- Financial details
    description TEXT NOT NULL,
    amount DECIMAL(12,2) NOT NULL CHECK (amount >= 0),

    -- Accounting
    account_code VARCHAR(10), -- PCMN code (e.g., "7000" for regular contributions, "7100" for extraordinary)

    -- Contribution type
    contribution_type VARCHAR(50) NOT NULL DEFAULT 'regular', -- 'regular', 'extraordinary', 'advance', 'adjustment'

    -- Dates
    contribution_date TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- When the contribution was due/requested
    payment_date TIMESTAMPTZ, -- When actually paid (NULL = unpaid)

    -- Payment details
    payment_method VARCHAR(50), -- 'bank_transfer', 'cash', 'check', 'domiciliation'
    payment_reference VARCHAR(100), -- Bank transfer reference, check number, etc.

    -- Status
    payment_status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'paid', 'partial', 'cancelled'

    -- Metadata
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT fk_account_code FOREIGN KEY (organization_id, account_code)
        REFERENCES accounts(organization_id, code) ON DELETE RESTRICT
);

-- Indexes for performance
CREATE INDEX idx_owner_contributions_organization ON owner_contributions(organization_id);
CREATE INDEX idx_owner_contributions_owner ON owner_contributions(owner_id);
CREATE INDEX idx_owner_contributions_unit ON owner_contributions(unit_id);
CREATE INDEX idx_owner_contributions_dates ON owner_contributions(contribution_date, payment_date);
CREATE INDEX idx_owner_contributions_status ON owner_contributions(payment_status);
CREATE INDEX idx_owner_contributions_account ON owner_contributions(account_code);

-- Note: updated_at is managed by the application layer

-- Comments
COMMENT ON TABLE owner_contributions IS 'Owner contributions/payments to the ACP (appels de fonds, cotisations). Represents REVENUE (classe 7 PCMN).';
COMMENT ON COLUMN owner_contributions.amount IS 'Contribution amount in EUR (always positive - money coming IN)';
COMMENT ON COLUMN owner_contributions.contribution_type IS 'Type: regular (quarterly fees), extraordinary (special works), advance, adjustment';
COMMENT ON COLUMN owner_contributions.payment_status IS 'Status: pending (not paid), paid (fully paid), partial (partially paid), cancelled';
COMMENT ON COLUMN owner_contributions.account_code IS 'PCMN account code (classe 7 - Produits): 7000=regular fees, 7100=extraordinary fees';

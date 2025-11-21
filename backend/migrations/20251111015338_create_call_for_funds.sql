-- Migration: Create Call for Funds (Appels de Fonds Collectifs)
-- Date: 2025-11-11
-- Description: Create table for collective payment requests sent by Syndic to all owners

-- ============================================================================
-- 1. Create contribution_type ENUM (if not exists)
-- ============================================================================
DO $$ BEGIN
    CREATE TYPE contribution_type AS ENUM ('regular', 'extraordinary', 'advance', 'adjustment');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- 2. Create call_for_funds_status ENUM
-- ============================================================================
CREATE TYPE call_for_funds_status AS ENUM ('draft', 'sent', 'partial', 'completed', 'cancelled');

-- ============================================================================
-- 3. Create call_for_funds table
-- ============================================================================
CREATE TABLE call_for_funds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Description
    title TEXT NOT NULL,
    description TEXT NOT NULL,

    -- Financial
    total_amount DECIMAL(10,2) NOT NULL CHECK (total_amount > 0),

    -- Type (use contribution_type enum)
    contribution_type contribution_type NOT NULL,

    -- Dates
    call_date TIMESTAMPTZ NOT NULL,
    due_date TIMESTAMPTZ NOT NULL CHECK (due_date > call_date),
    sent_date TIMESTAMPTZ,

    -- Status
    status call_for_funds_status NOT NULL DEFAULT 'draft',

    -- Accounting
    account_code TEXT,

    -- Metadata
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),

    CONSTRAINT valid_title CHECK (char_length(trim(title)) > 0),
    CONSTRAINT valid_description CHECK (char_length(trim(description)) > 0)
);

-- ============================================================================
-- 4. Add call_for_funds_id reference to owner_contributions
-- ============================================================================
ALTER TABLE owner_contributions
    ADD COLUMN call_for_funds_id UUID REFERENCES call_for_funds(id) ON DELETE SET NULL;

-- ============================================================================
-- 5. Create indexes for performance
-- ============================================================================
CREATE INDEX idx_call_for_funds_organization ON call_for_funds(organization_id);
CREATE INDEX idx_call_for_funds_building ON call_for_funds(building_id);
CREATE INDEX idx_call_for_funds_status ON call_for_funds(status);
CREATE INDEX idx_call_for_funds_call_date ON call_for_funds(call_date);
CREATE INDEX idx_call_for_funds_due_date ON call_for_funds(due_date);

CREATE INDEX idx_owner_contributions_call ON owner_contributions(call_for_funds_id) WHERE call_for_funds_id IS NOT NULL;

-- ============================================================================
-- 6. Add comments for documentation
-- ============================================================================
COMMENT ON TABLE call_for_funds IS 'Appels de fonds collectifs émis par le syndic pour tous les copropriétaires d''un immeuble';
COMMENT ON COLUMN call_for_funds.title IS 'Titre de l''appel de fonds (ex: "Appel de fonds T1 2025")';
COMMENT ON COLUMN call_for_funds.description IS 'Description détaillée de l''appel de fonds';
COMMENT ON COLUMN call_for_funds.total_amount IS 'Montant total à collecter auprès de tous les copropriétaires';
COMMENT ON COLUMN call_for_funds.call_date IS 'Date d''émission de l''appel de fonds';
COMMENT ON COLUMN call_for_funds.due_date IS 'Date limite de paiement';
COMMENT ON COLUMN call_for_funds.sent_date IS 'Date d''envoi effectif aux copropriétaires';
COMMENT ON COLUMN call_for_funds.status IS 'Statut: draft (brouillon), sent (envoyé), partial (partiellement payé), completed (complètement payé), cancelled (annulé)';

COMMENT ON COLUMN owner_contributions.call_for_funds_id IS 'Référence à l''appel de fonds collectif qui a généré cette contribution individuelle';

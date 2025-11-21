-- Create État Daté table (Belgian legal requirement for property sales)
-- Article 577-2 Code Civil belge

-- Create ENUMs
CREATE TYPE etat_date_status AS ENUM ('requested', 'in_progress', 'generated', 'delivered', 'expired');
CREATE TYPE etat_date_language AS ENUM ('fr', 'nl', 'de');

-- Create main table
CREATE TABLE etats_dates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,

    -- Workflow dates
    reference_date TIMESTAMPTZ NOT NULL,
    requested_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generated_date TIMESTAMPTZ,
    delivered_date TIMESTAMPTZ,

    -- Status and language
    status etat_date_status NOT NULL DEFAULT 'requested',
    language etat_date_language NOT NULL DEFAULT 'fr',

    -- Reference and notary info
    reference_number VARCHAR(100) NOT NULL UNIQUE,
    notary_name VARCHAR(255) NOT NULL,
    notary_email VARCHAR(255) NOT NULL,
    notary_phone VARCHAR(50),

    -- Section 1: Identification
    building_name VARCHAR(255) NOT NULL,
    building_address TEXT NOT NULL,
    unit_number VARCHAR(50) NOT NULL,
    unit_floor VARCHAR(20),
    unit_area DECIMAL(10,2),

    -- Section 2: Quote-parts (%)
    ordinary_charges_quota DECIMAL(5,2) NOT NULL CHECK (ordinary_charges_quota >= 0 AND ordinary_charges_quota <= 100),
    extraordinary_charges_quota DECIMAL(5,2) NOT NULL CHECK (extraordinary_charges_quota >= 0 AND extraordinary_charges_quota <= 100),

    -- Section 3: Situation financière
    owner_balance DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    arrears_amount DECIMAL(12,2) NOT NULL DEFAULT 0.00 CHECK (arrears_amount >= 0),

    -- Section 4: Provisions
    monthly_provision_amount DECIMAL(12,2) NOT NULL DEFAULT 0.00 CHECK (monthly_provision_amount >= 0),

    -- Section 5: Solde total
    total_balance DECIMAL(12,2) NOT NULL DEFAULT 0.00,

    -- Section 6: Travaux votés
    approved_works_unpaid DECIMAL(12,2) NOT NULL DEFAULT 0.00 CHECK (approved_works_unpaid >= 0),

    -- Sections 7-16: Complex data stored as JSONB
    additional_data JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Generated PDF file
    pdf_file_path TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_etats_dates_organization ON etats_dates(organization_id);
CREATE INDEX idx_etats_dates_building ON etats_dates(building_id);
CREATE INDEX idx_etats_dates_unit ON etats_dates(unit_id);
CREATE INDEX idx_etats_dates_status ON etats_dates(status);
CREATE INDEX idx_etats_dates_reference ON etats_dates(reference_number);
CREATE INDEX idx_etats_dates_requested_date ON etats_dates(requested_date);
CREATE INDEX idx_etats_dates_reference_date ON etats_dates(reference_date);

-- Index for JSONB queries
CREATE INDEX idx_etats_dates_additional_data ON etats_dates USING gin (additional_data);

-- Comments for documentation
COMMENT ON TABLE etats_dates IS 'États datés pour mutations immobilières (Art. 577-2 Code Civil belge)';
COMMENT ON COLUMN etats_dates.reference_date IS 'Date de référence pour les calculs financiers';
COMMENT ON COLUMN etats_dates.reference_number IS 'Numéro de référence unique (format: ED-YYYY-NNN-BLD-U)';
COMMENT ON COLUMN etats_dates.additional_data IS 'Sections 7-16: litiges, assurances, PV AG, budget, fonds de réserve, etc.';
COMMENT ON COLUMN etats_dates.owner_balance IS 'Solde propriétaire (positif=crédit, négatif=débit)';
COMMENT ON COLUMN etats_dates.arrears_amount IS 'Montant des arriérés (toujours >= 0)';
COMMENT ON COLUMN etats_dates.approved_works_unpaid IS 'Travaux votés non encore payés';

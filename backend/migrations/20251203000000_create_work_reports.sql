-- Migration: Create work_reports table for Digital Maintenance Logbook
-- Tracks maintenance work, repairs, and renovations with warranty management

CREATE TABLE work_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Work details
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    work_type VARCHAR(50) NOT NULL CHECK (work_type IN (
        'maintenance',
        'repair',
        'renovation',
        'emergency',
        'inspection',
        'installation',
        'other'
    )),
    contractor_name VARCHAR(255) NOT NULL,
    contractor_contact VARCHAR(255),

    -- Dates
    work_date TIMESTAMPTZ NOT NULL,
    completion_date TIMESTAMPTZ,

    -- Financial
    cost DOUBLE PRECISION NOT NULL CHECK (cost >= 0),
    invoice_number VARCHAR(100),

    -- Documentation (JSON arrays of file paths)
    photos JSONB NOT NULL DEFAULT '[]'::jsonb,
    documents JSONB NOT NULL DEFAULT '[]'::jsonb,
    notes TEXT,

    -- Warranty tracking
    warranty_type VARCHAR(50) NOT NULL CHECK (warranty_type IN (
        'none',
        'standard',    -- 2 years (vices apparents)
        'decennial',   -- 10 years (garantie décennale)
        'extended',    -- Extended warranty (equipment)
        'custom'       -- Custom warranty (uses warranty_custom_years)
    )),
    warranty_custom_years INTEGER CHECK (warranty_custom_years IS NULL OR warranty_custom_years > 0),
    warranty_expiry TIMESTAMPTZ NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CHECK (completion_date IS NULL OR completion_date >= work_date),
    CHECK (warranty_custom_years IS NULL OR warranty_type = 'custom')
);

-- Indexes for performance
CREATE INDEX idx_work_reports_organization ON work_reports(organization_id);
CREATE INDEX idx_work_reports_building ON work_reports(building_id);
CREATE INDEX idx_work_reports_work_date ON work_reports(work_date DESC);
CREATE INDEX idx_work_reports_warranty_expiry ON work_reports(warranty_expiry) WHERE warranty_type != 'none';
CREATE INDEX idx_work_reports_work_type ON work_reports(work_type);

-- GIN index for searching in photos and documents arrays
CREATE INDEX idx_work_reports_photos ON work_reports USING GIN (photos);
CREATE INDEX idx_work_reports_documents ON work_reports USING GIN (documents);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_work_reports_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_work_reports_updated_at
    BEFORE UPDATE ON work_reports
    FOR EACH ROW
    EXECUTE FUNCTION update_work_reports_updated_at();

-- Comments for documentation
COMMENT ON TABLE work_reports IS 'Digital maintenance logbook - tracks all work performed on buildings';
COMMENT ON COLUMN work_reports.warranty_type IS 'Belgian warranty types: standard (2 years), decennial (10 years)';
COMMENT ON COLUMN work_reports.cost IS 'Total cost in EUR';
COMMENT ON COLUMN work_reports.photos IS 'JSON array of photo file paths';
COMMENT ON COLUMN work_reports.documents IS 'JSON array of document file paths (invoices, certificates, etc.)';

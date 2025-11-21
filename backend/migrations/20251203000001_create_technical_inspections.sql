-- Migration: Create technical_inspections table for Digital Maintenance Logbook
-- Tracks mandatory technical inspections required by Belgian law

CREATE TABLE technical_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Inspection details
    title VARCHAR(255) NOT NULL,
    description TEXT,
    inspection_type VARCHAR(50) NOT NULL CHECK (inspection_type IN (
        'elevator',             -- Annuel (Belgian law)
        'boiler',              -- Annuel
        'electrical',          -- 5 ans
        'fire_extinguisher',   -- Annuel
        'fire_alarm',          -- Annuel
        'emergency_lighting',  -- Annuel
        'roof',                -- 5 ans
        'facade',              -- 10 ans
        'gas_installation',    -- Annuel
        'water_tank',          -- Annuel
        'drainage',            -- 5 ans
        'other'
    )),
    inspector_name VARCHAR(255) NOT NULL,
    inspector_company VARCHAR(255),
    inspector_certification VARCHAR(100),

    -- Dates
    inspection_date TIMESTAMPTZ NOT NULL,
    next_due_date TIMESTAMPTZ NOT NULL,

    -- Results
    status VARCHAR(50) NOT NULL CHECK (status IN (
        'pending',
        'completed',
        'failed',
        'passed_with_remarks'
    )) DEFAULT 'pending',
    result_summary TEXT,
    defects_found TEXT,
    recommendations TEXT,

    -- Compliance
    compliant BOOLEAN,
    compliance_certificate_number VARCHAR(100),
    compliance_valid_until TIMESTAMPTZ,

    -- Financial
    cost DOUBLE PRECISION CHECK (cost IS NULL OR cost >= 0),
    invoice_number VARCHAR(100),

    -- Documentation (JSON arrays of file paths)
    reports JSONB NOT NULL DEFAULT '[]'::jsonb,
    photos JSONB NOT NULL DEFAULT '[]'::jsonb,
    certificates JSONB NOT NULL DEFAULT '[]'::jsonb,
    notes TEXT,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CHECK (next_due_date > inspection_date),
    CHECK (compliance_valid_until IS NULL OR compliance_valid_until >= inspection_date)
);

-- Indexes for performance
CREATE INDEX idx_technical_inspections_organization ON technical_inspections(organization_id);
CREATE INDEX idx_technical_inspections_building ON technical_inspections(building_id);
CREATE INDEX idx_technical_inspections_inspection_date ON technical_inspections(inspection_date DESC);
CREATE INDEX idx_technical_inspections_next_due ON technical_inspections(next_due_date);
CREATE INDEX idx_technical_inspections_type ON technical_inspections(inspection_type);
CREATE INDEX idx_technical_inspections_status ON technical_inspections(status);

-- Index for overdue inspections
CREATE INDEX idx_technical_inspections_overdue
    ON technical_inspections(next_due_date, status)
    WHERE status = 'pending' AND next_due_date < NOW();

-- GIN indexes for searching in document arrays
CREATE INDEX idx_technical_inspections_reports ON technical_inspections USING GIN (reports);
CREATE INDEX idx_technical_inspections_photos ON technical_inspections USING GIN (photos);
CREATE INDEX idx_technical_inspections_certificates ON technical_inspections USING GIN (certificates);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_technical_inspections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_technical_inspections_updated_at
    BEFORE UPDATE ON technical_inspections
    FOR EACH ROW
    EXECUTE FUNCTION update_technical_inspections_updated_at();

-- Comments for documentation
COMMENT ON TABLE technical_inspections IS 'Mandatory technical inspections as required by Belgian law';
COMMENT ON COLUMN technical_inspections.inspection_type IS 'Belgian mandatory inspections: elevator (annual), boiler (annual), electrical (5 years), etc.';
COMMENT ON COLUMN technical_inspections.next_due_date IS 'Automatically calculated based on inspection type and Belgian legal requirements';
COMMENT ON COLUMN technical_inspections.cost IS 'Inspection cost in EUR';
COMMENT ON COLUMN technical_inspections.reports IS 'JSON array of inspection report file paths';
COMMENT ON COLUMN technical_inspections.photos IS 'JSON array of photo file paths';
COMMENT ON COLUMN technical_inspections.certificates IS 'JSON array of compliance certificate file paths';

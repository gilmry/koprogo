-- Add organization_id to all tables for complete multi-tenancy isolation
-- Issue #020 - Multi-Tenancy Parfait

-- Add organization_id to units table
ALTER TABLE units
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Add organization_id to owners table
ALTER TABLE owners
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Add organization_id to expenses table
ALTER TABLE expenses
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Add organization_id to meetings table
ALTER TABLE meetings
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Add organization_id to documents table
ALTER TABLE documents
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Create indexes for efficient organization filtering
CREATE INDEX IF NOT EXISTS idx_units_organization_id ON units(organization_id);
CREATE INDEX IF NOT EXISTS idx_owners_organization_id ON owners(organization_id);
CREATE INDEX IF NOT EXISTS idx_expenses_organization_id ON expenses(organization_id);
CREATE INDEX IF NOT EXISTS idx_meetings_organization_id ON meetings(organization_id);
CREATE INDEX IF NOT EXISTS idx_documents_organization_id ON documents(organization_id);

-- Create composite indexes for common query patterns (organization_id + created_at)
CREATE INDEX IF NOT EXISTS idx_buildings_org_created ON buildings(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_units_org_created ON units(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_owners_org_created ON owners(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_expenses_org_created ON expenses(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_meetings_org_created ON meetings(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_documents_org_created ON documents(organization_id, created_at DESC);

-- Optional: Enable Row-Level Security (RLS) for PostgreSQL
-- Uncomment these if you want database-level isolation enforcement

ALTER TABLE buildings ENABLE ROW LEVEL SECURITY;
ALTER TABLE units ENABLE ROW LEVEL SECURITY;
ALTER TABLE owners ENABLE ROW LEVEL SECURITY;
ALTER TABLE expenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE meetings ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURICREATE POLICY buildings_isolation ON buildings
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY units_isolation ON units
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY owners_isolation ON owners
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY expenses_isolation ON expenses
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY meetings_isolation ON meetings
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

CREATE POLICY documents_isolation ON documents
USING (organization_id = current_setting('app.current_organization_id', true)::UUID);

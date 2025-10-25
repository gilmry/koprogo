-- Add performance indexes for frequently queried columns
-- These indexes will significantly improve query performance under load

-- Indexes on foreign keys for JOIN operations
CREATE INDEX IF NOT EXISTS idx_units_organization_id ON units(organization_id);
CREATE INDEX IF NOT EXISTS idx_units_building_id ON units(building_id);
CREATE INDEX IF NOT EXISTS idx_units_owner_id ON units(owner_id) WHERE owner_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_buildings_organization_id ON buildings(organization_id);

CREATE INDEX IF NOT EXISTS idx_owners_organization_id ON owners(organization_id);

CREATE INDEX IF NOT EXISTS idx_expenses_organization_id ON expenses(organization_id);
CREATE INDEX IF NOT EXISTS idx_expenses_building_id ON expenses(building_id);

CREATE INDEX IF NOT EXISTS idx_meetings_organization_id ON meetings(organization_id);
CREATE INDEX IF NOT EXISTS idx_meetings_building_id ON meetings(building_id);

CREATE INDEX IF NOT EXISTS idx_documents_organization_id ON documents(organization_id);
CREATE INDEX IF NOT EXISTS idx_documents_building_id ON documents(building_id);

-- Indexes for auth-related queries
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at) WHERE NOT revoked;

-- Composite indexes for common query patterns (pagination with ordering)
CREATE INDEX IF NOT EXISTS idx_units_org_number ON units(organization_id, unit_number);
CREATE INDEX IF NOT EXISTS idx_buildings_org_name ON buildings(organization_id, name);
CREATE INDEX IF NOT EXISTS idx_owners_org_name ON owners(organization_id, last_name, first_name);
CREATE INDEX IF NOT EXISTS idx_expenses_org_date ON expenses(organization_id, expense_date DESC);

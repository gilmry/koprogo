-- Add is_seed_data column to organizations table
-- This allows us to identify and selectively delete seed data without affecting production data

ALTER TABLE organizations
ADD COLUMN is_seed_data BOOLEAN NOT NULL DEFAULT false;

-- Create index for faster queries
CREATE INDEX idx_organizations_is_seed_data ON organizations(is_seed_data);

-- Add comment
COMMENT ON COLUMN organizations.is_seed_data IS 'Indicates if this organization was created by the seed script (for testing/demo purposes)';

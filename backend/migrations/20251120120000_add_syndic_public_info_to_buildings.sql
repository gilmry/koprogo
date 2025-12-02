-- Add public syndic information to buildings table
-- Issue #92: Public Syndic Information Page (Belgian legal requirement)
-- Belgian law requires building syndics to publicly display contact information

-- Add syndic contact fields
ALTER TABLE buildings
ADD COLUMN syndic_name VARCHAR(255),
ADD COLUMN syndic_email VARCHAR(255),
ADD COLUMN syndic_phone VARCHAR(50),
ADD COLUMN syndic_address TEXT,
ADD COLUMN syndic_office_hours TEXT,
ADD COLUMN syndic_emergency_contact VARCHAR(50),
ADD COLUMN slug VARCHAR(255) UNIQUE;

-- Create index for slug lookups (public page access)
CREATE INDEX idx_buildings_slug ON buildings(slug);

-- Create index for syndic name searches
CREATE INDEX idx_buildings_syndic_name ON buildings(syndic_name);

-- Comments for documentation
COMMENT ON COLUMN buildings.syndic_name IS 'Name of the property syndic/manager (public info per Belgian law)';
COMMENT ON COLUMN buildings.syndic_email IS 'Public contact email for syndic';
COMMENT ON COLUMN buildings.syndic_phone IS 'Public contact phone for syndic';
COMMENT ON COLUMN buildings.syndic_address IS 'Syndic office address';
COMMENT ON COLUMN buildings.syndic_office_hours IS 'Office hours (e.g., "Mon-Fri 9h-17h")';
COMMENT ON COLUMN buildings.syndic_emergency_contact IS 'Emergency contact number (24/7)';
COMMENT ON COLUMN buildings.slug IS 'URL-friendly slug for public pages (SEO-optimized)';

-- Note: All syndic fields are optional (nullable) as not all buildings may have a syndic yet
-- The slug will be auto-generated from building name + address when creating/updating buildings

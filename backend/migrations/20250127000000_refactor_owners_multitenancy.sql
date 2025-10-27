-- Migration: Refactor owners for multi-tenancy and many-to-many relationships
-- This migration creates a unit_owners junction table for many-to-many relationships

-- Step 1: Add organization_id to owners table (if not exists)
ALTER TABLE owners ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Step 2: Remove UNIQUE constraint on email (owners can exist in multiple organizations)
ALTER TABLE owners DROP CONSTRAINT IF EXISTS owners_email_key;

-- Step 3: Add composite unique constraint (email unique per organization)
DROP INDEX IF EXISTS idx_owners_email_org;
CREATE UNIQUE INDEX idx_owners_email_org ON owners(email, organization_id);

-- Step 4: Add index for organization lookups (if not exists)
CREATE INDEX IF NOT EXISTS idx_owners_organization ON owners(organization_id);

-- Step 5: Create unit_owners junction table for many-to-many relationship
CREATE TABLE unit_owners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,

    -- Ownership percentage (e.g., 0.5 for 50%, 1.0 for 100%)
    ownership_percentage DOUBLE PRECISION NOT NULL DEFAULT 1.0 CHECK (ownership_percentage > 0 AND ownership_percentage <= 1.0),

    -- Date when ownership started
    start_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Date when ownership ended (NULL if still current owner)
    end_date TIMESTAMPTZ,

    -- Is this owner the primary contact for this unit?
    is_primary_contact BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure no duplicate owner-unit pairs that are active
    UNIQUE(unit_id, owner_id, end_date),

    -- Ensure end_date is after start_date
    CHECK (end_date IS NULL OR end_date > start_date)
);

-- Indexes for performance
CREATE INDEX idx_unit_owners_unit ON unit_owners(unit_id);
CREATE INDEX idx_unit_owners_owner ON unit_owners(owner_id);
CREATE INDEX idx_unit_owners_active ON unit_owners(unit_id, owner_id) WHERE end_date IS NULL;

-- Step 6: Migrate existing data from units.owner_id to unit_owners
INSERT INTO unit_owners (unit_id, owner_id, ownership_percentage, is_primary_contact, start_date)
SELECT
    u.id as unit_id,
    u.owner_id,
    1.0 as ownership_percentage,
    true as is_primary_contact,
    u.created_at as start_date
FROM units u
WHERE u.owner_id IS NOT NULL;

-- Step 7: Remove owner_id column from units (deprecated)
-- We keep it for now to ensure backward compatibility, but it should be removed later
-- ALTER TABLE units DROP COLUMN owner_id;

-- Step 8: Add a comment to mark owner_id as deprecated
COMMENT ON COLUMN units.owner_id IS 'DEPRECATED: Use unit_owners table instead. Will be removed in future version.';

-- Step 9: Create a view for easy querying of current unit owners
CREATE OR REPLACE VIEW v_current_unit_owners AS
SELECT
    uo.id,
    uo.unit_id,
    uo.owner_id,
    uo.ownership_percentage,
    uo.is_primary_contact,
    u.building_id,
    u.unit_number,
    u.unit_type,
    o.first_name,
    o.last_name,
    o.email,
    o.phone,
    o.organization_id
FROM unit_owners uo
JOIN units u ON uo.unit_id = u.id
JOIN owners o ON uo.owner_id = o.id
WHERE uo.end_date IS NULL;  -- Only current owners

-- Step 10: Create function to get all owners of a unit
CREATE OR REPLACE FUNCTION get_unit_owners(p_unit_id UUID)
RETURNS TABLE (
    owner_id UUID,
    first_name VARCHAR,
    last_name VARCHAR,
    email VARCHAR,
    phone VARCHAR,
    ownership_percentage DOUBLE PRECISION,
    is_primary_contact BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        o.id,
        o.first_name,
        o.last_name,
        o.email,
        o.phone,
        uo.ownership_percentage,
        uo.is_primary_contact
    FROM unit_owners uo
    JOIN owners o ON uo.owner_id = o.id
    WHERE uo.unit_id = p_unit_id
      AND uo.end_date IS NULL
    ORDER BY uo.is_primary_contact DESC, o.last_name, o.first_name;
END;
$$ LANGUAGE plpgsql;

-- Step 11: Create function to get all units of an owner
CREATE OR REPLACE FUNCTION get_owner_units(p_owner_id UUID)
RETURNS TABLE (
    unit_id UUID,
    building_id UUID,
    building_name VARCHAR,
    unit_number VARCHAR,
    unit_type TEXT,
    ownership_percentage DOUBLE PRECISION,
    is_primary_contact BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.building_id,
        b.name as building_name,
        u.unit_number,
        u.unit_type::TEXT,
        uo.ownership_percentage,
        uo.is_primary_contact
    FROM unit_owners uo
    JOIN units u ON uo.unit_id = u.id
    JOIN buildings b ON u.building_id = b.id
    WHERE uo.owner_id = p_owner_id
      AND uo.end_date IS NULL
    ORDER BY b.name, u.unit_number;
END;
$$ LANGUAGE plpgsql;

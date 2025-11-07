-- Migration: Add missing columns to board tables for multi-tenancy and active status
-- Date: 2025-11-01
-- Issue: #82 - Fix seed script compatibility

-- ============================================================================
-- Add organization_id and is_active to board_members
-- ============================================================================

-- Add organization_id to board_members
ALTER TABLE board_members
ADD COLUMN organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Populate organization_id from building relationship
UPDATE board_members bm
SET organization_id = b.organization_id
FROM buildings b
WHERE bm.building_id = b.id;

-- Make it NOT NULL after population
ALTER TABLE board_members
ALTER COLUMN organization_id SET NOT NULL;

-- Add is_active flag to board_members
ALTER TABLE board_members
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_board_members_organization_id ON board_members(organization_id);
CREATE INDEX IF NOT EXISTS idx_board_members_is_active ON board_members(is_active);

-- ============================================================================
-- Add organization_id to board_decisions
-- ============================================================================

-- Add organization_id to board_decisions
ALTER TABLE board_decisions
ADD COLUMN organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Populate organization_id from building relationship
UPDATE board_decisions bd
SET organization_id = b.organization_id
FROM buildings b
WHERE bd.building_id = b.id;

-- Make it NOT NULL after population
ALTER TABLE board_decisions
ALTER COLUMN organization_id SET NOT NULL;

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_board_decisions_organization_id ON board_decisions(organization_id);

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON COLUMN board_members.organization_id IS 'Organization that owns the building (for multi-tenancy)';
COMMENT ON COLUMN board_members.is_active IS 'Whether the board member mandate is currently active';
COMMENT ON COLUMN board_decisions.organization_id IS 'Organization that owns the building (for multi-tenancy)';

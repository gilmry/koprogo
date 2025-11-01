-- Migration: Add board_member role to user_roles table
-- Date: 2025-11-01
-- Issue: #82 - Conseil de Copropriété

-- Add 'board_member' to the CHECK constraint of user_roles table
ALTER TABLE user_roles DROP CONSTRAINT IF EXISTS user_roles_role_check;

ALTER TABLE user_roles ADD CONSTRAINT user_roles_role_check
    CHECK (role IN ('superadmin', 'syndic', 'accountant', 'board_member', 'owner'));

-- Create index for efficient querying of board members
CREATE INDEX IF NOT EXISTS idx_user_roles_board_member
    ON user_roles(role, organization_id)
    WHERE role = 'board_member';

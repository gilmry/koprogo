-- Migration: Add user_id to owners table for linking users to owner entities
-- This allows admins to configure which user account corresponds to which owner
-- An owner can optionally be linked to a user account (for portal access)

-- Step 1: Add user_id column to owners table
ALTER TABLE owners
ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Step 2: Add index for user_id lookups
CREATE INDEX IF NOT EXISTS idx_owners_user_id ON owners(user_id);

-- Step 3: Add unique constraint (one user can only be linked to one owner)
CREATE UNIQUE INDEX IF NOT EXISTS idx_owners_user_id_unique ON owners(user_id) WHERE user_id IS NOT NULL;

-- Step 4: Add comment
COMMENT ON COLUMN owners.user_id IS 'Optional link to user account. Set by admin to grant portal access to this owner.';

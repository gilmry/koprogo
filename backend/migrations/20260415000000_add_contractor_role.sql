-- Migration: Add contractor role to user_roles and users tables
-- Date: 2026-04-15
-- Story: STORY-P7-1001 — Contractor as first-class domain role
-- Refs: docs/cowork/Plan-Maury-2026-04-13.md Epic P7-10

-- 1. Extend user_roles CHECK constraint to include 'contractor'
ALTER TABLE user_roles DROP CONSTRAINT IF EXISTS user_roles_role_check;
ALTER TABLE user_roles ADD CONSTRAINT user_roles_role_check
    CHECK (role IN ('superadmin', 'syndic', 'accountant', 'board_member', 'contractor', 'owner'));

-- 2. Extend users table CHECK constraint (if it exists)
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_role_check;
ALTER TABLE users ADD CONSTRAINT users_role_check
    CHECK (role IN ('superadmin', 'syndic', 'accountant', 'board_member', 'contractor', 'owner'));

-- 3. Index for efficient contractor queries
CREATE INDEX IF NOT EXISTS idx_user_roles_contractor
    ON user_roles(role, organization_id)
    WHERE role = 'contractor';

-- 4. Contractor profiles table (profession, SIREN/TVA, insurance)
CREATE TABLE IF NOT EXISTS contractor_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    profession VARCHAR(100) NOT NULL,
    siren_or_vat VARCHAR(20),                -- Belgian SIREN or VAT number (BE0123456789)
    insurance_number VARCHAR(50),
    insurance_expires_at TIMESTAMPTZ,
    hourly_rate NUMERIC(10, 2),
    specialties TEXT[],                       -- Array of specialties (e.g. ['plumbing', 'heating'])
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_contractor_profiles_org
    ON contractor_profiles(organization_id) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_contractor_profiles_user
    ON contractor_profiles(user_id);

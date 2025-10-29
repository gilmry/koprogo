CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('superadmin', 'syndic', 'accountant', 'owner')),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, role, organization_id)
);

-- Ensure fast lookups and enforce a single active role per user
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles(role);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_roles_primary_per_user
    ON user_roles(user_id)
    WHERE is_primary = true;
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_roles_unique_role_no_org
    ON user_roles(user_id, role)
    WHERE organization_id IS NULL;

-- Backfill existing users with their current role as primary assignment
INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
SELECT gen_random_uuid(), id, role, organization_id, true, created_at, updated_at
FROM users
ON CONFLICT DO NOTHING;

-- Cleanup duplicate superadmin assignments if they already existed
WITH ranked_roles AS (
    SELECT
        id,
        ROW_NUMBER() OVER (PARTITION BY user_id, role ORDER BY created_at) AS rn
    FROM user_roles
    WHERE organization_id IS NULL
)
DELETE FROM user_roles
WHERE id IN (SELECT id FROM ranked_roles WHERE rn > 1);

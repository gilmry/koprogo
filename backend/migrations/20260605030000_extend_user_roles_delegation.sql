-- Migration: Story 3.5 — Temporary role delegation.
-- Date: 2026-06-05
-- Story: 3.5 — Délégation temporaire de rôle (FR8 INV-8).
-- Refs: docs/maury/refonte-ux-multi-role-acp/stories.md L392-410
--
-- A syndic can temporarily delegate their role to an Owner for N days. The
-- assignment auto-expires via `valid_until` (NULL = permanent native role).
-- `delegated_from_user_id` records the delegator so the @security invariant
-- "non-transitive" (no re-delegation of a delegated role) can be enforced at
-- the use-case layer.

ALTER TABLE user_roles
    ADD COLUMN IF NOT EXISTS valid_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS delegated_from_user_id UUID
        REFERENCES users(id) ON DELETE SET NULL;

-- Partial index: only the currently active delegations (valid_until NOT NULL).
-- The permanent / native rows (valid_until IS NULL) keep using the existing
-- `idx_user_roles_user_id` for lookups.
CREATE INDEX IF NOT EXISTS idx_user_roles_delegation_active
    ON user_roles (user_id, role)
    WHERE valid_until IS NOT NULL;

-- Audit-only index on the delegator side (audit views / list_delegations_of).
CREATE INDEX IF NOT EXISTS idx_user_roles_delegated_from
    ON user_roles (delegated_from_user_id)
    WHERE delegated_from_user_id IS NOT NULL;

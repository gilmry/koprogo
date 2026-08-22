-- Story 3.5 — rollback temporary role delegation columns.
DROP INDEX IF EXISTS idx_user_roles_delegated_from;
DROP INDEX IF EXISTS idx_user_roles_delegation_active;

ALTER TABLE user_roles
    DROP COLUMN IF EXISTS delegated_from_user_id,
    DROP COLUMN IF EXISTS valid_until;

DROP INDEX IF EXISTS idx_owner_contributions_acp_id;
ALTER TABLE owner_contributions DROP CONSTRAINT IF EXISTS fk_owner_contributions_acp;
ALTER TABLE owner_contributions DROP COLUMN IF EXISTS acp_id;

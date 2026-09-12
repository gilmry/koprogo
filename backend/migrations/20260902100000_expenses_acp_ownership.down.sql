DROP INDEX IF EXISTS idx_expenses_acp_id;
ALTER TABLE expenses DROP CONSTRAINT IF EXISTS fk_expenses_acp;
ALTER TABLE expenses DROP COLUMN IF EXISTS acp_id;

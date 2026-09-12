DROP INDEX IF EXISTS idx_convocations_acp_id;
ALTER TABLE convocations DROP CONSTRAINT IF EXISTS fk_convocations_acp;
ALTER TABLE convocations DROP COLUMN IF EXISTS acp_id;

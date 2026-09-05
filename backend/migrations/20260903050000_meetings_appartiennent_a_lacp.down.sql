DROP INDEX IF EXISTS idx_meetings_acp_id;
ALTER TABLE meetings DROP CONSTRAINT IF EXISTS fk_meetings_acp;
ALTER TABLE meetings DROP COLUMN IF EXISTS acp_id;

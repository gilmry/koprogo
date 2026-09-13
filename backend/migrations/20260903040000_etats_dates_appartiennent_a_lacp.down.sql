DROP INDEX IF EXISTS idx_etats_dates_acp_id;
ALTER TABLE etats_dates DROP CONSTRAINT IF EXISTS fk_etats_dates_acp;
ALTER TABLE etats_dates DROP COLUMN IF EXISTS acp_id;

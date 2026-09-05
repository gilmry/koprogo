DROP INDEX IF EXISTS idx_call_for_funds_acp_id;
ALTER TABLE call_for_funds DROP CONSTRAINT IF EXISTS fk_call_for_funds_acp;
ALTER TABLE call_for_funds DROP COLUMN IF EXISTS acp_id;

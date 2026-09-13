DROP INDEX IF EXISTS idx_journal_entries_acp_id;
ALTER TABLE journal_entries DROP CONSTRAINT IF EXISTS fk_journal_entries_acp;
ALTER TABLE journal_entries DROP COLUMN IF EXISTS acp_id;

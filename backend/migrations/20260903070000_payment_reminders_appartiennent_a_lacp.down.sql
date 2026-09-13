DROP INDEX IF EXISTS idx_payment_reminders_acp_id;
ALTER TABLE payment_reminders DROP CONSTRAINT IF EXISTS fk_payment_reminders_acp;
ALTER TABLE payment_reminders DROP COLUMN IF EXISTS acp_id;

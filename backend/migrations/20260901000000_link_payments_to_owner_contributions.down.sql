DROP INDEX IF EXISTS idx_payments_contribution_id;
ALTER TABLE payments DROP CONSTRAINT IF EXISTS fk_payments_contribution;
ALTER TABLE payments DROP COLUMN IF EXISTS contribution_id;

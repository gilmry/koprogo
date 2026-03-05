-- Fix payment constraint to allow succeeded_at to remain set when status is 'refunded'.
-- A refunded payment was previously succeeded, so succeeded_at is a meaningful audit trail.
-- Original constraint required succeeded_at IS NULL for ALL non-succeeded statuses,
-- which blocked the refund status transition.

ALTER TABLE payments DROP CONSTRAINT IF EXISTS chk_succeeded_at_when_succeeded;

ALTER TABLE payments ADD CONSTRAINT chk_succeeded_at_when_succeeded CHECK (
    (status = 'succeeded' AND succeeded_at IS NOT NULL) OR
    (status = 'refunded' AND succeeded_at IS NOT NULL) OR
    (status NOT IN ('succeeded', 'refunded') AND succeeded_at IS NULL)
);

COMMENT ON CONSTRAINT chk_succeeded_at_when_succeeded ON payments IS
'Ensures succeeded_at is set for succeeded and refunded payments (refunded payments were previously succeeded),
and is NULL for all other statuses (pending, processing, requires_action, failed, cancelled).';

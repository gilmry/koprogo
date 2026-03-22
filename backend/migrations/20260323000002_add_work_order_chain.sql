-- Migration: Issue #309 - Connect expense approval chain
-- Add contractor report link to expenses and work order tracking to tickets

-- Add contractor_report_id FK to expenses
ALTER TABLE expenses
    ADD COLUMN IF NOT EXISTS contractor_report_id UUID REFERENCES contractor_reports(id) ON DELETE SET NULL;

-- Add work order tracking to tickets
ALTER TABLE tickets
    ADD COLUMN IF NOT EXISTS work_order_sent_at TIMESTAMPTZ;

-- Index for finding expenses linked to reports (Issue #309)
CREATE INDEX IF NOT EXISTS idx_expenses_contractor_report_id
    ON expenses (contractor_report_id) WHERE contractor_report_id IS NOT NULL;

-- Index for finding tickets with work orders sent (for PWA magic link workflow)
CREATE INDEX IF NOT EXISTS idx_tickets_work_order_sent
    ON tickets (work_order_sent_at) WHERE work_order_sent_at IS NOT NULL;

-- Composite index for finding work expenses pending validation
CREATE INDEX IF NOT EXISTS idx_expenses_works_pending_approval
    ON expenses (organization_id, approval_status)
    WHERE category = 'works';

-- Comments
COMMENT ON COLUMN expenses.contractor_report_id IS 'Link to validated contractor report (Issue #309) - required for Works category before approval';
COMMENT ON COLUMN tickets.work_order_sent_at IS 'Timestamp when magic link PWA was sent to contractor (Issue #309)';

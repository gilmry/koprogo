-- Migration: Create payment_reminders table for automated payment recovery workflow
-- Issue #83: Payment Recovery Workflow (Workflow Recouvrement Impayés)
-- This enables automatic reminder system for overdue payments with 3 escalation levels

-- Create ENUM types for reminder level, status, and delivery method
CREATE TYPE reminder_level AS ENUM ('FirstReminder', 'SecondReminder', 'FormalNotice');
CREATE TYPE reminder_status AS ENUM ('Pending', 'Sent', 'Opened', 'Paid', 'Escalated', 'Cancelled');
CREATE TYPE delivery_method AS ENUM ('Email', 'RegisteredLetter', 'Bailiff');

-- Create payment_reminders table
CREATE TABLE payment_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,

    -- Reminder details
    level reminder_level NOT NULL DEFAULT 'FirstReminder',
    status reminder_status NOT NULL DEFAULT 'Pending',

    -- Financial details
    amount_owed DOUBLE PRECISION NOT NULL CHECK (amount_owed > 0),
    penalty_amount DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (penalty_amount >= 0),
    total_amount DOUBLE PRECISION NOT NULL CHECK (total_amount >= amount_owed),

    -- Timing details
    due_date TIMESTAMPTZ NOT NULL, -- Original due date of the expense
    days_overdue INTEGER NOT NULL CHECK (days_overdue >= 0),

    -- Delivery details
    delivery_method delivery_method NOT NULL DEFAULT 'Email',
    sent_date TIMESTAMPTZ, -- When reminder was sent
    opened_date TIMESTAMPTZ, -- When email was opened (if tracked)

    -- Document and tracking
    pdf_path TEXT, -- Path to generated PDF letter
    tracking_number TEXT, -- Tracking number for registered letter
    notes TEXT, -- Additional notes (e.g., cancellation reason)

    -- Audit timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CHECK (opened_date IS NULL OR (sent_date IS NOT NULL AND opened_date >= sent_date)),
    CHECK (total_amount = amount_owed + penalty_amount)
);

-- Indexes for performance
CREATE INDEX idx_payment_reminders_organization ON payment_reminders(organization_id);
CREATE INDEX idx_payment_reminders_expense ON payment_reminders(expense_id);
CREATE INDEX idx_payment_reminders_owner ON payment_reminders(owner_id);
CREATE INDEX idx_payment_reminders_status ON payment_reminders(status);
CREATE INDEX idx_payment_reminders_level ON payment_reminders(level);
CREATE INDEX idx_payment_reminders_due_date ON payment_reminders(due_date);

-- Index for finding pending reminders to send
CREATE INDEX idx_payment_reminders_pending ON payment_reminders(status, created_at)
    WHERE status = 'Pending';

-- Index for finding sent reminders needing escalation
CREATE INDEX idx_payment_reminders_escalation ON payment_reminders(status, sent_date, level)
    WHERE status IN ('Sent', 'Opened');

-- Index for analytics (overdue by organization)
CREATE INDEX idx_payment_reminders_overdue_org ON payment_reminders(organization_id, days_overdue, status);

-- Comments for documentation
COMMENT ON TABLE payment_reminders IS 'Automated payment reminder system for overdue expenses (Belgian legal compliance)';
COMMENT ON COLUMN payment_reminders.level IS 'Escalation level: FirstReminder (J+15), SecondReminder (J+30), FormalNotice (J+60)';
COMMENT ON COLUMN payment_reminders.penalty_amount IS 'Late payment penalties calculated at Belgian legal rate (8% annual)';
COMMENT ON COLUMN payment_reminders.days_overdue IS 'Number of days since due_date';
COMMENT ON COLUMN payment_reminders.tracking_number IS 'Postal tracking number for registered letters (mise en demeure)';

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_payment_reminders_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function before UPDATE
CREATE TRIGGER trigger_payment_reminders_updated_at
    BEFORE UPDATE ON payment_reminders
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_reminders_updated_at();

-- Grant permissions (if using row-level security in future)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON payment_reminders TO koprogo;

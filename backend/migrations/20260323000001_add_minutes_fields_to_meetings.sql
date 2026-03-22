-- Migration: Add PV (procès-verbal) distribution tracking to meetings (Issue #313)
-- Belgian legal requirement: Track when AG minutes are sent to owners (within 30 days)

ALTER TABLE meetings
    ADD COLUMN IF NOT EXISTS minutes_document_id UUID REFERENCES documents(id),
    ADD COLUMN IF NOT EXISTS minutes_sent_at TIMESTAMPTZ;

-- Create index for finding overdue minutes
-- Minutes are overdue if meeting is Completed, minutes_sent_at IS NULL, and updated_at < 30 days ago
CREATE INDEX IF NOT EXISTS idx_meetings_minutes_overdue
    ON meetings (status, updated_at)
    WHERE status = 'completed' AND minutes_sent_at IS NULL;

-- Add comments for documentation
COMMENT ON COLUMN meetings.minutes_document_id IS 'FK to documents table - PDF of the AG minutes (procès-verbal)';
COMMENT ON COLUMN meetings.minutes_sent_at IS 'When the AG minutes were sent to owners - must be within 30 days of completion per Belgian law';

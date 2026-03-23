-- AGE Request enhancements (Issue #279 - Part 1)
-- Add syndic notification tracking and deadline management

-- Add syndic notification timestamp
ALTER TABLE age_requests ADD COLUMN IF NOT EXISTS syndic_notified_at TIMESTAMPTZ;

-- Add configurable syndic deadline (default 15 days per Art. 3.87 §2 CC)
ALTER TABLE age_requests ADD COLUMN IF NOT EXISTS syndic_deadline_days INTEGER DEFAULT 15;

-- Index for monitoring syndic response deadlines
-- Useful for background jobs checking which AGE requests have expired deadlines
CREATE INDEX IF NOT EXISTS idx_age_requests_syndic_deadline
    ON age_requests (submitted_to_syndic_at, status)
    WHERE status = 'submitted';

-- Index for finding AGE requests not yet notified
CREATE INDEX IF NOT EXISTS idx_age_requests_not_notified
    ON age_requests (created_at)
    WHERE status = 'reached' AND syndic_notified_at IS NULL;

-- Index for reaching reached status (for auto-notification workflows)
CREATE INDEX IF NOT EXISTS idx_age_requests_reached
    ON age_requests (status, threshold_reached_at)
    WHERE status = 'reached';

-- Columns documentation
COMMENT ON COLUMN age_requests.syndic_notified_at
    IS 'When the syndic was formally notified of the AGE request (status = reached)';
COMMENT ON COLUMN age_requests.syndic_deadline_days
    IS 'Number of days the syndic has to respond after being notified (default 15, per Art. 3.87 §2 CC)';

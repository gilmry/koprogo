-- Migration: Add is_second_convocation field to meetings table
-- Issue #311: Support for 2nd convocation without quorum requirement (Art. 3.87 §5 CC)
-- Date: 2026-03-23

-- Add is_second_convocation column
ALTER TABLE meetings ADD COLUMN IF NOT EXISTS is_second_convocation BOOLEAN NOT NULL DEFAULT FALSE;

-- Add comment explaining the Belgian legal requirement
COMMENT ON COLUMN meetings.is_second_convocation IS
    'Art. 3.87 §5 Code Civil Belge: Mark as true for 2nd convocation (no quorum check required).
     When true, check_quorum_for_voting() is skipped, allowing voting regardless of quorum status.';

-- Create partial index for finding 2nd convocations efficiently
CREATE INDEX IF NOT EXISTS idx_meetings_second_convocation
    ON meetings (is_second_convocation)
    WHERE is_second_convocation = TRUE;

-- Create index for building+status+is_second_convocation queries (common filtering pattern)
CREATE INDEX IF NOT EXISTS idx_meetings_building_status_convocation
    ON meetings (building_id, status, is_second_convocation);

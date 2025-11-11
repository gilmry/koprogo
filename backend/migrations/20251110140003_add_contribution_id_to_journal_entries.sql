-- Add contribution_id column to journal_entries table
-- This allows journal entries to reference owner_contributions (revenue)
-- in addition to expenses

ALTER TABLE journal_entries
ADD COLUMN contribution_id UUID REFERENCES owner_contributions(id) ON DELETE SET NULL;

-- Add index for contribution_id lookups
CREATE INDEX idx_journal_entries_contribution ON journal_entries(contribution_id);

-- Add comment
COMMENT ON COLUMN journal_entries.contribution_id IS 'Reference to owner_contribution if this journal entry tracks revenue';

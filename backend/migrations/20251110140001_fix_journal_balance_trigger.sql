-- Migration: Fix journal balance trigger to validate at statement level, not row level
--
-- The original trigger validated balance after EACH row insertion, causing false positives
-- when lines are inserted sequentially. We need to validate after ALL lines are inserted.

-- Drop the old trigger
DROP TRIGGER IF EXISTS trigger_validate_journal_balance ON journal_entry_lines;

-- Drop the old function
DROP FUNCTION IF EXISTS validate_journal_entry_balance();

-- Create new function that validates balance (same logic, different timing)
CREATE OR REPLACE FUNCTION validate_journal_entry_balance()
RETURNS TRIGGER AS $$
DECLARE
    total_debits DECIMAL(12,2);
    total_credits DECIMAL(12,2);
    affected_entry_id UUID;
BEGIN
    -- Get the journal_entry_id from the affected row
    IF TG_OP = 'DELETE' THEN
        affected_entry_id := OLD.journal_entry_id;
    ELSE
        affected_entry_id := NEW.journal_entry_id;
    END IF;

    -- Calculate totals for the journal entry
    SELECT
        COALESCE(SUM(debit), 0),
        COALESCE(SUM(credit), 0)
    INTO total_debits, total_credits
    FROM journal_entry_lines
    WHERE journal_entry_id = affected_entry_id;

    -- Allow small rounding differences (0.01€)
    IF ABS(total_debits - total_credits) > 0.01 THEN
        RAISE EXCEPTION 'Journal entry % is unbalanced: debits=% credits=%',
            affected_entry_id,
            total_debits,
            total_credits;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create new trigger that fires AFTER the entire statement (not after each row)
-- This allows multiple lines to be inserted before validation
CREATE CONSTRAINT TRIGGER trigger_validate_journal_balance
    AFTER INSERT OR UPDATE OR DELETE ON journal_entry_lines
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION validate_journal_entry_balance();

COMMENT ON TRIGGER trigger_validate_journal_balance ON journal_entry_lines IS
    'Validates journal entry balance (debits = credits) after all lines are inserted (deferred constraint)';

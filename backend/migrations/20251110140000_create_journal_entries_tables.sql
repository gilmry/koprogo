-- Migration: Create journal entries tables for double-entry bookkeeping
--
-- CREDITS & ATTRIBUTION:
-- This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
-- Noalyss is a free accounting software for Belgian and French accounting
-- License: GPL-2.0-or-later (GNU General Public License version 2 or later)
-- Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
-- Copyright: Dany De Bontridder <dany@alchimerys.eu>
--
-- Tables inspired by Noalyss structure:
-- - journal_entries (inspired by Noalyss `jrn` table)
-- - journal_entry_lines (inspired by Noalyss `jrnx` table)

-- Table: journal_entries
-- Represents a single accounting transaction (e.g., an expense payment)
-- Each entry contains multiple lines (debits and credits) that must balance
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    entry_date TIMESTAMPTZ NOT NULL,
    description TEXT,
    -- Optional reference to source document (invoice number, etc.)
    document_ref VARCHAR(100),
    -- Optional link to the expense that generated this entry
    expense_id UUID REFERENCES expenses(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_journal_entries_organization ON journal_entries(organization_id);
CREATE INDEX idx_journal_entries_entry_date ON journal_entries(entry_date);
CREATE INDEX idx_journal_entries_expense ON journal_entries(expense_id);

-- Table: journal_entry_lines
-- Individual debit/credit lines that make up a journal entry
-- Implements double-entry bookkeeping: sum(debits) = sum(credits) per journal_entry_id
CREATE TABLE journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    -- Link to account via composite key (organization_id + account_code)
    organization_id UUID NOT NULL,
    account_code VARCHAR(10) NOT NULL,
    -- Debit amount (increases assets and expenses, decreases liabilities and revenue)
    debit DECIMAL(12,2) NOT NULL DEFAULT 0.00 CHECK (debit >= 0),
    -- Credit amount (decreases assets and expenses, increases liabilities and revenue)
    credit DECIMAL(12,2) NOT NULL DEFAULT 0.00 CHECK (credit >= 0),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraint: Each line must be EITHER debit OR credit (not both, not neither)
    CONSTRAINT debit_or_credit CHECK (
        (debit > 0 AND credit = 0) OR (credit > 0 AND debit = 0)
    ),

    -- Foreign key to accounts table using composite key
    CONSTRAINT fk_account FOREIGN KEY (organization_id, account_code)
        REFERENCES accounts(organization_id, code) ON DELETE RESTRICT
);

CREATE INDEX idx_journal_entry_lines_entry ON journal_entry_lines(journal_entry_id);
CREATE INDEX idx_journal_entry_lines_account ON journal_entry_lines(organization_id, account_code);
CREATE INDEX idx_journal_entry_lines_debit ON journal_entry_lines(debit) WHERE debit > 0;
CREATE INDEX idx_journal_entry_lines_credit ON journal_entry_lines(credit) WHERE credit > 0;

-- Function: Validate that journal entry debits equal credits
-- This ensures double-entry bookkeeping integrity
CREATE OR REPLACE FUNCTION validate_journal_entry_balance()
RETURNS TRIGGER AS $$
DECLARE
    total_debits DECIMAL(12,2);
    total_credits DECIMAL(12,2);
BEGIN
    -- Calculate totals for the journal entry
    SELECT
        COALESCE(SUM(debit), 0),
        COALESCE(SUM(credit), 0)
    INTO total_debits, total_credits
    FROM journal_entry_lines
    WHERE journal_entry_id = COALESCE(NEW.journal_entry_id, OLD.journal_entry_id);

    -- Allow small rounding differences (0.01€)
    IF ABS(total_debits - total_credits) > 0.01 THEN
        RAISE EXCEPTION 'Journal entry % is unbalanced: debits=% credits=%',
            COALESCE(NEW.journal_entry_id, OLD.journal_entry_id),
            total_debits,
            total_credits;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger: Validate balance on INSERT/UPDATE/DELETE of journal entry lines
CREATE TRIGGER trigger_validate_journal_balance
    AFTER INSERT OR UPDATE OR DELETE ON journal_entry_lines
    FOR EACH ROW
    EXECUTE FUNCTION validate_journal_entry_balance();

-- View: Account balances calculated from journal entries
-- This replaces the old method of calculating balances from expenses directly
CREATE VIEW account_balances AS
SELECT
    jel.organization_id,
    jel.account_code,
    a.label as account_label,
    a.account_type,
    SUM(jel.debit) as total_debit,
    SUM(jel.credit) as total_credit,
    -- Balance calculation depends on account type:
    -- Assets & Expenses: debit increases balance (debit - credit)
    -- Liabilities & Revenue: credit increases balance (credit - debit)
    CASE
        WHEN a.account_type IN ('ASSET', 'EXPENSE') THEN SUM(jel.debit) - SUM(jel.credit)
        WHEN a.account_type IN ('LIABILITY', 'REVENUE') THEN SUM(jel.credit) - SUM(jel.debit)
        ELSE 0
    END as balance
FROM journal_entry_lines jel
JOIN accounts a ON a.organization_id = jel.organization_id AND a.code = jel.account_code
GROUP BY jel.organization_id, jel.account_code, a.label, a.account_type;

-- Comments for documentation
COMMENT ON TABLE journal_entries IS 'Accounting journal entries - each entry represents a complete transaction with balanced debits and credits';
COMMENT ON TABLE journal_entry_lines IS 'Individual debit/credit lines that compose journal entries - implements double-entry bookkeeping';
COMMENT ON COLUMN journal_entry_lines.debit IS 'Debit amount - increases assets/expenses, decreases liabilities/revenue';
COMMENT ON COLUMN journal_entry_lines.credit IS 'Credit amount - decreases assets/expenses, increases liabilities/revenue';
COMMENT ON VIEW account_balances IS 'Real-time account balances calculated from journal entries using double-entry bookkeeping';

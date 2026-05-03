-- Migration: complete EXP-003 Decimal cascade — expenses.amount FLOAT8 → NUMERIC(12,2)
--
-- Surfacé par PR #450 (chore/ci-gitflow-alignment) — BDD ColumnDecode panics
-- sur Expense entity avec `amount: Decimal` lecture FLOAT8 column.
--
-- Précision NUMERIC(12,2) = 10 chiffres avant virgule + 2 après → 9_999_999_999.99 max
-- Aligné sur journal_entry_lines.{debit,credit} et amount_excl_vat/amount_incl_vat
-- déjà en NUMERIC(10,2). On utilise (12,2) car expenses.amount peut représenter
-- des montants > 9_999_999.99 (gros travaux copro).
--
-- Aucune dépendance view/trigger sur expenses.amount (vérifié via
-- information_schema.view_column_usage).
--
-- ADR : 0007 (Decimal vs f64), 0008 (NUMERIC vs DOUBLE).

BEGIN;

-- 1. Drop CHECK constraint qui référence "0::double precision" (incompatible NUMERIC)
ALTER TABLE expenses DROP CONSTRAINT IF EXISTS expenses_amount_check;

-- 2. ALTER type
ALTER TABLE expenses
    ALTER COLUMN amount TYPE NUMERIC(12,2)
    USING amount::NUMERIC(12,2);

-- 3. Re-add CHECK avec littéral NUMERIC
ALTER TABLE expenses
    ADD CONSTRAINT expenses_amount_check CHECK (amount > 0);

COMMIT;

COMMENT ON COLUMN expenses.amount IS 'Montant total TTC en EUR (NUMERIC(12,2) — précision exacte requise pour comptabilité belge PCMN — ADR-0007/0008)';

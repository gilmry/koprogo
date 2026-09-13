ALTER TABLE charge_distributions DROP CONSTRAINT IF EXISTS unique_expense_unit_owner;
ALTER TABLE charge_distributions ADD CONSTRAINT unique_expense_unit UNIQUE (expense_id, unit_id);

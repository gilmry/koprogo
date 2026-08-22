ALTER TABLE quotes DROP COLUMN work_category;

ALTER TABLE quotes
    ALTER COLUMN amount_excl_vat SET NOT NULL,
    ALTER COLUMN vat_rate SET NOT NULL,
    ALTER COLUMN amount_incl_vat SET NOT NULL,
    ALTER COLUMN validity_date SET NOT NULL,
    ALTER COLUMN estimated_duration_days SET NOT NULL;

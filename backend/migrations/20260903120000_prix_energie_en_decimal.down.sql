-- Retour à la virgule flottante : perd de la précision sur les prix déjà
-- enregistrés, ce qui est précisément le défaut que la migration corrige.
ALTER TABLE provider_offers
    ALTER COLUMN price_kwh_electricity TYPE DOUBLE PRECISION,
    ALTER COLUMN price_kwh_gas TYPE DOUBLE PRECISION,
    ALTER COLUMN fixed_monthly_fee TYPE DOUBLE PRECISION;

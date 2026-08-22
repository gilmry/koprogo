-- Down — Story H12 : retrait du critère de répartition.

ALTER TABLE charge_distributions
    DROP CONSTRAINT IF EXISTS valid_distribution_criteria;

ALTER TABLE charge_distributions
    DROP COLUMN IF EXISTS distribution_criteria;

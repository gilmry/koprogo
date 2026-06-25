-- Down — Story H13 : retrait des fonds réserve/roulement de l'ACP.

ALTER TABLE acps
    DROP CONSTRAINT IF EXISTS non_negative_reserve_fund,
    DROP CONSTRAINT IF EXISTS non_negative_working_capital;

ALTER TABLE acps
    DROP COLUMN IF EXISTS reserve_fund_balance,
    DROP COLUMN IF EXISTS working_capital_balance,
    DROP COLUMN IF EXISTS reserve_fund_waived;
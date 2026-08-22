-- Story H13 (Track H, CL4) — fonds de réserve & de roulement de l'ACP.
--
-- Art. 3.86 §3 Code civil (loi du 18/06/2018, en vigueur 2019) : l'ACP doit
-- constituer un fonds de réserve (≥ 5 % des charges ordinaires N-1, renonçable
-- 4/5) et un fonds de roulement, sur des comptes DISTINCTS au nom de l'ACP.
-- On modélise les soldes + l'éventuelle renonciation. Cf. ADR-0012.
--
-- Rétro-compatible : NOT NULL DEFAULT 0 / false → les ACPs existantes prennent
-- des soldes nuls et réserve non renoncée (comportement conservateur).

ALTER TABLE acps
    ADD COLUMN reserve_fund_balance NUMERIC(14, 2) NOT NULL DEFAULT 0,
    ADD COLUMN working_capital_balance NUMERIC(14, 2) NOT NULL DEFAULT 0,
    ADD COLUMN reserve_fund_waived BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE acps
    ADD CONSTRAINT non_negative_reserve_fund CHECK (reserve_fund_balance >= 0),
    ADD CONSTRAINT non_negative_working_capital CHECK (working_capital_balance >= 0);

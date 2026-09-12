-- Le syndic doit dire, à l'appel de fonds, quelle part alimente le fonds de
-- réserve.
--
-- Art. 3.86 § 3, alinéa 7 : « Le syndic communique à toutes les parties
-- concernées LORS DE L'APPEL DE FONDS quelle part sera affectée au fonds de
-- réserve. »
--
-- Ce n'est pas une formalité d'affichage. Le fonds de réserve est la part
-- qu'un copropriétaire ne récupère pas en vendant son lot : elle suit le lot,
-- pas le vendeur. Il doit donc savoir ce qu'il y verse au moment où on le lui
-- réclame, et non le découvrir à la mutation.
--
-- Défaut à zéro pour l'existant : une part nulle est licite — toutes les
-- charges n'alimentent pas le fonds — et devient au moins explicite.

ALTER TABLE call_for_funds
    ADD COLUMN IF NOT EXISTS reserve_fund_share NUMERIC(12, 2) NOT NULL DEFAULT 0;

ALTER TABLE call_for_funds
    ADD CONSTRAINT chk_call_for_funds_reserve_share
    CHECK (reserve_fund_share >= 0 AND reserve_fund_share <= total_amount);

COMMENT ON COLUMN call_for_funds.reserve_fund_share IS
    'Part du montant appelé affectée au fonds de réserve (Art. 3.86 § 3 al. 7). Bornée par le total.';

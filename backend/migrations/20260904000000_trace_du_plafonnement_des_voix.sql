-- Art. 3.87 § 7 al. 4 — la trace du plafonnement des voix.
--
-- « Nul ne peut prendre part au vote, même comme mandant ou mandataire, pour
-- un nombre de voix supérieur à la somme des voix dont disposent les autres
-- copropriétaires présents ou représentés. »
--
-- Arbitrage humain du 2026-09-04 : le texte plafonne, il n'annule pas. Le
-- majoritaire est ramené au poids des autres et l'assemblée délibère sur ce
-- décompte corrigé.
--
-- L'écart est conservé parce que l'ACP doit pouvoir en répondre. Si la
-- décision est attaquée, il faut montrer que la règle a été appliquée, et de
-- combien. Un plafonnement silencieux serait indéfendable : le procès-verbal
-- afficherait un décompte que rien dans les bulletins ne permet de retrouver.
--
-- `jsonb` et non des colonnes : le nombre de votants plafonnés n'est pas borné
-- par le schéma, et cette donnée se lit, ne se filtre pas.
--
-- NULL = aucun plafonnement appliqué, ce qui est le cas ordinaire. Un tableau
-- vide voudrait dire la même chose ; NULL le dit sans ambiguïté.
ALTER TABLE resolutions
    ADD COLUMN voix_plafonnees jsonb;

COMMENT ON COLUMN resolutions.voix_plafonnees IS
    'Art. 3.87 § 7 al. 4 : écarts entre voix brutes et voix retenues, par votant plafonné. NULL si aucun.';

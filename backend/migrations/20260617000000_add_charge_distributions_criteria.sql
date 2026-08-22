-- Story H12 (Track H, CL4) — critère légal de répartition des charges.
--
-- Art. 3.84 / 3.86 Code civil : les charges communes se répartissent selon la
-- valeur (quote-part / tantièmes), l'utilité, ou un critère mixte. On enregistre
-- sous quel critère chaque ligne de répartition a été calculée (traçabilité).
--
-- Rétro-compatible : NOT NULL DEFAULT 'value' → les lignes existantes prennent
-- le critère par défaut (valeur), conforme au comportement antérieur.

ALTER TABLE charge_distributions
    ADD COLUMN distribution_criteria TEXT NOT NULL DEFAULT 'value';

ALTER TABLE charge_distributions
    ADD CONSTRAINT valid_distribution_criteria
    CHECK (distribution_criteria IN ('value', 'utility', 'mixed'));

-- Track H Conformité légale — Story H4 (WP-CL1) — ADR-0010
-- Acte de base au niveau ACP : `acps.total_tantiemes` devient la source de
-- vérité du dénominateur des quotités (1000 millièmes / 10000 dix-millièmes /
-- autre selon l'acte authentique — Art. 3.84 CC). `buildings.total_tantiemes`
-- est conservé mais redéfini comme sous-total de quotités du bloc.

ALTER TABLE acps
    ADD COLUMN total_tantiemes INTEGER NOT NULL DEFAULT 1000
    CHECK (total_tantiemes > 0);

COMMENT ON COLUMN acps.total_tantiemes IS
    'Denominateur de l''acte de base (quotites) — 1000/10000/autre. Art. 3.84 CC. '
    'Source de verite de la copropriete (ACP) ; buildings.total_tantiemes = sous-total bloc. ADR-0010.';

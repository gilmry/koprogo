-- Le budget prévisionnel appartient à l'ACP, pas au syndic qui le prépare.
--
-- Art. 3.89 § 5, 16° : le syndic est chargé « de préparer le budget
-- prévisionnel [...] ; ces budgets prévisionnels sont soumis, chaque année, au
-- vote de l'association des copropriétaires ». Il prépare ; elle vote et
-- supporte. Le mandat change au plus tous les trois ans (Art. 3.89 § 1er), le
-- budget reste. Cf. ADR-0045.
--
-- `organization_id` est conservé comme trace de l'auteur, et retiré de tout
-- prédicat d'autorisation : le périmètre d'un syndic se dérive de son mandat.

ALTER TABLE budgets ADD COLUMN IF NOT EXISTS acp_id UUID;

-- Reprise : l'ACP se déduit de l'immeuble, lien stable qui ne change qu'avec
-- une modification de l'acte de base.
UPDATE budgets b
SET acp_id = bl.acp_id
FROM buildings bl
WHERE b.building_id = bl.id
  AND b.acp_id IS NULL;

-- Un budget orphelin d'ACP serait un budget que personne ne vote : on refuse
-- plutôt que de laisser passer une ligne muette.
DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM budgets WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % budget(s) sans ACP résoluble depuis leur immeuble', orphelins;
    END IF;
END $$;

ALTER TABLE budgets ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE budgets
    ADD CONSTRAINT fk_budgets_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_budgets_acp_id ON budgets (acp_id);

COMMENT ON COLUMN budgets.acp_id IS
    'ACP propriétaire du budget (Art. 3.89 § 5, 16°). Porte le cloisonnement.';
COMMENT ON COLUMN budgets.organization_id IS
    'Syndic auteur, trace uniquement. Jamais un prédicat d''autorisation (ADR-0045).';

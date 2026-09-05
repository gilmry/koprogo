-- L'appel de fonds est lancé au nom de l'ACP, pas du syndic qui l'émet.
--
-- Art. 3.86 § 3 : le patrimoine de l'ACP « est constitué par des apports
-- périodiques des copropriétaires décidés par l'assemblée générale ». Le
-- syndic lance l'appel et peut prendre les mesures de recouvrement ; la
-- créance appartient à l'ACP. Le mandat s'éteint au plus tous les trois ans
-- (Art. 3.89 § 1er), la créance reste. Cf. ADR-0045.

ALTER TABLE call_for_funds ADD COLUMN IF NOT EXISTS acp_id UUID;

UPDATE call_for_funds c
SET acp_id = b.acp_id
FROM buildings b
WHERE c.building_id = b.id
  AND c.acp_id IS NULL;

-- Un appel de fonds sans ACP est un appel dont on ignore le créancier.
DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM call_for_funds WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % appel(s) de fonds sans ACP résoluble depuis leur immeuble', orphelins;
    END IF;
END $$;

ALTER TABLE call_for_funds ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE call_for_funds
    ADD CONSTRAINT fk_call_for_funds_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_call_for_funds_acp_id ON call_for_funds (acp_id);

COMMENT ON COLUMN call_for_funds.acp_id IS
    'ACP créancière (Art. 3.86 § 3). Porte le cloisonnement.';
COMMENT ON COLUMN call_for_funds.organization_id IS
    'Syndic émetteur, trace uniquement. Jamais un prédicat d''autorisation (ADR-0045).';

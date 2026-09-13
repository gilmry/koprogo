-- La convocation est un acte de l'ACP, dont elle supporte les frais.
--
-- Art. 3.87 § 3 : « Les frais administratifs afférents à la convocation à
-- l'assemblée générale sont à charge de l'association des copropriétaires ».
-- Le syndic l'envoie, l'ACP la paie et la conserve. Cf. ADR-0045.

ALTER TABLE convocations ADD COLUMN IF NOT EXISTS acp_id UUID;

UPDATE convocations t
SET acp_id = b.acp_id
FROM buildings b
WHERE t.building_id = b.id
  AND t.acp_id IS NULL;

DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM convocations WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % ligne(s) de convocations sans ACP résoluble depuis leur immeuble',
            orphelins;
    END IF;
END $$;

ALTER TABLE convocations ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE convocations
    ADD CONSTRAINT fk_convocations_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_convocations_acp_id ON convocations (acp_id);

COMMENT ON COLUMN convocations.acp_id IS 'ACP dont les copropriétaires sont convoqués (Art. 3.87 § 3). Porte le cloisonnement.';
COMMENT ON COLUMN convocations.organization_id IS
    'Syndic auteur, trace uniquement. Jamais un prédicat d''''autorisation (ADR-0045).';

-- L'état daté atteste d'une situation vis-à-vis de l'ACP, pas du syndic.
--
-- Art. 3.94 : à la transmission d'un lot, le notaire réclame au syndic l'état
-- de la situation du copropriétaire cédant vis-à-vis de l'association. Les
-- sommes attestées sont dues à l'ACP ; le syndic les certifie. Cf. ADR-0045.

ALTER TABLE etats_dates ADD COLUMN IF NOT EXISTS acp_id UUID;

UPDATE etats_dates t
SET acp_id = b.acp_id
FROM buildings b
WHERE t.building_id = b.id
  AND t.acp_id IS NULL;

DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM etats_dates WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % ligne(s) de etats_dates sans ACP résoluble depuis leur immeuble',
            orphelins;
    END IF;
END $$;

ALTER TABLE etats_dates ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE etats_dates
    ADD CONSTRAINT fk_etats_dates_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_etats_dates_acp_id ON etats_dates (acp_id);

COMMENT ON COLUMN etats_dates.acp_id IS 'ACP dont l''état daté atteste la situation (Art. 3.94). Porte le cloisonnement.';
COMMENT ON COLUMN etats_dates.organization_id IS
    'Syndic auteur, trace uniquement. Jamais un prédicat d''''autorisation (ADR-0045).';

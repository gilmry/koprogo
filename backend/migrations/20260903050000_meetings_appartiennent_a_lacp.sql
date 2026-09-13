-- L'assemblée générale est l'organe de l'ACP, pas une réunion du cabinet.
--
-- Art. 3.87 § 1er : « Chaque propriétaire d'un lot fait partie de l'assemblée
-- générale ». Le syndic la tient (§ 2), il ne la possède pas. Un changement de
-- syndic ne rend pas les assemblées passées invisibles. Cf. ADR-0045.

ALTER TABLE meetings ADD COLUMN IF NOT EXISTS acp_id UUID;

UPDATE meetings t
SET acp_id = b.acp_id
FROM buildings b
WHERE t.building_id = b.id
  AND t.acp_id IS NULL;

DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM meetings WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % ligne(s) de meetings sans ACP résoluble depuis leur immeuble',
            orphelins;
    END IF;
END $$;

ALTER TABLE meetings ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE meetings
    ADD CONSTRAINT fk_meetings_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_meetings_acp_id ON meetings (acp_id);

COMMENT ON COLUMN meetings.acp_id IS 'ACP dont c''est l''assemblée (Art. 3.87). Porte le cloisonnement.';
COMMENT ON COLUMN meetings.organization_id IS
    'Syndic auteur, trace uniquement. Jamais un prédicat d''''autorisation (ADR-0045).';

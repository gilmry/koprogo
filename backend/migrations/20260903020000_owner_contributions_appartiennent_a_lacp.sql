-- La quote-part est due à l'ACP, pas au cabinet qui l'a émise.
--
-- Art. 3.86 § 3 : les apports périodiques des copropriétaires constituent le
-- patrimoine de l'association. Ce que doit un copropriétaire, il le doit à sa
-- copropriété. Le syndic recouvre pour elle, il n'est pas créancier.
-- Cf. ADR-0045.

ALTER TABLE owner_contributions ADD COLUMN IF NOT EXISTS acp_id UUID;

-- Deux chemins de reprise, du plus sûr au plus indirect.
-- 1. par l'appel de fonds, qui porte déjà son ACP
UPDATE owner_contributions oc
SET acp_id = c.acp_id
FROM call_for_funds c
WHERE oc.call_for_funds_id = c.id
  AND oc.acp_id IS NULL;

-- 2. par le lot : lot -> immeuble -> ACP, chemin fixé par l'acte de base
UPDATE owner_contributions oc
SET acp_id = b.acp_id
FROM units u
JOIN buildings b ON b.id = u.building_id
WHERE oc.unit_id = u.id
  AND oc.acp_id IS NULL;

DO $$
DECLARE orphelins INT;
BEGIN
    SELECT COUNT(*) INTO orphelins FROM owner_contributions WHERE acp_id IS NULL;
    IF orphelins > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % quote(s)-part(s) sans ACP créancière résoluble', orphelins;
    END IF;
END $$;

ALTER TABLE owner_contributions ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE owner_contributions
    ADD CONSTRAINT fk_owner_contributions_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_owner_contributions_acp_id ON owner_contributions (acp_id);

COMMENT ON COLUMN owner_contributions.acp_id IS
    'ACP créancière de la quote-part (Art. 3.86 § 3). Porte le cloisonnement.';

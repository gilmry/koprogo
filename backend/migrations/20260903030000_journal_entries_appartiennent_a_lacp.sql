-- Le grand livre est celui de l'ACP, pas du syndic qui le tient.
--
-- Art. 3.89 § 5, 15° : le syndic est chargé « de tenir les comptes de
-- l'association des copropriétaires de manière claire, précise et détaillée
-- suivant le plan comptable minimum normalisé à établir par le Roi ». Il les
-- tient ; ils ne sont pas les siens. Et l'Art. 3.89 § 5, 7° impose de les
-- transmettre au successeur sous trente jours. Cf. ADR-0045.
--
-- La colonne est NOT NULL alors que `building_id` reste facultative : une ACP
-- peut regrouper plusieurs immeubles (Art. 3.84) et passer des écritures qui
-- ne se rattachent à aucun, mais aucune écriture n'existe hors d'une
-- comptabilité.

ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS acp_id UUID;

-- Trois chemins de reprise, du plus direct au plus indirect.
-- 1. par la dépense qui a produit l'écriture
UPDATE journal_entries je
SET acp_id = e.acp_id
FROM expenses e
WHERE je.expense_id = e.id
  AND je.acp_id IS NULL;

-- 2. par la quote-part encaissée
UPDATE journal_entries je
SET acp_id = oc.acp_id
FROM owner_contributions oc
WHERE je.contribution_id = oc.id
  AND je.acp_id IS NULL;

-- 3. par l'immeuble, quand l'écriture en désigne un
UPDATE journal_entries je
SET acp_id = b.acp_id
FROM buildings b
WHERE je.building_id = b.id
  AND je.acp_id IS NULL;

-- Une écriture sans ACP est une écriture dans les livres de personne.
DO $$
DECLARE orphelines INT;
BEGIN
    SELECT COUNT(*) INTO orphelines FROM journal_entries WHERE acp_id IS NULL;
    IF orphelines > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % écriture(s) sans ACP résoluble (ni dépense, ni quote-part, ni immeuble)',
            orphelines;
    END IF;
END $$;

ALTER TABLE journal_entries ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE journal_entries
    ADD CONSTRAINT fk_journal_entries_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_journal_entries_acp_id ON journal_entries (acp_id);

COMMENT ON COLUMN journal_entries.acp_id IS
    'ACP dont ces comptes sont les comptes (Art. 3.89 § 5, 15°). Porte le cloisonnement.';

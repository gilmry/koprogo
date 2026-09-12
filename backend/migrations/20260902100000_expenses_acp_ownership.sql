-- Rattache les charges à l'ACP qui les possède.
--
-- POURQUOI
--   Une ACP est une entité juridique à part entière : elle a son numéro BCE,
--   son acte de base, ses lots et SA COMPTABILITÉ. Un syndic n'en est que le
--   mandataire, désigné et révocable par l'assemblée générale (Art. 3.89 CC).
--
--   `expenses` ne portait que `organization_id` — le cabinet syndic. Mesuré le
--   2026-09-02 sur une passation réelle : après changement de syndic, le
--   cabinet entrant héritait d'une copropriété sans grand livre, sans budget
--   et sans copropriétaires, tandis que le cabinet sortant continuait de les
--   voir. Une estampille figée à la saisie ne peut pas répondre à « qui a le
--   droit de voir cette charge aujourd'hui ».
--
-- DEUX FAITS DISTINCTS, DEUX COLONNES
--   `acp_id`          — À QUI la charge appartient. Clé de portée. Suit l'ACP
--                       lors des passations.
--   `organization_id` — QUI l'a encodée. Traçabilité, et responsabilité du
--                       cabinet qui a saisi. Ne doit JAMAIS servir de critère
--                       de portée.
--
-- REPRISE
--   L'ACP se déduit de l'immeuble, qui la porte déjà (`buildings.acp_id`).
--   La reprise est donc exacte et sans perte : aucune charge n'existe sans
--   immeuble (`building_id` est NOT NULL).

ALTER TABLE expenses ADD COLUMN IF NOT EXISTS acp_id UUID;

UPDATE expenses e
   SET acp_id = b.acp_id
  FROM buildings b
 WHERE e.building_id = b.id
   AND e.acp_id IS DISTINCT FROM b.acp_id;

-- Garde-fou : aucune charge ne doit rester sans ACP après reprise.
DO $$
DECLARE orphelines bigint;
BEGIN
    SELECT count(*) INTO orphelines FROM expenses WHERE acp_id IS NULL;
    IF orphelines > 0 THEN
        RAISE EXCEPTION
            'Reprise incomplète : % charge(s) sans ACP. Vérifier les immeubles orphelins avant de poursuivre.',
            orphelines;
    END IF;
END $$;

ALTER TABLE expenses ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE expenses DROP CONSTRAINT IF EXISTS fk_expenses_acp;
ALTER TABLE expenses
    ADD CONSTRAINT fk_expenses_acp
        FOREIGN KEY (acp_id) REFERENCES acps(id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_expenses_acp_id ON expenses(acp_id);

COMMENT ON COLUMN expenses.acp_id IS
    'ACP proprietaire de la charge. Cle de portee : suit la copropriete lors des passations de syndic.';
COMMENT ON COLUMN expenses.organization_id IS
    'Cabinet syndic ayant ENCODE la charge (tracabilite). Ne pas utiliser comme critere de portee.';

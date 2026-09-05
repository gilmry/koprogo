-- La relance réclame une somme due à l'ACP, pas au syndic qui l'envoie.
--
-- Art. 3.86 § 3 : « Le syndic peut prendre toutes les mesures judiciaires et
-- extrajudiciaires pour la récupération des charges ». Il recouvre POUR
-- l'association ; la créance et les pénalités de retard lui reviennent. Une
-- relance qui survit au mandat continue de désigner la bonne créancière.
-- Cf. ADR-0045.

ALTER TABLE payment_reminders ADD COLUMN IF NOT EXISTS acp_id UUID;

-- La relance porte la dépense impayée, qui porte déjà son ACP.
UPDATE payment_reminders pr
SET acp_id = e.acp_id
FROM expenses e
WHERE pr.expense_id = e.id
  AND pr.acp_id IS NULL;

DO $$
DECLARE orphelines INT;
BEGIN
    SELECT COUNT(*) INTO orphelines FROM payment_reminders WHERE acp_id IS NULL;
    IF orphelines > 0 THEN
        RAISE EXCEPTION
            'Reprise impossible : % relance(s) sans ACP créancière résoluble depuis leur dépense',
            orphelines;
    END IF;
END $$;

ALTER TABLE payment_reminders ALTER COLUMN acp_id SET NOT NULL;

ALTER TABLE payment_reminders
    ADD CONSTRAINT fk_payment_reminders_acp FOREIGN KEY (acp_id) REFERENCES acps (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_payment_reminders_acp_id ON payment_reminders (acp_id);

COMMENT ON COLUMN payment_reminders.acp_id IS
    'ACP créancière de la somme réclamée (Art. 3.86 § 3). Porte le cloisonnement.';

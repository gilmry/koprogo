-- Rattache un paiement a la contribution qu'il solde.
--
-- POURQUOI
--   `payments` ne portait que `expense_id` : un paiement pouvait designer la
--   facture fournisseur qu'il regle, mais rien ne le reliait a la quote-part
--   d'un coproprietaire. Enregistrer un paiement laissait donc la contribution
--   correspondante en `pending` indefiniment, et le syndic n'avait aucun moyen
--   de savoir qui avait paye autrement qu'en pointant a la main.
--
--   Un `POST /payments` portant `contribution_id` repondait 201 en jetant le
--   champ : serde ignore les champs inconnus par defaut. La perte etait
--   silencieuse (rapport de test du 2026-09-01, constat F4).
--
-- CE QUE CELA NE FAIT PAS
--   Le rattachement ne solde rien a lui seul. Un paiement naissant `pending`
--   est une INTENTION (le module gere Stripe : 3DS, SEPA, webhooks). La
--   contribution n'est marquee payee qu'a la transition vers `succeeded`,
--   c'est-a-dire quand l'argent est effectivement arrive.
--
-- ON DELETE SET NULL
--   Aligne sur `fk_payments_expense`. La trace comptable du paiement doit
--   survivre a la disparition de la contribution : perdre le lien est
--   acceptable, perdre le mouvement d'argent ne l'est pas.

ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS contribution_id UUID;

ALTER TABLE payments
    DROP CONSTRAINT IF EXISTS fk_payments_contribution;

ALTER TABLE payments
    ADD CONSTRAINT fk_payments_contribution
        FOREIGN KEY (contribution_id) REFERENCES owner_contributions(id) ON DELETE SET NULL;

-- Partiel : la grande majorite des paiements n'a pas de contribution liee.
CREATE INDEX IF NOT EXISTS idx_payments_contribution_id
    ON payments(contribution_id) WHERE contribution_id IS NOT NULL;

COMMENT ON COLUMN payments.contribution_id IS
    'Quote-part de coproprietaire soldee par ce paiement. La contribution passe a `paid` quand le paiement atteint `succeeded`, pas a sa creation.';

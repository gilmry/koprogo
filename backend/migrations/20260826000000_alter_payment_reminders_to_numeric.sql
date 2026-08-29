-- Migration: ADR-0008 — payment_reminders montants DOUBLE PRECISION -> NUMERIC
-- Suite de #661 : dette f64 monétaire identifiée par le gate
-- `scripts/check-no-f64-money.sh` et gelée dans son allowlist.
--
-- Ancrage légal : les pénalités de retard sont calculées au taux légal civil
-- belge (4,5 % annuel en 2026, Arrêté Royal publié au Moniteur belge) sur des
-- sommes RÉCLAMÉES À UN COPROPRIÉTAIRE, jusqu'à la lettre recommandée et
-- l'huissier. Un montant opposable ne se calcule pas en IEEE 754.
--
-- Deux défauts sont corrigés ici, pas un :
--
--   1. `DOUBLE PRECISION` sur trois montants en euros (ADR-0007/0008 §A).
--
--   2. `CHECK (total_amount = amount_owed + penalty_amount)` : une égalité
--      flottante EXACTE. En binary64, `owed + penalty` peut différer du
--      `total` calculé côté application au dernier bit, et la contrainte
--      rejette alors une ligne parfaitement valide — un échec d'insertion
--      non reproductible, dépendant des valeurs. En NUMERIC, l'égalité est
--      exacte par construction et la contrainte devient fiable.
--
-- Forward-only, idempotente (convention projet : pas de fichier `down`).
--
-- Effet sur d'éventuelles données existantes : le cast arrondit à 2 décimales,
-- ce qui est la précision voulue pour des euros. `total_amount` est ensuite
-- recalculé depuis ses deux composantes pour que la contrainte d'égalité
-- reste vraie même si les trois arrondis ont divergé — d'où l'ordre
-- DROP CONSTRAINT -> ALTER -> UPDATE -> ADD CONSTRAINT : PostgreSQL revalide
-- les CHECK pendant l'ALTER, et le faire dans l'autre sens ferait échouer la
-- migration sur des lignes que ce même UPDATE aurait réparées.

-- 1. Retirer les CHECK portant sur les trois colonnes, le temps de la
--    conversion.
--
--    Les deux CHECK anonymes de la table d'origine ont reçu de PostgreSQL les
--    noms `payment_reminders_check` (total >= owed) et
--    `payment_reminders_check2` (total = owed + penalty) — vérifié sur une base
--    reconstruite depuis zéro. Les viser par un nom inventé les laisserait en
--    place : l'ALTER échouerait alors sur toute ligne que l'UPDATE de l'étape 3
--    est justement censé réparer.
--
--    Les CHECK de positivité sont retirés puis recréés : un `ALTER COLUMN TYPE`
--    ne réécrit pas l'expression d'un CHECK existant, il y insère un cast vers
--    l'ancien type. `CHECK (amount_owed > 0)` devient donc
--    `CHECK (amount_owed::double precision > 0::double precision)` — la
--    positivité continuerait d'être évaluée en flottant après la migration,
--    exactement ce que celle-ci vient supprimer.
ALTER TABLE payment_reminders
    DROP CONSTRAINT IF EXISTS payment_reminders_check,
    DROP CONSTRAINT IF EXISTS payment_reminders_check2,
    DROP CONSTRAINT IF EXISTS payment_reminders_total_amount_check,
    DROP CONSTRAINT IF EXISTS payment_reminders_total_gte_owed_check,
    DROP CONSTRAINT IF EXISTS payment_reminders_amount_owed_check,
    DROP CONSTRAINT IF EXISTS payment_reminders_penalty_amount_check;

-- 2. Convertir les trois montants. NUMERIC(12,2) : jusqu'à
--    9 999 999 999,99 € — largement au-delà de toute charge de copropriété.
ALTER TABLE payment_reminders
    ALTER COLUMN amount_owed TYPE NUMERIC(12, 2) USING amount_owed::NUMERIC(12, 2),
    ALTER COLUMN penalty_amount TYPE NUMERIC(12, 2) USING penalty_amount::NUMERIC(12, 2),
    ALTER COLUMN total_amount TYPE NUMERIC(12, 2) USING total_amount::NUMERIC(12, 2);

ALTER TABLE payment_reminders
    ALTER COLUMN penalty_amount SET DEFAULT 0.00;

-- 3. Re-synchroniser le total après arrondi (no-op si déjà cohérent).
UPDATE payment_reminders
SET total_amount = amount_owed + penalty_amount
WHERE total_amount <> amount_owed + penalty_amount;

-- 4. Rétablir les contraintes, toutes évaluées en NUMERIC.
ALTER TABLE payment_reminders
    ADD CONSTRAINT payment_reminders_amount_owed_check
        CHECK (amount_owed > 0),
    ADD CONSTRAINT payment_reminders_penalty_amount_check
        CHECK (penalty_amount >= 0),
    ADD CONSTRAINT payment_reminders_total_gte_owed_check
        CHECK (total_amount >= amount_owed),
    ADD CONSTRAINT payment_reminders_total_amount_check
        CHECK (total_amount = amount_owed + penalty_amount);

COMMENT ON COLUMN payment_reminders.amount_owed IS
    'Montant du en euros. NUMERIC(12,2) exact (ADR-0007/0008).';
COMMENT ON COLUMN payment_reminders.penalty_amount IS
    'Penalites de retard au taux legal civil belge (4,5% annuel en 2026, cf. BELGIAN_PENALTY_RATE dans payment_reminder.rs). NUMERIC(12,2) exact (ADR-0008).';
COMMENT ON COLUMN payment_reminders.total_amount IS
    'amount_owed + penalty_amount. NUMERIC(12,2) — egalite verifiee par CHECK, exacte depuis le passage en NUMERIC.';

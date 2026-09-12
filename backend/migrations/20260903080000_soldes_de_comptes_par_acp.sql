-- Les soldes de comptes se calculent par ACP, pas par syndic.
--
-- La vue `account_balances` groupait sur `journal_entry_lines.organization_id`.
-- Deux conséquences, et la seconde est la plus grave :
--
-- 1. un cabinet gérant cinq ACP obtenait **un seul bilan pour les cinq**, alors
--    que chaque ACP a son propre patrimoine (Art. 3.86 § 3) et ses propres
--    comptes, tenus séparément (Art. 3.89 § 5, 15°) ;
--
-- 2. après une passation, les écritures d'une même ACP portent deux
--    organisations différentes — celle du syndic sortant et celle de l'entrant.
--    Le groupement par organisation **coupait donc le solde d'une ACP en deux**
--    au moment précis où l'Art. 3.89 § 5, 7° exige que le dossier soit transmis
--    entier. Le successeur reprenait des comptes amputés de tout l'historique.
--
-- La vue groupe désormais sur l'ACP de l'écriture. `organization_id` disparaît
-- de l'agrégat : une somme d'écritures posées par deux mandataires successifs
-- n'a pas d'auteur unique, et la question « qui a passé cette écriture » se
-- pose au niveau de l'écriture, pas du solde.
--
-- Cf. ADR-0045.

DROP VIEW IF EXISTS account_balances;

CREATE VIEW account_balances AS
SELECT
    je.acp_id,
    jel.account_code,
    a.label as account_label,
    a.account_type,
    SUM(jel.debit) as total_debit,
    SUM(jel.credit) as total_credit,
    -- Actifs et charges : le débit augmente le solde.
    -- Passifs et produits : le crédit l'augmente.
    CASE
        WHEN a.account_type IN ('ASSET', 'EXPENSE') THEN SUM(jel.debit) - SUM(jel.credit)
        WHEN a.account_type IN ('LIABILITY', 'REVENUE') THEN SUM(jel.credit) - SUM(jel.debit)
        ELSE 0
    END as balance
FROM journal_entry_lines jel
JOIN journal_entries je ON je.id = jel.journal_entry_id
-- Le plan comptable reste rattaché à l'organisation. C'est une limite connue :
-- l'AR du 12/07/2012 fixe le plan comptable minimum normalisé DE L'ACP, donc
-- `accounts` devrait suivre l'ACP comme le reste. Tant que ce n'est pas fait,
-- la jointure résout le libellé via l'organisation qui a posé la ligne, ce qui
-- reste correct ligne à ligne.
JOIN accounts a ON a.organization_id = jel.organization_id AND a.code = jel.account_code
GROUP BY je.acp_id, jel.account_code, a.label, a.account_type;

COMMENT ON VIEW account_balances IS
    'Soldes de comptes par ACP, calculés en partie double. Un solde appartient à une copropriété, jamais à son syndic (ADR-0045).';

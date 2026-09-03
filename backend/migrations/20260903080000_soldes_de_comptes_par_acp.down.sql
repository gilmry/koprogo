DROP VIEW IF EXISTS account_balances;

CREATE VIEW account_balances AS
SELECT
    jel.organization_id,
    jel.account_code,
    a.label as account_label,
    a.account_type,
    SUM(jel.debit) as total_debit,
    SUM(jel.credit) as total_credit,
    CASE
        WHEN a.account_type IN ('ASSET', 'EXPENSE') THEN SUM(jel.debit) - SUM(jel.credit)
        WHEN a.account_type IN ('LIABILITY', 'REVENUE') THEN SUM(jel.credit) - SUM(jel.debit)
        ELSE 0
    END as balance
FROM journal_entry_lines jel
JOIN accounts a ON a.organization_id = jel.organization_id AND a.code = jel.account_code
GROUP BY jel.organization_id, jel.account_code, a.label, a.account_type;

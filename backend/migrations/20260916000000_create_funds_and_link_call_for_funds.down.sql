-- Réversibilité de la migration des fonds.
--
-- L'ordre compte : la contrainte de clé étrangère part avec la colonne, et
-- la colonne AVANT la table qu'elle référence. L'inverse laisserait
-- `call_for_funds.fund_id` pointer vers une table disparue.
DROP INDEX IF EXISTS idx_call_for_funds_fund;
ALTER TABLE call_for_funds DROP COLUMN IF EXISTS fund_id;

DROP INDEX IF EXISTS idx_funds_acp;
DROP INDEX IF EXISTS idx_funds_un_seul_par_type_structurel;
DROP TABLE IF EXISTS funds;

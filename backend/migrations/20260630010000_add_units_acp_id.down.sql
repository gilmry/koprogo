-- Story H15 — DOWN Étape 1/3 : drop colonne acp_id + index
-- Rollback de 20260630010000_add_units_acp_id.sql.
--
-- Restaure l'état pré-Story-H15 : la colonne organization_id reste, l'index
-- idx_units_acp_id et la colonne acp_id disparaissent. Ne touche pas la table
-- `acps` elle-même (gérée par Story 1.1 / create_acps).

DROP INDEX IF EXISTS idx_units_acp_id;
ALTER TABLE units DROP COLUMN IF EXISTS acp_id;

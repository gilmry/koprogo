-- Story 1.2 — DOWN Étape 1/3 : drop colonne acp_id + index
-- Rollback de 20260601_020000_add_buildings_acp_id.sql.
--
-- Restaure l'état pré-Story-1.2 : la colonne organization_id reste, l'index
-- idx_buildings_acp_id et la colonne acp_id disparaissent. Ne touche pas la
-- table `acps` elle-même (gérée par Story 1.1 / 010000_create_acps_DOWN.sql).

DROP INDEX IF EXISTS idx_buildings_acp_id;
ALTER TABLE buildings DROP COLUMN IF EXISTS acp_id;

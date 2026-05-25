-- Rollback de la migration 20260601010000_create_acps.sql (Story 1.1).
--
-- À appliquer MANUELLEMENT en cas de rollback (la convention sqlx du projet
-- ne gère pas les migrations descendantes automatiquement).
--
-- Pré-condition : aucune table dépendante (`buildings.acp_id`, …) ne pointe
-- encore vers `acps`. La story 1.2 ajoutera `buildings.acp_id` ; si déjà
-- présent, ce rollback doit être précédé du rollback de 1.2.

DROP INDEX IF EXISTS idx_acps_slug;
DROP INDEX IF EXISTS idx_acps_organization_id;
DROP TABLE IF EXISTS acps;

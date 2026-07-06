-- Story H15 — DOWN Étape 2/3 : annule le backfill
-- Rollback de 20260630020000_backfill_units_acp_id.sql.
--
-- Démarche :
--   1. NULLIFY acp_id pour tous les lots (le UP n'a rempli que depuis le
--      building parent ; organization_id n'a jamais été touché par UP).
--   2. Supprime les audit_logs UnitAcpBackfilled.
--
-- Idempotent. Ne touche pas la colonne organization_id (encore présente à
-- cette étape — DROP arrive à 030000). Ne supprime aucune ACP (units ne crée
-- pas d'ACP miroir, contrairement à buildings).

UPDATE units SET acp_id = NULL WHERE acp_id IS NOT NULL;

DELETE FROM audit_logs WHERE event_type = 'UnitAcpBackfilled';

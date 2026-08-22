-- Story 1.2 — DOWN Étape 2/3 : annule le backfill
-- Rollback de 20260601_030000_backfill_buildings_acp_id.sql.
--
-- Démarche :
--   1. Pour chaque building dont acp_id pointe sur une ACP miroir créée
--      par la migration UP, NULLIFY acp_id (on garde organization_id qui
--      n'a jamais été touché par UP).
--   2. Supprime les ACPs miroirs (créées par UP) qui n'ont plus aucun
--      building rattaché. On les identifie par slug ('-mirror-<hex>')
--      pour ne pas supprimer des ACPs créées via l'API Story 1.1.
--   3. Supprime les audit_logs BuildingAcpBackfilled.
--
-- Idempotent. Ne touche pas la colonne organization_id (encore présente à
-- cette étape — DROP arrive à 040000).

-- Étape DOWN 1 : nullify acp_id pour tous les buildings (le UP n'a rempli
-- que ceux avec organization_id NOT NULL — donc on remet à NULL le résultat).
UPDATE buildings SET acp_id = NULL WHERE acp_id IS NOT NULL;

-- Étape DOWN 2 : supprime les ACPs miroirs (slug se termine par
-- '-mirror-<hex>'). On évite de toucher les ACPs créées par l'API
-- Story 1.1 (slug sans le suffixe '-mirror-').
DELETE FROM acps
WHERE slug LIKE '%-mirror-%'
  AND NOT EXISTS (
      SELECT 1 FROM buildings b WHERE b.acp_id = acps.id
  );

-- Étape DOWN 3 : audit log purge.
DELETE FROM audit_logs WHERE event_type = 'BuildingAcpBackfilled';

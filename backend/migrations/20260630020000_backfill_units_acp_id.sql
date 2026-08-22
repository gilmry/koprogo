-- Story H15 — Backfill data (Étape 2/3)
-- Source : architecture.md §1.1 (units.acp_id = building.acp_id).
--
-- Contrairement à buildings (Story 1.2 qui créait des ACPs miroirs depuis
-- organizations), units dérive son acp_id directement du building parent —
-- lequel a DÉJÀ son acp_id NOT NULL depuis la migration 20260601040000.
-- Un lot appartient toujours à la même ACP que son building.
--
-- Démarche :
--   1. UPDATE units.acp_id = building.acp_id (join via units.building_id).
--   2. INSERT audit_logs.event_type='UnitAcpBackfilled' par lot (idempotent).
--   3. Pré-check final : aucun lot avec acp_id NULL (sauf orphelin sans
--      building résolvable → @edge : la migration ÉCHOUE en le signalant).
--
-- Idempotence : le WHERE u.acp_id IS NULL évite de réécrire ; l'audit log
-- NOT EXISTS évite les doublons.
--
-- Rollback : `20260630020000_backfill_units_acp_id.down.sql`.

-- Étape 2.1 : backfill acp_id depuis le building parent.
UPDATE units u
SET acp_id = b.acp_id
FROM buildings b
WHERE u.building_id = b.id
  AND u.acp_id IS NULL;

-- Étape 2.2 : audit log par lot backfillé.
INSERT INTO audit_logs (
    id, timestamp, event_type, user_id, organization_id,
    resource_type, resource_id, success, created_at, metadata
)
SELECT
    gen_random_uuid(),
    now(),
    'UnitAcpBackfilled',
    NULL,
    u.organization_id,
    'Unit',
    u.id,
    true,
    now(),
    jsonb_build_object(
        'acp_id',           u.acp_id,
        'building_id',      u.building_id,
        'organization_id',  u.organization_id,
        'story',            'H15',
        'migration',        '20260630020000_backfill_units_acp_id'
    )
FROM units u
WHERE u.acp_id IS NOT NULL
  AND NOT EXISTS (
      SELECT 1 FROM audit_logs al
      WHERE al.event_type = 'UnitAcpBackfilled'
        AND al.resource_id = u.id
  );

-- Pré-check final (@edge) : aucun lot ne doit rester sans acp_id. Un lot
-- orphelin (building inexistant ou building sans acp_id — impossible
-- post-20260601040000) fait ÉCHOUER la migration en le signalant.
DO $$
DECLARE
    orphan_count INT;
BEGIN
    SELECT COUNT(*) INTO orphan_count
    FROM units
    WHERE acp_id IS NULL;
    IF orphan_count > 0 THEN
        RAISE EXCEPTION 'Backfill incomplet : % lot(s) orphelin(s) (acp_id NULL — building inexistant ou sans acp_id)', orphan_count;
    END IF;
END $$;

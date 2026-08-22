-- Story 1.2 — Étape 3/3 : ALTER acp_id SET NOT NULL + DROP organization_id
-- Source : docs/maury/refonte-ux-multi-role-acp/architecture.md §5.2
--
-- Pré-check : aucun building avec acp_id IS NULL. Si la migration 030000
-- a bien tourné, cette assertion passe ; sinon l'EXCEPTION fait échouer la
-- migration et préserve l'état (le SET NOT NULL n'est pas exécuté).
--
-- DROP organization_id : ATTENTION, casse le code qui réfère encore
-- `buildings.organization_id`. Story 1.3 doit refactorer ces refs vers
-- `acp_id` AVANT que cette migration soit appliquée en CI/prod. Les stories
-- 1.2+1.3+1.4 sont mergées ensemble sur dev (cf. notes du prompt).
--
-- Rollback : `20260601_040000_buildings_acp_id_not_null.down.sql`.

-- Pré-check assertion.
DO $$
DECLARE
    null_count INT;
BEGIN
    SELECT COUNT(*) INTO null_count FROM buildings WHERE acp_id IS NULL;
    IF null_count > 0 THEN
        RAISE EXCEPTION 'NOT NULL impossible : % buildings ont acp_id IS NULL. Re-jouer la migration 030000 backfill.', null_count;
    END IF;
END $$;

-- Étape 3.1 : NOT NULL.
ALTER TABLE buildings ALTER COLUMN acp_id SET NOT NULL;

-- Étape 3.2 : drop des index/policies/colonne organization_id.
-- Les policies RLS (cf. 20250103000000_add_organization_id_to_all_tables.sql)
-- référencent organization_id sur buildings : DROP la policy avant DROP la
-- colonne, sinon PostgreSQL refuse.
DROP POLICY IF EXISTS buildings_isolation ON buildings;

-- Les index composites organization_id + created_at doivent aussi sauter
-- avant DROP COLUMN.
DROP INDEX IF EXISTS idx_buildings_org_created;

ALTER TABLE buildings DROP COLUMN organization_id;

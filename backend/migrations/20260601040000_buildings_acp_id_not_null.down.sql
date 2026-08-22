-- Story 1.2 — DOWN Étape 3/3
-- Rollback de 20260601_040000_buildings_acp_id_not_null.sql.
--
-- Démarche :
--   1. Ré-ajoute organization_id NULLABLE (le NOT NULL viendra après backfill
--      inverse — on copie depuis acps.organization_id).
--   2. Backfill : building.organization_id = (SELECT organization_id FROM
--      acps WHERE id = building.acp_id).
--   3. Restore l'index composite idx_buildings_org_created.
--   4. Restore la RLS policy buildings_isolation.
--   5. ALTER acp_id DROP NOT NULL (acp_id redevient NULLABLE pour permettre
--      l'enchaînement avec le DOWN 030000 qui nullify acp_id).
--
-- Note RLS : la policy d'origine fait `USING (organization_id = current_setting(...)::UUID)`
-- mais la table buildings a ALTER TABLE ENABLE ROW LEVEL SECURITY puis
-- ENABLE/DISABLE selon 20250103000002_disable_rls_policies.sql. On restaure
-- la policy à l'identique de la migration 20250103000000.

ALTER TABLE buildings ADD COLUMN organization_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE;

UPDATE buildings b
SET organization_id = (
    SELECT a.organization_id FROM acps a WHERE a.id = b.acp_id
)
WHERE b.acp_id IS NOT NULL;

-- Restore index composite (création conditionnelle pour idempotence).
CREATE INDEX IF NOT EXISTS idx_buildings_org_created
    ON buildings(organization_id, created_at DESC);

-- Restore RLS policy buildings_isolation (cf. 20250103000000).
-- Note : ENABLE ROW LEVEL SECURITY a déjà été appelé ; on ne re-DISABLE pas.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_policies
        WHERE schemaname = 'public' AND tablename = 'buildings'
          AND policyname = 'buildings_isolation'
    ) THEN
        EXECUTE 'CREATE POLICY buildings_isolation ON buildings
                 USING (organization_id = current_setting(''app.current_organization_id'', true)::UUID)';
    END IF;
END $$;

ALTER TABLE buildings ALTER COLUMN acp_id DROP NOT NULL;

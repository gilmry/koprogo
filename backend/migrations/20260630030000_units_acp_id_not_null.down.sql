-- Story H15 — DOWN Étape 3/3
-- Rollback de 20260630030000_units_acp_id_not_null.sql.
--
-- Démarche :
--   1. Ré-ajoute organization_id NULLABLE.
--   2. Backfill inverse : units.organization_id = acps.organization_id via acp_id.
--   3. Restore les 3 index org (idx_units_organization_id, idx_units_org_created,
--      idx_units_org_number).
--   4. Restore la RLS policy units_isolation (à l'identique de 20250103000000).
--   5. ALTER acp_id DROP NOT NULL (acp_id redevient NULLABLE pour permettre
--      l'enchaînement avec le DOWN 020000 qui nullify acp_id).

ALTER TABLE units ADD COLUMN organization_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE;

UPDATE units u
SET organization_id = (
    SELECT a.organization_id FROM acps a WHERE a.id = u.acp_id
)
WHERE u.acp_id IS NOT NULL;

-- Restore index org (création conditionnelle pour idempotence).
CREATE INDEX IF NOT EXISTS idx_units_organization_id ON units(organization_id);
CREATE INDEX IF NOT EXISTS idx_units_org_created ON units(organization_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_units_org_number ON units(organization_id, unit_number);

-- Restore RLS policy units_isolation (cf. 20250103000000). ENABLE ROW LEVEL
-- SECURITY a déjà été appelé ; on ne re-DISABLE pas (cf. 20250103000002).
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_policies
        WHERE schemaname = 'public' AND tablename = 'units'
          AND policyname = 'units_isolation'
    ) THEN
        EXECUTE 'CREATE POLICY units_isolation ON units
                 USING (organization_id = current_setting(''app.current_organization_id'', true)::UUID)';
    END IF;
END $$;

ALTER TABLE units ALTER COLUMN acp_id DROP NOT NULL;

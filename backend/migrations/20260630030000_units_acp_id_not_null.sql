-- Story H15 — Étape 3/3 : ALTER acp_id SET NOT NULL + DROP organization_id
-- Source : architecture.md §1.1.
--
-- Pré-check : aucun lot avec acp_id IS NULL. Si la migration 020000 a bien
-- tourné, l'assertion passe ; sinon l'EXCEPTION fait échouer la migration et
-- préserve l'état (le SET NOT NULL n'est pas exécuté).
--
-- DROP organization_id : casse le code qui réfère encore `units.organization_id`.
-- Le refactor domaine/repo/use-cases/handlers (Story H15 code) doit être mergé
-- AVEC cette migration (les requêtes runtime `sqlx::query` du repo units lisent
-- désormais acp_id).
--
-- Rollback : `20260630030000_units_acp_id_not_null.down.sql`.

-- Pré-check assertion.
DO $$
DECLARE
    null_count INT;
BEGIN
    SELECT COUNT(*) INTO null_count FROM units WHERE acp_id IS NULL;
    IF null_count > 0 THEN
        RAISE EXCEPTION 'NOT NULL impossible : % lot(s) ont acp_id IS NULL. Re-jouer la migration 020000 backfill.', null_count;
    END IF;
END $$;

-- Étape 3.1 : NOT NULL.
ALTER TABLE units ALTER COLUMN acp_id SET NOT NULL;

-- Étape 3.2 : drop policy RLS + index composites org + colonne organization_id.
-- La policy units_isolation (cf. 20250103000000) référence organization_id :
-- DROP la policy avant DROP la colonne. Les 3 index org (idx_units_organization_id,
-- idx_units_org_created, idx_units_org_number) doivent aussi sauter avant DROP COLUMN.
DROP POLICY IF EXISTS units_isolation ON units;
DROP INDEX IF EXISTS idx_units_organization_id;
DROP INDEX IF EXISTS idx_units_org_created;
DROP INDEX IF EXISTS idx_units_org_number;

ALTER TABLE units DROP COLUMN organization_id;

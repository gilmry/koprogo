-- Story 1.2 — Migration data buildings.organization_id → acp_id (Étape 1/3)
-- Source : docs/maury/refonte-ux-multi-role-acp/architecture.md §5.2
--
-- Ajoute la colonne `acp_id` NULLABLE + FK + index. NULLABLE car le backfill
-- s'effectue dans la migration suivante (030000) ; l'ALTER NOT NULL n'arrive
-- qu'à l'étape 040000.
--
-- Rollback : `20260601_020000_add_buildings_acp_id.down.sql`.

ALTER TABLE buildings
    ADD COLUMN acp_id UUID NULL REFERENCES acps(id);

CREATE INDEX IF NOT EXISTS idx_buildings_acp_id ON buildings(acp_id);

COMMENT ON COLUMN buildings.acp_id IS 'FK vers acps (Story 1.2 etape 1/3 : NULLABLE le temps du backfill).';

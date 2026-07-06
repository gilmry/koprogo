-- Story H15 — Migration data units.organization_id → acp_id (Étape 1/3)
-- Source : docs/maury/refonte-ux-multi-role-acp/track-h-conformite-legale/architecture.md §1.1
-- Cohérence #602 (buildings.organization_id → acp_id déjà migré en 20260601040000).
--
-- Ajoute la colonne `acp_id` NULLABLE + FK + index. NULLABLE car le backfill
-- s'effectue dans la migration suivante (020000) depuis `building.acp_id` ;
-- l'ALTER NOT NULL n'arrive qu'à l'étape 030000.
--
-- Rollback : `20260630010000_add_units_acp_id.down.sql`.

ALTER TABLE units
    ADD COLUMN acp_id UUID NULL REFERENCES acps(id);

CREATE INDEX IF NOT EXISTS idx_units_acp_id ON units(acp_id);

COMMENT ON COLUMN units.acp_id IS 'FK vers acps (Story H15 etape 1/3 : NULLABLE le temps du backfill depuis building.acp_id).';

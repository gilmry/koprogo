-- Rollback Story H4 (WP-CL1) — retrait de la colonne acte de base ACP.
ALTER TABLE acps DROP COLUMN IF EXISTS total_tantiemes;

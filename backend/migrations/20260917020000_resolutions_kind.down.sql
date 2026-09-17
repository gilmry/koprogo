-- Annule Story 4.6 (#581). La contrainte tombe avec la colonne.
ALTER TABLE resolutions DROP COLUMN IF EXISTS kind;

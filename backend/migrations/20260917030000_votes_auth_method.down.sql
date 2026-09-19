-- Annule Story 4.2 (#48). La contrainte tombe avec la colonne.
ALTER TABLE votes DROP COLUMN IF EXISTS auth_method;

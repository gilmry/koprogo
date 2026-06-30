-- Down — Story H17 (Track H, CL3). Réversible : retire la qualification de
-- titularité / représentant de vote des lignes `unit_owners`.

ALTER TABLE unit_owners DROP CONSTRAINT IF EXISTS valid_ownership_type;
ALTER TABLE unit_owners DROP COLUMN IF EXISTS is_voting_representative;
ALTER TABLE unit_owners DROP COLUMN IF EXISTS ownership_type;

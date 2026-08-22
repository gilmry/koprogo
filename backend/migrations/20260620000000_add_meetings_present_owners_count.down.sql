-- Down — Story H9 : retrait du compteur de têtes présentes.

ALTER TABLE meetings
    DROP CONSTRAINT IF EXISTS non_negative_present_owners_count;

ALTER TABLE meetings
    DROP COLUMN IF EXISTS present_owners_count;

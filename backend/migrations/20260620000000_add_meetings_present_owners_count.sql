-- Story H9 (Track H, CL3) — volet « têtes » du quorum double (Art. 3.87 §5).
--
-- L'AG délibère valablement si > la moitié des COPROPRIÉTAIRES (têtes) sont
-- présents/représentés ET ≥ la moitié des quotités, OU si les quotités > 3/4.
-- On stocke le nombre de copropriétaires présents/représentés (saisi par le
-- syndic, comme `present_quotas`). Le total des copropriétaires est calculé
-- (COUNT DISTINCT owners du building). Cf. ADR-0011.
--
-- Nullable : tant que la présence n'est pas saisie, `attendance_recorded` reste
-- faux (cf. `present_quotas IS NOT NULL`). CHECK >= 0.

ALTER TABLE meetings
    ADD COLUMN present_owners_count INTEGER;

ALTER TABLE meetings
    ADD CONSTRAINT non_negative_present_owners_count
    CHECK (present_owners_count IS NULL OR present_owners_count >= 0);

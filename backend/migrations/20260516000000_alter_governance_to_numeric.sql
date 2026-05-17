-- Migration: ADR-0008 — governance quotities/voting-power columns DOUBLE PRECISION -> NUMERIC
-- Story 521-C1 (#534), closes #525 (units.quota DOUBLE PRECISION -> Rust Decimal ColumnDecode panic).
-- Legal anchor: Art. 3.87 §5 CC (quorum), Art. 3.87 §2 CC (AGE 1/5 threshold).
--
-- PCMN / governance exactness requires NUMERIC end-to-end (cf. ADR-0007/0008).
-- DOUBLE PRECISION (IEEE 754 binary64) forces a lossy f64 round-trip even into a
-- Rust Decimal target. Idempotent forward-only (project convention: no down file).
-- v0.1.0, no production data (memory project_koprogo-current-state) — USING cast safe here.

-- units.quota : tantiemes / quotites (CHECK quota > 0 AND quota <= 10000 preserved).
ALTER TABLE units
    ALTER COLUMN quota TYPE NUMERIC(10, 4) USING quota::NUMERIC(10, 4);

-- meetings.total_quotas / present_quotas : nullable — preserve nullability.
ALTER TABLE meetings
    ALTER COLUMN total_quotas TYPE NUMERIC(10, 4) USING total_quotas::NUMERIC(10, 4),
    ALTER COLUMN present_quotas TYPE NUMERIC(10, 4) USING present_quotas::NUMERIC(10, 4);

COMMENT ON COLUMN units.quota IS
    'Quote-part (tantiemes/milliemes). NUMERIC(10,4) — exact (ADR-0008). 0 < quota <= 10000.';
COMMENT ON COLUMN meetings.total_quotas IS
    'Total des milliemes du batiment (generalement 1000). NUMERIC(10,4) exact (ADR-0008).';
COMMENT ON COLUMN meetings.present_quotas IS
    'Milliemes presents + representes par procuration. NUMERIC(10,4) exact (ADR-0008).';

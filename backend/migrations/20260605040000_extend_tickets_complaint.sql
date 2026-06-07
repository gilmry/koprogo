-- Story 3.6 (FR31, brief C17) — Ticket extension: plaintes + évidences + témoins.
--
-- This migration extends the existing `tickets` table (created in
-- 20251116000000_create_tickets.sql) with the fields needed to support the
-- "Complaint" workflow (refonte UX multi-rôle ACP, slice 3, story 3.6):
--
-- - `kind`                  : `request` (default — backward-compat) or `complaint`
-- - `severity`              : urgency tier used for complaint triage
-- - `incident_date`         : when the incident occurred (≤ created_at)
-- - `evidence_attachments`  : URLs / S3-MinIO references (max 10, app-level)
-- - `witnesses`             : user_ids témoins (max 10, app-level, no dup)
--
-- INV-24 (immutabilité après 5 min) is enforced at the use-case layer
-- (comparing `created_at` vs `now()` + `chrono::Duration::minutes(5)`).
-- A 5-min DB CHECK is intentionally NOT added because it would require row
-- triggers to compare NOW() at every UPDATE; the application boundary is the
-- right authoritative place for this temporal invariant.
--
-- All new columns carry sensible defaults so pre-existing `Request` rows
-- remain valid without backfill.

ALTER TABLE tickets
    ADD COLUMN kind VARCHAR(16) NOT NULL DEFAULT 'request'
        CHECK (kind IN ('request', 'complaint')),
    ADD COLUMN severity VARCHAR(16)
        CHECK (severity IN ('low', 'normal', 'high', 'critical')),
    ADD COLUMN incident_date TIMESTAMPTZ,
    ADD COLUMN evidence_attachments TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN witnesses UUID[] NOT NULL DEFAULT '{}';

-- A Complaint MUST carry a severity (triage signal). A Request MAY omit it.
ALTER TABLE tickets
    ADD CONSTRAINT tickets_complaint_requires_severity
    CHECK (kind <> 'complaint' OR severity IS NOT NULL);

-- Partial index speeds up syndic complaint dashboards (high/critical triage).
CREATE INDEX idx_tickets_kind_severity ON tickets(kind, severity)
    WHERE kind = 'complaint';

-- Story 3.6 — DOWN migration for tickets extension (Complaint workflow).

DROP INDEX IF EXISTS idx_tickets_kind_severity;

ALTER TABLE tickets
    DROP CONSTRAINT IF EXISTS tickets_complaint_requires_severity;

ALTER TABLE tickets
    DROP COLUMN IF EXISTS witnesses,
    DROP COLUMN IF EXISTS evidence_attachments,
    DROP COLUMN IF EXISTS incident_date,
    DROP COLUMN IF EXISTS severity,
    DROP COLUMN IF EXISTS kind;

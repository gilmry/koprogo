-- Add retention_until column to audit_logs for GDPR compliance
-- GDPR Article 30: 7-year retention policy for Belgium

ALTER TABLE audit_logs
ADD COLUMN IF NOT EXISTS retention_until TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 years');

-- Index for efficient cleanup queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_retention ON audit_logs(retention_until);

-- Comment for documentation
COMMENT ON COLUMN audit_logs.retention_until IS 'Date when this log can be purged. Default: 7 years from creation (Belgium GDPR requirement).';

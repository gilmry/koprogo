-- Create audit_logs table for security and compliance tracking
-- This table stores all audit events from the AuditLogEntry system

CREATE TABLE IF NOT EXISTS audit_logs (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Event timestamp (when the event occurred)
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Event type (e.g., UserLogin, BuildingCreated, UnauthorizedAccess)
    event_type TEXT NOT NULL,

    -- User who performed the action (nullable for system events)
    user_id UUID,

    -- Organization context (nullable for non-tenant events)
    organization_id UUID,

    -- Resource information
    resource_type TEXT,
    resource_id UUID,

    -- Request metadata
    ip_address TEXT,
    user_agent TEXT,

    -- Additional JSON metadata (flexible storage for event-specific data)
    metadata JSONB,

    -- Event outcome
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,

    -- Record creation timestamp (for audit trail)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on timestamp for time-based queries (most common use case)
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);

-- Index on user_id for user activity tracking
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id) WHERE user_id IS NOT NULL;

-- Index on organization_id for tenant-specific audit queries
CREATE INDEX idx_audit_logs_organization_id ON audit_logs(organization_id) WHERE organization_id IS NOT NULL;

-- Index on event_type for filtering by event category
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);

-- Composite index for organization + timestamp (most common query pattern)
CREATE INDEX idx_audit_logs_org_timestamp ON audit_logs(organization_id, timestamp DESC) WHERE organization_id IS NOT NULL;

-- Index on success for filtering failed operations
CREATE INDEX idx_audit_logs_success ON audit_logs(success) WHERE success = false;

-- GIN index on metadata JSONB for flexible querying
CREATE INDEX idx_audit_logs_metadata ON audit_logs USING GIN(metadata) WHERE metadata IS NOT NULL;

-- Comments for documentation
COMMENT ON TABLE audit_logs IS 'Audit trail for all system events (security, compliance, debugging)';
COMMENT ON COLUMN audit_logs.id IS 'Unique identifier for the audit log entry';
COMMENT ON COLUMN audit_logs.timestamp IS 'When the event occurred';
COMMENT ON COLUMN audit_logs.event_type IS 'Type of event (UserLogin, BuildingCreated, etc.)';
COMMENT ON COLUMN audit_logs.user_id IS 'User who performed the action (NULL for system events)';
COMMENT ON COLUMN audit_logs.organization_id IS 'Organization context (NULL for non-tenant events)';
COMMENT ON COLUMN audit_logs.resource_type IS 'Type of resource affected (Building, Unit, etc.)';
COMMENT ON COLUMN audit_logs.resource_id IS 'ID of the resource affected';
COMMENT ON COLUMN audit_logs.ip_address IS 'Client IP address';
COMMENT ON COLUMN audit_logs.user_agent IS 'Client user agent string';
COMMENT ON COLUMN audit_logs.metadata IS 'Additional event-specific data in JSON format';
COMMENT ON COLUMN audit_logs.success IS 'Whether the operation succeeded';
COMMENT ON COLUMN audit_logs.error_message IS 'Error message if operation failed';
COMMENT ON COLUMN audit_logs.created_at IS 'When this audit log entry was created';

-- Optional: Add foreign key constraints (commented out for flexibility)
-- Uncomment if you want referential integrity enforcement
ALTER TABLE audit_logs ADD CONSTRAINT fk_audit_logs_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE audit_logs ADD CONSTRAINT fk_audit_logs_organization_id
FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE SET NULL;

-- Optional: Add table partitioning for large-scale deployments
-- Example: Partition by month for time-series data
-- CREATE TABLE audit_logs_2025_01 PARTITION OF audit_logs
--     FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

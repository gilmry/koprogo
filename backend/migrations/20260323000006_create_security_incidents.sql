-- Migration: GDPR Article 33 - Security Incident Tracking & 72-hour APD Notification
-- Issue #317 - Security incident procedure
-- Date: 2026-03-23

-- Table: Security Incidents
-- Tracks security breaches and incidents with APD notification status
CREATE TABLE IF NOT EXISTS security_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low')),
    incident_type VARCHAR(100) NOT NULL, -- "data_breach", "unauthorized_access", "malware", "ransomware", etc.
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    data_categories_affected TEXT[] NOT NULL, -- array: "payment_data", "personal_info", "building_data", etc.
    affected_subjects_count INTEGER,
    discovery_at TIMESTAMPTZ NOT NULL,
    notification_at TIMESTAMPTZ, -- when APD was notified (NULL until notified)
    apd_reference_number VARCHAR(100), -- APD's acknowledgment number
    status VARCHAR(50) NOT NULL CHECK (status IN ('detected', 'investigating', 'contained', 'reported', 'closed')),
    reported_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    investigation_notes TEXT,
    root_cause TEXT,
    remediation_steps TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for efficient querying
CREATE INDEX idx_security_incidents_organization ON security_incidents(organization_id);
CREATE INDEX idx_security_incidents_severity ON security_incidents(severity);
CREATE INDEX idx_security_incidents_status ON security_incidents(status);
CREATE INDEX idx_security_incidents_created_at ON security_incidents(created_at DESC);
CREATE INDEX idx_security_incidents_discovery_at ON security_incidents(discovery_at DESC);

-- Partial index: Find incidents overdue for APD notification (>72 hours old, not yet reported)
-- Used for automated compliance checks
CREATE INDEX idx_security_incidents_not_reported
    ON security_incidents(discovery_at)
    WHERE notification_at IS NULL
      AND status IN ('detected', 'investigating', 'contained')
      AND discovery_at < (NOW() - INTERVAL '72 hours');

-- Partial index: Find high-severity incidents requiring urgent response
CREATE INDEX idx_security_incidents_urgent
    ON security_incidents(discovery_at DESC)
    WHERE severity IN ('critical', 'high')
      AND status IN ('detected', 'investigating');

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_security_incidents_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER security_incidents_update_timestamp
    BEFORE UPDATE ON security_incidents
    FOR EACH ROW
    EXECUTE FUNCTION update_security_incidents_timestamp();

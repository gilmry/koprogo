-- Create tickets table for maintenance requests system
-- Issue #85 - Phase 2

CREATE TABLE IF NOT EXISTS tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    unit_id UUID REFERENCES units(id) ON DELETE SET NULL,
    created_by UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    priority VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Open',
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT tickets_title_not_empty CHECK (LENGTH(TRIM(title)) > 0),
    CONSTRAINT tickets_description_not_empty CHECK (LENGTH(TRIM(description)) > 0),
    CONSTRAINT tickets_category_valid CHECK (category IN (
        'Plumbing', 'Electrical', 'Heating', 'CommonAreas',
        'Elevator', 'Security', 'Cleaning', 'Landscaping', 'Other'
    )),
    CONSTRAINT tickets_priority_valid CHECK (priority IN ('Low', 'Medium', 'High', 'Critical')),
    CONSTRAINT tickets_status_valid CHECK (status IN ('Open', 'InProgress', 'Resolved', 'Closed', 'Cancelled')),
    CONSTRAINT tickets_resolved_at_set_when_resolved CHECK (
        (status IN ('Resolved', 'Closed') AND resolved_at IS NOT NULL) OR
        (status NOT IN ('Resolved', 'Closed') AND resolved_at IS NULL)
    ),
    CONSTRAINT tickets_closed_at_set_when_closed CHECK (
        (status = 'Closed' AND closed_at IS NOT NULL) OR
        (status != 'Closed' AND closed_at IS NULL)
    )
);

-- Indexes for performance
CREATE INDEX idx_tickets_building ON tickets(building_id);
CREATE INDEX idx_tickets_organization ON tickets(organization_id);
CREATE INDEX idx_tickets_unit ON tickets(unit_id);
CREATE INDEX idx_tickets_created_by ON tickets(created_by);
CREATE INDEX idx_tickets_assigned_to ON tickets(assigned_to);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_tickets_priority ON tickets(priority);
CREATE INDEX idx_tickets_category ON tickets(category);
CREATE INDEX idx_tickets_created_at ON tickets(created_at DESC);

-- Composite indexes for common queries
CREATE INDEX idx_tickets_building_status ON tickets(building_id, status);
CREATE INDEX idx_tickets_assigned_status ON tickets(assigned_to, status) WHERE assigned_to IS NOT NULL;
CREATE INDEX idx_tickets_organization_building ON tickets(organization_id, building_id);

-- Comments
COMMENT ON TABLE tickets IS 'Maintenance request tickets submitted by co-owners';
COMMENT ON COLUMN tickets.category IS 'Type of maintenance: Plumbing, Electrical, Heating, etc.';
COMMENT ON COLUMN tickets.priority IS 'Urgency level: Low, Medium, High, Critical';
COMMENT ON COLUMN tickets.status IS 'Workflow state: Open → InProgress → Resolved → Closed';
COMMENT ON COLUMN tickets.created_by IS 'Owner who created the ticket';
COMMENT ON COLUMN tickets.assigned_to IS 'User (syndic/contractor) assigned to handle the ticket';
COMMENT ON COLUMN tickets.resolution_notes IS 'Notes from resolver explaining how issue was fixed';

-- Migration: Create board of directors system tables
-- Date: 2025-11-01
-- Issue: #82 - Conseil de Copropriété (Article 577-8/4 Code Civil belge)

-- ============================================================================
-- ENUM Types
-- ============================================================================

CREATE TYPE board_position AS ENUM ('president', 'treasurer', 'secretary', 'member');
CREATE TYPE decision_status AS ENUM ('pending', 'in_progress', 'completed', 'overdue', 'cancelled');

-- ============================================================================
-- Table: board_members
-- Purpose: Members of the board of directors (conseil de copropriété)
-- Legal: Mandatory for buildings with >20 units in Belgium
-- Note: Board members must be property owners (Owner), not platform users
--       Owners may not have platform accounts but can still be board members
-- ============================================================================

CREATE TABLE IF NOT EXISTS board_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    position board_position NOT NULL,
    mandate_start TIMESTAMPTZ NOT NULL,
    mandate_end TIMESTAMPTZ NOT NULL,
    elected_by_meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Business rules
    CONSTRAINT mandate_dates_valid CHECK (mandate_start < mandate_end),
    CONSTRAINT mandate_duration_one_year CHECK (
        mandate_end - mandate_start >= INTERVAL '330 days' AND
        mandate_end - mandate_start <= INTERVAL '395 days'
    ),

    -- One owner can only have one active position per building at a time
    UNIQUE (owner_id, building_id, position)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_board_members_owner_id ON board_members(owner_id);
CREATE INDEX IF NOT EXISTS idx_board_members_building_id ON board_members(building_id);
CREATE INDEX IF NOT EXISTS idx_board_members_mandate_end ON board_members(building_id, mandate_end);

-- ============================================================================
-- Table: board_decisions
-- Purpose: Track AG decisions and their execution by the syndic
-- Monitored by: Board of directors
-- ============================================================================

CREATE TABLE IF NOT EXISTS board_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    subject VARCHAR(255) NOT NULL,
    decision_text TEXT NOT NULL,
    deadline TIMESTAMPTZ,
    status decision_status NOT NULL DEFAULT 'pending',
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Business rules
    CONSTRAINT subject_not_empty CHECK (LENGTH(TRIM(subject)) > 0),
    CONSTRAINT decision_text_not_empty CHECK (LENGTH(TRIM(decision_text)) > 0)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_board_decisions_building_id ON board_decisions(building_id);
CREATE INDEX IF NOT EXISTS idx_board_decisions_meeting_id ON board_decisions(meeting_id);
CREATE INDEX IF NOT EXISTS idx_board_decisions_status ON board_decisions(status);
CREATE INDEX IF NOT EXISTS idx_board_decisions_deadline ON board_decisions(building_id, deadline, status);

-- ============================================================================
-- Trigger: Prevent syndic from being board member (legal incompatibility)
-- Legal basis: Article 577-8/4 §3 Code Civil belge
-- The syndic cannot be a member of the board of directors
-- Note: Board members are owners. We check if the owner is linked to a user
--       who has the syndic role for the same organization.
-- ============================================================================

CREATE OR REPLACE FUNCTION check_syndic_board_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if the owner is linked to a user who is syndic for the same building
    IF EXISTS (
        SELECT 1
        FROM owners o
        INNER JOIN user_roles ur ON ur.user_id = o.user_id
        WHERE o.id = NEW.owner_id
          AND ur.role = 'syndic'
          AND ur.organization_id = (
              SELECT organization_id
              FROM buildings
              WHERE id = NEW.building_id
          )
    ) THEN
        RAISE EXCEPTION 'A syndic cannot be a board member for the same building (legal incompatibility)';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_syndic_board_incompatibility
    BEFORE INSERT OR UPDATE ON board_members
    FOR EACH ROW
    EXECUTE FUNCTION check_syndic_board_incompatibility();

-- ============================================================================
-- Trigger: Prevent syndic role if user is board member
-- Reciprocal check for user_roles table
-- Note: Checks if the user is linked to an owner who is a board member
-- ============================================================================

CREATE OR REPLACE FUNCTION check_board_syndic_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    -- Only check if role is syndic
    IF NEW.role = 'syndic' THEN
        -- Check if user is linked to an owner who is a board member for any building in the same organization
        IF EXISTS (
            SELECT 1
            FROM owners o
            INNER JOIN board_members bm ON bm.owner_id = o.id
            INNER JOIN buildings b ON bm.building_id = b.id
            WHERE o.user_id = NEW.user_id
              AND b.organization_id = NEW.organization_id
              AND bm.mandate_end > CURRENT_TIMESTAMP -- Only active mandates
        ) THEN
            RAISE EXCEPTION 'A board member cannot be a syndic for the same organization (legal incompatibility)';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_board_syndic_incompatibility
    BEFORE INSERT OR UPDATE ON user_roles
    FOR EACH ROW
    EXECUTE FUNCTION check_board_syndic_incompatibility();

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE board_members IS 'Board of directors members (conseil de copropriété) - Mandatory for buildings >20 units (Belgian law)';
COMMENT ON TABLE board_decisions IS 'AG decisions tracked and monitored by the board of directors';
COMMENT ON COLUMN board_members.position IS 'Position in board: president, treasurer, or member';
COMMENT ON COLUMN board_members.mandate_start IS 'Start date of mandate (typically from AG date)';
COMMENT ON COLUMN board_members.mandate_end IS 'End date of mandate (must be ~1 year from start)';
COMMENT ON COLUMN board_decisions.deadline IS 'Deadline for syndic to execute the decision (if applicable)';
COMMENT ON COLUMN board_decisions.status IS 'Status: pending, in_progress, completed, overdue, cancelled';

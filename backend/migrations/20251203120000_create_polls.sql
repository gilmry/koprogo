-- Migration: Create polls system for board decision-making
-- Issue #51 - Board Tools (Polls, Tasks, Issues)
-- Belgian context: Conseil de copropriété quick consultations

-- ============================================================================
-- ENUM Types
-- ============================================================================

CREATE TYPE poll_type AS ENUM (
    'yes_no',
    'multiple_choice',
    'rating',
    'open_ended'
);

CREATE TYPE poll_status AS ENUM (
    'draft',
    'active',
    'closed',
    'cancelled'
);

-- ============================================================================
-- Table: polls
-- Purpose: Polls created by board members or syndic
-- Use cases: Contractor selection, color choices, scheduling decisions
-- ============================================================================

CREATE TABLE polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Poll details
    title VARCHAR(255) NOT NULL,
    description TEXT,
    poll_type poll_type NOT NULL,
    options JSONB NOT NULL DEFAULT '[]'::jsonb, -- Array of PollOption objects
    is_anonymous BOOLEAN NOT NULL DEFAULT false,

    -- Voting settings
    allow_multiple_votes BOOLEAN NOT NULL DEFAULT false,
    require_all_owners BOOLEAN NOT NULL DEFAULT false,

    -- Schedule
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at TIMESTAMPTZ NOT NULL,

    -- Status and results
    status poll_status NOT NULL DEFAULT 'draft',
    total_eligible_voters INTEGER NOT NULL DEFAULT 0,
    total_votes_cast INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT poll_title_not_empty CHECK (LENGTH(TRIM(title)) > 0),
    CONSTRAINT poll_ends_after_starts CHECK (ends_at > starts_at),
    CONSTRAINT poll_eligible_voters_positive CHECK (total_eligible_voters >= 0),
    CONSTRAINT poll_votes_count_valid CHECK (total_votes_cast >= 0 AND total_votes_cast <= total_eligible_voters)
);

-- Indexes for performance
CREATE INDEX idx_polls_building ON polls(building_id);
CREATE INDEX idx_polls_created_by ON polls(created_by);
CREATE INDEX idx_polls_status ON polls(status);
CREATE INDEX idx_polls_ends_at ON polls(ends_at) WHERE status = 'active';
CREATE INDEX idx_polls_building_status ON polls(building_id, status);

-- ============================================================================
-- Table: poll_votes
-- Purpose: Individual votes cast on polls
-- Privacy: Can be anonymous (owner_id NULL)
-- ============================================================================

CREATE TABLE poll_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    owner_id UUID REFERENCES owners(id) ON DELETE CASCADE, -- NULL if anonymous
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Vote details (only one field should be populated based on poll_type)
    selected_option_ids JSONB NOT NULL DEFAULT '[]'::jsonb, -- Array of UUIDs for YesNo/MultipleChoice
    rating_value INTEGER CHECK (rating_value IS NULL OR (rating_value >= 1 AND rating_value <= 5)),
    open_text TEXT,

    -- Metadata
    voted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45), -- IPv4 or IPv6 for audit trail

    -- Constraints
    CONSTRAINT poll_vote_has_content CHECK (
        (jsonb_array_length(selected_option_ids) > 0) OR
        (rating_value IS NOT NULL) OR
        (open_text IS NOT NULL)
    )
);

-- Indexes for performance
CREATE INDEX idx_poll_votes_poll ON poll_votes(poll_id);
CREATE INDEX idx_poll_votes_owner ON poll_votes(owner_id) WHERE owner_id IS NOT NULL;
CREATE INDEX idx_poll_votes_building ON poll_votes(building_id);
CREATE INDEX idx_poll_votes_poll_owner ON poll_votes(poll_id, owner_id);

-- Unique constraint: One vote per owner per poll (if not anonymous)
CREATE UNIQUE INDEX idx_poll_votes_unique ON poll_votes(poll_id, owner_id) WHERE owner_id IS NOT NULL;

-- ============================================================================
-- Triggers
-- ============================================================================

-- Update polls.updated_at on change
CREATE OR REPLACE FUNCTION update_polls_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_polls_updated_at
    BEFORE UPDATE ON polls
    FOR EACH ROW
    EXECUTE FUNCTION update_polls_timestamp();

-- Auto-close polls when end_date is reached (called by cron job)
CREATE OR REPLACE FUNCTION auto_close_expired_polls()
RETURNS INTEGER AS $$
DECLARE
    closed_count INTEGER;
BEGIN
    UPDATE polls
    SET status = 'closed', updated_at = NOW()
    WHERE status = 'active'
      AND ends_at <= NOW();

    GET DIAGNOSTICS closed_count = ROW_COUNT;
    RETURN closed_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE polls IS 'Polls for quick board consultations between general assemblies';
COMMENT ON TABLE poll_votes IS 'Individual votes cast on polls (can be anonymous)';
COMMENT ON COLUMN polls.options IS 'JSONB array of PollOption objects: {id, option_text, attachment_url, vote_count, display_order}';
COMMENT ON COLUMN poll_votes.selected_option_ids IS 'JSONB array of option UUIDs for YesNo/MultipleChoice polls';
COMMENT ON COLUMN poll_votes.ip_address IS 'IP address for audit trail (GDPR Article 30)';
COMMENT ON FUNCTION auto_close_expired_polls IS 'Cron job function to auto-close polls past their end_date';

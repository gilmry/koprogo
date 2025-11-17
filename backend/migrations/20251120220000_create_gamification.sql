-- Migration: Create gamification tables for achievements and challenges
-- Issue #49 - Phase 6/6 - Gamification & Achievements
-- Belgian Context: Community engagement and participation recognition

-- Create custom ENUM types
CREATE TYPE achievement_category AS ENUM (
    'Community',   -- Community participation achievements
    'Sel',         -- SEL (Local Exchange) achievements
    'Booking',     -- Resource booking achievements
    'Sharing',     -- Object sharing achievements
    'Skills',      -- Skills directory achievements
    'Notice',      -- Notice board achievements
    'Governance',  -- Meeting/voting participation achievements
    'Milestone'    -- Platform usage milestones
);

CREATE TYPE achievement_tier AS ENUM (
    'Bronze',   -- Entry-level achievements
    'Silver',   -- Intermediate achievements
    'Gold',     -- Advanced achievements
    'Platinum', -- Expert-level achievements
    'Diamond'   -- Exceptional achievements
);

CREATE TYPE challenge_type AS ENUM (
    'Individual', -- Individual user challenge
    'Team',       -- Team-based challenge (not implemented in Phase 6)
    'Building'    -- Building-wide challenge
);

CREATE TYPE challenge_status AS ENUM (
    'Draft',     -- Challenge created but not yet active
    'Active',    -- Challenge is active (start <= NOW < end)
    'Completed', -- Challenge has ended
    'Cancelled'  -- Challenge was cancelled
);

-- ============================================================================
-- ACHIEVEMENTS TABLE
-- ============================================================================

CREATE TABLE achievements (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    category achievement_category NOT NULL,
    tier achievement_tier NOT NULL,
    name VARCHAR(100) NOT NULL CHECK (LENGTH(name) >= 3),
    description VARCHAR(500) NOT NULL CHECK (LENGTH(description) >= 10),
    icon VARCHAR(255) NOT NULL CHECK (LENGTH(icon) >= 1),
    points_value INTEGER NOT NULL CHECK (points_value >= 0 AND points_value <= 1000),
    requirements TEXT NOT NULL, -- JSON criteria for achievement
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    is_repeatable BOOLEAN NOT NULL DEFAULT FALSE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Comments on achievements table
COMMENT ON TABLE achievements IS 'Achievement definitions for gamification system';
COMMENT ON COLUMN achievements.category IS 'Achievement category (Community, Sel, Booking, etc.)';
COMMENT ON COLUMN achievements.tier IS 'Achievement tier (Bronze, Silver, Gold, Platinum, Diamond)';
COMMENT ON COLUMN achievements.points_value IS 'Points awarded when earned (0-1000)';
COMMENT ON COLUMN achievements.requirements IS 'JSON criteria for earning (e.g., {"action": "booking_created", "count": 1})';
COMMENT ON COLUMN achievements.is_secret IS 'Hidden achievement (only visible after earning)';
COMMENT ON COLUMN achievements.is_repeatable IS 'Can be earned multiple times';
COMMENT ON COLUMN achievements.display_order IS 'Display order in UI (lower = first)';

-- Indexes for achievements
CREATE INDEX idx_achievements_organization ON achievements(organization_id, display_order ASC, tier ASC);
CREATE INDEX idx_achievements_category ON achievements(organization_id, category, display_order ASC);
CREATE INDEX idx_achievements_tier ON achievements(tier, display_order ASC);

-- ============================================================================
-- USER_ACHIEVEMENTS TABLE
-- ============================================================================

CREATE TABLE user_achievements (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id UUID NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    earned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    progress_data TEXT, -- Optional JSON progress data
    times_earned INTEGER NOT NULL DEFAULT 1 CHECK (times_earned >= 1),
    -- Unique constraint for non-repeatable achievements is handled in application logic
    CONSTRAINT unique_user_achievement UNIQUE (user_id, achievement_id)
);

-- Comments on user_achievements table
COMMENT ON TABLE user_achievements IS 'User-earned achievements with progress tracking';
COMMENT ON COLUMN user_achievements.user_id IS 'User who earned the achievement';
COMMENT ON COLUMN user_achievements.achievement_id IS 'Achievement that was earned';
COMMENT ON COLUMN user_achievements.earned_at IS 'First time earned';
COMMENT ON COLUMN user_achievements.progress_data IS 'Optional JSON progress data';
COMMENT ON COLUMN user_achievements.times_earned IS 'Number of times earned (for repeatable achievements)';

-- Indexes for user_achievements
CREATE INDEX idx_user_achievements_user ON user_achievements(user_id, earned_at DESC);
CREATE INDEX idx_user_achievements_achievement ON user_achievements(achievement_id, earned_at DESC);

-- ============================================================================
-- CHALLENGES TABLE
-- ============================================================================

CREATE TABLE challenges (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID REFERENCES buildings(id) ON DELETE CASCADE, -- NULL = organization-wide
    challenge_type challenge_type NOT NULL,
    status challenge_status NOT NULL DEFAULT 'Draft',
    title VARCHAR(100) NOT NULL CHECK (LENGTH(title) >= 3),
    description VARCHAR(1000) NOT NULL CHECK (LENGTH(description) >= 10),
    icon VARCHAR(255) NOT NULL CHECK (LENGTH(icon) >= 1),
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    target_metric VARCHAR(100) NOT NULL CHECK (LENGTH(target_metric) >= 3),
    target_value INTEGER NOT NULL CHECK (target_value > 0),
    reward_points INTEGER NOT NULL CHECK (reward_points >= 0 AND reward_points <= 10000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT check_start_before_end CHECK (start_date < end_date)
);

-- Comments on challenges table
COMMENT ON TABLE challenges IS 'Time-bound challenges for community engagement';
COMMENT ON COLUMN challenges.challenge_type IS 'Challenge type (Individual, Team, Building)';
COMMENT ON COLUMN challenges.status IS 'Challenge status (Draft, Active, Completed, Cancelled)';
COMMENT ON COLUMN challenges.start_date IS 'Challenge start date (Active if NOW >= start_date)';
COMMENT ON COLUMN challenges.end_date IS 'Challenge end date (Active if NOW < end_date)';
COMMENT ON COLUMN challenges.target_metric IS 'Metric to track (e.g., "bookings_created", "exchanges_completed")';
COMMENT ON COLUMN challenges.target_value IS 'Target value to achieve';
COMMENT ON COLUMN challenges.reward_points IS 'Points awarded upon completion (0-10000)';

-- Indexes for challenges
CREATE INDEX idx_challenges_organization ON challenges(organization_id, start_date DESC);
CREATE INDEX idx_challenges_building ON challenges(building_id, start_date DESC);
CREATE INDEX idx_challenges_status ON challenges(organization_id, status, start_date DESC);
CREATE INDEX idx_challenges_dates ON challenges(start_date ASC, end_date ASC);

-- Partial indexes for common queries
CREATE INDEX idx_challenges_active ON challenges(organization_id, start_date ASC)
    WHERE status = 'Active' AND start_date <= NOW() AND end_date > NOW();

CREATE INDEX idx_challenges_ended_not_completed ON challenges(end_date ASC)
    WHERE status = 'Active' AND end_date <= NOW();

-- ============================================================================
-- CHALLENGE_PROGRESS TABLE
-- ============================================================================

CREATE TABLE challenge_progress (
    id UUID PRIMARY KEY,
    challenge_id UUID NOT NULL REFERENCES challenges(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    current_value INTEGER NOT NULL DEFAULT 0 CHECK (current_value >= 0),
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Unique constraint to prevent duplicate progress
    CONSTRAINT unique_user_challenge UNIQUE (user_id, challenge_id),
    -- Constraint to ensure completed_at is set when completed
    CONSTRAINT check_completed_at CHECK (
        (completed = FALSE AND completed_at IS NULL) OR
        (completed = TRUE AND completed_at IS NOT NULL)
    )
);

-- Comments on challenge_progress table
COMMENT ON TABLE challenge_progress IS 'User progress tracking for challenges';
COMMENT ON COLUMN challenge_progress.current_value IS 'Current progress value (compared to target_value)';
COMMENT ON COLUMN challenge_progress.completed IS 'Challenge completed flag';
COMMENT ON COLUMN challenge_progress.completed_at IS 'Timestamp when challenge was completed';

-- Indexes for challenge_progress
CREATE INDEX idx_challenge_progress_challenge ON challenge_progress(challenge_id, current_value DESC);
CREATE INDEX idx_challenge_progress_user ON challenge_progress(user_id, created_at DESC);

-- Partial index for active challenges
CREATE INDEX idx_challenge_progress_active_user ON challenge_progress(user_id, created_at DESC)
    WHERE completed = FALSE;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Trigger to update achievements.updated_at timestamp
CREATE OR REPLACE FUNCTION update_achievement_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_achievement_timestamp
BEFORE UPDATE ON achievements
FOR EACH ROW
EXECUTE FUNCTION update_achievement_timestamp();

-- Trigger to update challenges.updated_at timestamp
CREATE OR REPLACE FUNCTION update_challenge_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_challenge_timestamp
BEFORE UPDATE ON challenges
FOR EACH ROW
EXECUTE FUNCTION update_challenge_timestamp();

-- Trigger to update challenge_progress.updated_at timestamp
CREATE OR REPLACE FUNCTION update_challenge_progress_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_challenge_progress_timestamp
BEFORE UPDATE ON challenge_progress
FOR EACH ROW
EXECUTE FUNCTION update_challenge_progress_timestamp();

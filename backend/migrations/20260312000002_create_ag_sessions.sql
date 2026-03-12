-- Migration: Create ag_sessions table (BC15 - AG Visioconférence)
-- Art. 3.87 §1 CC: AG peut se tenir "physiquement ou à distance"
-- Art. 3.87 §5 CC: quorum combiné présentiel + distanciel

CREATE TYPE video_platform AS ENUM (
    'zoom',
    'microsoft_teams',
    'google_meet',
    'jitsi',
    'whereby',
    'other'
);

CREATE TYPE ag_session_status AS ENUM (
    'scheduled',
    'live',
    'ended',
    'cancelled'
);

CREATE TABLE ag_sessions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    meeting_id                  UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    platform                    video_platform NOT NULL,
    video_url                   TEXT NOT NULL,
    host_url                    TEXT,
    status                      ag_session_status NOT NULL DEFAULT 'scheduled',
    scheduled_start             TIMESTAMPTZ NOT NULL,
    actual_start                TIMESTAMPTZ,
    actual_end                  TIMESTAMPTZ,

    -- Quorum distanciel (Art. 3.87 §5 CC)
    remote_attendees_count      INTEGER NOT NULL DEFAULT 0,
    remote_voting_power         NUMERIC(10, 4) NOT NULL DEFAULT 0,
    quorum_remote_contribution  NUMERIC(8, 4) NOT NULL DEFAULT 0,

    -- Accès et sécurité
    access_password             TEXT,
    waiting_room_enabled        BOOLEAN NOT NULL DEFAULT TRUE,
    recording_enabled           BOOLEAN NOT NULL DEFAULT FALSE,
    recording_url               TEXT,

    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by                  UUID NOT NULL,

    -- Une seule session active par réunion
    CONSTRAINT uq_ag_sessions_meeting_id UNIQUE (meeting_id),
    CONSTRAINT chk_ag_sessions_remote_attendees CHECK (remote_attendees_count >= 0),
    CONSTRAINT chk_ag_sessions_remote_voting_power CHECK (remote_voting_power >= 0),
    CONSTRAINT chk_ag_sessions_quorum_contribution CHECK (quorum_remote_contribution >= 0 AND quorum_remote_contribution <= 100),
    CONSTRAINT chk_ag_sessions_actual_end CHECK (actual_end IS NULL OR actual_start IS NOT NULL)
);

-- Index pour recherches courantes
CREATE INDEX idx_ag_sessions_organization_id ON ag_sessions(organization_id);
CREATE INDEX idx_ag_sessions_meeting_id ON ag_sessions(meeting_id);
CREATE INDEX idx_ag_sessions_status ON ag_sessions(status);
CREATE INDEX idx_ag_sessions_scheduled_start ON ag_sessions(scheduled_start);

-- Index partiel : sessions planifiées dont la date est passée (daemon de démarrage auto)
CREATE INDEX idx_ag_sessions_pending_start
    ON ag_sessions(scheduled_start)
    WHERE status = 'scheduled';

-- Trigger mise à jour updated_at
CREATE OR REPLACE FUNCTION update_ag_session_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_ag_sessions_updated_at
    BEFORE UPDATE ON ag_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_ag_session_timestamp();

COMMENT ON TABLE ag_sessions IS 'Sessions de visioconférence pour les assemblées générales — Art. 3.87 §1 CC';
COMMENT ON COLUMN ag_sessions.remote_voting_power IS 'Millièmes cumulés des participants distants (Art. 3.87 §5 CC)';
COMMENT ON COLUMN ag_sessions.quorum_remote_contribution IS 'Contribution distancielle au quorum en % (millièmes distants / total bâtiment × 100)';
COMMENT ON COLUMN ag_sessions.host_url IS 'URL hôte avec droits admin — NE PAS exposer aux participants';

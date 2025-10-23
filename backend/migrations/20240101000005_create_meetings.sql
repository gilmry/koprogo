-- Create meetings table
CREATE TYPE meeting_type AS ENUM ('ordinary', 'extraordinary');
CREATE TYPE meeting_status AS ENUM ('scheduled', 'completed', 'cancelled');

CREATE TABLE meetings (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    meeting_type meeting_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    scheduled_date TIMESTAMPTZ NOT NULL,
    location VARCHAR(255) NOT NULL,
    status meeting_status NOT NULL DEFAULT 'scheduled',
    agenda JSONB NOT NULL DEFAULT '[]',
    attendees_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meetings_building ON meetings(building_id);
CREATE INDEX idx_meetings_date ON meetings(scheduled_date);
CREATE INDEX idx_meetings_status ON meetings(status);

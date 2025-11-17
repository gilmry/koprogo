-- Migration: Create notices table for Community Notice Board (Issue #49 - Phase 2)
-- Date: 2025-11-20 17:00:00

-- Create custom ENUM types for notices
CREATE TYPE notice_type AS ENUM (
    'Announcement',    -- General announcement (info, rules, reminders)
    'Event',          -- Community event (party, meeting, workshop)
    'LostAndFound',   -- Lost and found items
    'ClassifiedAd'    -- Classified ad (buy, sell, services)
);

CREATE TYPE notice_category AS ENUM (
    'General',        -- General information
    'Maintenance',    -- Maintenance and repairs
    'Social',         -- Social events and activities
    'Security',       -- Security and safety
    'Environment',    -- Environment and recycling
    'Parking',        -- Parking and transportation
    'Other'           -- Other category
);

CREATE TYPE notice_status AS ENUM (
    'Draft',          -- Draft (not visible to others)
    'Published',      -- Published (visible to all building members)
    'Archived',       -- Archived (moved to history)
    'Expired'         -- Expired (automatically expired based on expires_at)
);

-- Create notices table
CREATE TABLE notices (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    notice_type notice_type NOT NULL,
    category notice_category NOT NULL,
    title VARCHAR(255) NOT NULL CHECK (LENGTH(title) >= 5),
    content TEXT NOT NULL CHECK (LENGTH(content) > 0 AND LENGTH(content) <= 10000),
    status notice_status NOT NULL DEFAULT 'Draft',
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    -- Event-specific fields
    event_date TIMESTAMPTZ,
    event_location VARCHAR(500),
    -- Contact info for LostAndFound and ClassifiedAd
    contact_info VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT check_event_fields CHECK (
        notice_type != 'Event' OR (event_date IS NOT NULL AND event_location IS NOT NULL)
    )
);

-- Create indexes for efficient queries

-- Index for building-based queries (primary use case: list all building notices)
CREATE INDEX idx_notices_building ON notices(building_id, created_at DESC);

-- Index for published notices (marketplace view: pinned first, then by published_at)
CREATE INDEX idx_notices_published ON notices(building_id, status, is_pinned DESC, published_at DESC)
WHERE status = 'Published';

-- Partial index for pinned notices only (important announcements)
CREATE INDEX idx_notices_pinned ON notices(building_id, published_at DESC)
WHERE is_pinned = true;

-- Index for filtering by type (e.g., events calendar)
CREATE INDEX idx_notices_type ON notices(building_id, notice_type, created_at DESC);

-- Index for filtering by category (e.g., maintenance announcements)
CREATE INDEX idx_notices_category ON notices(building_id, category, created_at DESC);

-- Index for filtering by status (e.g., drafts, archived)
CREATE INDEX idx_notices_status ON notices(building_id, status, created_at DESC);

-- Index for author-based queries (my notices)
CREATE INDEX idx_notices_author ON notices(author_id, created_at DESC);

-- Partial index for expired notices (background job: auto-expire cron)
CREATE INDEX idx_notices_expired ON notices(building_id, expires_at ASC)
WHERE status = 'Published' AND expires_at IS NOT NULL;

-- Add column comments for documentation
COMMENT ON TABLE notices IS 'Community notice board for announcements, events, lost & found, and classified ads';
COMMENT ON COLUMN notices.notice_type IS 'Type of notice: Announcement, Event, LostAndFound, ClassifiedAd';
COMMENT ON COLUMN notices.category IS 'Category: General, Maintenance, Social, Security, Environment, Parking, Other';
COMMENT ON COLUMN notices.status IS 'Workflow status: Draft → Published → Archived/Expired';
COMMENT ON COLUMN notices.is_pinned IS 'Pin important notices to top of board (admin-only)';
COMMENT ON COLUMN notices.expires_at IS 'Auto-expiration date (background job marks as Expired)';
COMMENT ON COLUMN notices.event_date IS 'Required for Event type: date and time of event';
COMMENT ON COLUMN notices.event_location IS 'Required for Event type: location description';
COMMENT ON COLUMN notices.contact_info IS 'Optional contact info for LostAndFound and ClassifiedAd types';

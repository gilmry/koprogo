-- Migration: Create resource_bookings table for community space bookings
-- Issue #49 - Phase 5/6 - Resource Booking Calendar
-- Belgian Legal Context: Shared property regulation (Article 3 Loi Copropriété)

-- Create custom ENUM types
CREATE TYPE resource_type AS ENUM (
    'MeetingRoom',
    'LaundryRoom',
    'Gym',
    'Rooftop',
    'ParkingSpot',
    'CommonSpace',
    'GuestRoom',
    'BikeStorage',
    'Other'
);

CREATE TYPE booking_status AS ENUM (
    'Pending',     -- Awaiting confirmation (if approval required)
    'Confirmed',   -- Booking confirmed
    'Cancelled',   -- Cancelled by user or admin
    'Completed',   -- Booking completed (auto-set after end_time)
    'NoShow'       -- User didn't show up
);

CREATE TYPE recurring_pattern AS ENUM (
    'None',
    'Daily',
    'Weekly',
    'Monthly'
);

-- Create resource_bookings table
CREATE TABLE resource_bookings (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    resource_type resource_type NOT NULL,
    resource_name VARCHAR(100) NOT NULL CHECK (LENGTH(resource_name) >= 3),
    booked_by UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    status booking_status NOT NULL DEFAULT 'Confirmed',
    notes VARCHAR(500),
    recurring_pattern recurring_pattern NOT NULL DEFAULT 'None',
    recurrence_end_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT check_start_before_end CHECK (start_time < end_time),
    CONSTRAINT check_recurring_end_date CHECK (
        (recurring_pattern = 'None' AND recurrence_end_date IS NULL) OR
        (recurring_pattern != 'None' AND recurrence_end_date IS NOT NULL AND recurrence_end_date > start_time)
    )
);

-- Comment on table
COMMENT ON TABLE resource_bookings IS 'Community resource bookings for Belgian copropriété shared spaces';

-- Comments on columns
COMMENT ON COLUMN resource_bookings.resource_type IS 'Type of resource being booked';
COMMENT ON COLUMN resource_bookings.resource_name IS 'Specific resource name (e.g., "Meeting Room A", "Laundry Room 1st Floor")';
COMMENT ON COLUMN resource_bookings.booked_by IS 'Owner ID who made the booking';
COMMENT ON COLUMN resource_bookings.status IS 'Booking status lifecycle';
COMMENT ON COLUMN resource_bookings.recurring_pattern IS 'Recurring pattern (None, Daily, Weekly, Monthly)';
COMMENT ON COLUMN resource_bookings.recurrence_end_date IS 'End date for recurring bookings';

-- Indexes for performance
CREATE INDEX idx_resource_bookings_building ON resource_bookings(building_id, start_time DESC);
CREATE INDEX idx_resource_bookings_booked_by ON resource_bookings(booked_by, start_time DESC);
CREATE INDEX idx_resource_bookings_resource ON resource_bookings(building_id, resource_type, resource_name, start_time ASC);
CREATE INDEX idx_resource_bookings_status ON resource_bookings(building_id, status, start_time ASC);
CREATE INDEX idx_resource_bookings_start_time ON resource_bookings(start_time ASC);
CREATE INDEX idx_resource_bookings_end_time ON resource_bookings(end_time ASC);

-- Partial indexes for performance optimization (common queries)
-- Note: Cannot use NOW() in index predicate (NOW() is not IMMUTABLE)
-- Queries will filter on time conditions at runtime
CREATE INDEX idx_resource_bookings_active ON resource_bookings(building_id, start_time ASC, end_time ASC)
    WHERE status = 'Confirmed';

CREATE INDEX idx_resource_bookings_upcoming ON resource_bookings(building_id, start_time ASC)
    WHERE status IN ('Confirmed', 'Pending');

CREATE INDEX idx_resource_bookings_conflict_check ON resource_bookings(building_id, resource_type, resource_name, start_time ASC, end_time ASC)
    WHERE status IN ('Pending', 'Confirmed');

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_resource_booking_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_resource_booking_timestamp
BEFORE UPDATE ON resource_bookings
FOR EACH ROW
EXECUTE FUNCTION update_resource_booking_timestamp();

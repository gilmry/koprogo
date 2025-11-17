-- Migration: Create shared_objects table for Object Sharing Library (Issue #49 - Phase 4)
-- Date: 2025-11-20 19:00:00

-- Create custom ENUM types for shared objects
CREATE TYPE shared_object_category AS ENUM (
    'Tools',       -- Tools and equipment (drill, ladder, hammer, saw, etc.)
    'Books',       -- Books and magazines
    'Electronics', -- Electronics (projector, camera, tablet, etc.)
    'Sports',      -- Sports equipment (bike, skis, tennis racket, etc.)
    'Gardening',   -- Gardening tools and equipment (mower, trimmer, etc.)
    'Kitchen',     -- Kitchen appliances (mixer, pressure cooker, etc.)
    'Baby',        -- Baby and children items (stroller, car seat, toys, etc.)
    'Other'        -- Other shared objects
);

CREATE TYPE object_condition AS ENUM (
    'Excellent',   -- Excellent condition (like new)
    'Good',        -- Good condition (minor wear)
    'Fair',        -- Fair condition (visible wear but functional)
    'Used'         -- Used condition (significant wear)
);

-- Create shared_objects table
CREATE TABLE shared_objects (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    object_category shared_object_category NOT NULL,
    object_name VARCHAR(100) NOT NULL CHECK (LENGTH(object_name) >= 3),
    description VARCHAR(1000) NOT NULL CHECK (LENGTH(description) > 0),
    condition object_condition NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT TRUE,
    rental_credits_per_day INT CHECK (rental_credits_per_day >= 0 AND rental_credits_per_day <= 20),
    deposit_credits INT CHECK (deposit_credits >= 0 AND deposit_credits <= 100),
    borrowing_duration_days INT CHECK (borrowing_duration_days >= 1 AND borrowing_duration_days <= 90),
    current_borrower_id UUID REFERENCES owners(id) ON DELETE SET NULL,
    borrowed_at TIMESTAMPTZ,
    due_back_at TIMESTAMPTZ,
    photos TEXT[], -- Array of photo URLs
    location_details VARCHAR(500),
    usage_instructions TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT check_borrower_not_owner CHECK (owner_id != current_borrower_id),
    CONSTRAINT check_borrowing_dates CHECK (
        (current_borrower_id IS NULL AND borrowed_at IS NULL AND due_back_at IS NULL) OR
        (current_borrower_id IS NOT NULL AND borrowed_at IS NOT NULL AND due_back_at IS NOT NULL)
    )
);

-- Create indexes for efficient queries

-- Index for building-based queries (primary use case: list all building objects)
CREATE INDEX idx_shared_objects_building ON shared_objects(building_id, is_available DESC, object_name ASC);

-- Partial index for available objects (marketplace view: object sharing library)
CREATE INDEX idx_shared_objects_available ON shared_objects(building_id, object_name ASC)
WHERE is_available = TRUE;

-- Partial index for borrowed objects (includes overdue check at query time)
-- Note: Cannot use due_back_at < NOW() in index predicate (NOW() is not IMMUTABLE)
-- Background jobs will filter on due_back_at < NOW() at runtime
CREATE INDEX idx_shared_objects_borrowed ON shared_objects(building_id, due_back_at ASC)
WHERE current_borrower_id IS NOT NULL;

-- Index for filtering by category (e.g., find all tools)
CREATE INDEX idx_shared_objects_category ON shared_objects(building_id, object_category, is_available DESC, object_name ASC);

-- Index for owner-based queries (my shared objects)
CREATE INDEX idx_shared_objects_owner ON shared_objects(owner_id, created_at DESC);

-- Index for borrower-based queries (my borrowed objects)
CREATE INDEX idx_shared_objects_borrower ON shared_objects(current_borrower_id, due_back_at ASC)
WHERE current_borrower_id IS NOT NULL;

-- Partial index for free/volunteer objects (rental_credits_per_day IS NULL OR = 0)
CREATE INDEX idx_shared_objects_free ON shared_objects(building_id, object_name ASC)
WHERE rental_credits_per_day IS NULL OR rental_credits_per_day = 0;

-- Add column comments for documentation
COMMENT ON TABLE shared_objects IS 'Object sharing library for community equipment, tools, books, and appliances';
COMMENT ON COLUMN shared_objects.object_category IS 'Object category: Tools, Books, Electronics, Sports, Gardening, Kitchen, Baby, Other';
COMMENT ON COLUMN shared_objects.condition IS 'Object condition: Excellent (like new), Good (minor wear), Fair (visible wear), Used (significant wear)';
COMMENT ON COLUMN shared_objects.object_name IS 'Name of the shared object (3-100 characters)';
COMMENT ON COLUMN shared_objects.description IS 'Detailed description of the object (max 1000 characters)';
COMMENT ON COLUMN shared_objects.is_available IS 'Whether the object is currently available for borrowing';
COMMENT ON COLUMN shared_objects.rental_credits_per_day IS 'Rental rate in SEL credits per day (0-20, NULL or 0 = free/volunteer)';
COMMENT ON COLUMN shared_objects.deposit_credits IS 'Security deposit in SEL credits (0-100, refunded on return)';
COMMENT ON COLUMN shared_objects.borrowing_duration_days IS 'Maximum borrowing duration in days (1-90)';
COMMENT ON COLUMN shared_objects.current_borrower_id IS 'Current borrower (NULL if available, owner_id of borrower if borrowed)';
COMMENT ON COLUMN shared_objects.borrowed_at IS 'When the object was borrowed (NULL if available)';
COMMENT ON COLUMN shared_objects.due_back_at IS 'When the object is due back (NULL if available, calculated from borrowed_at + duration)';
COMMENT ON COLUMN shared_objects.photos IS 'Array of photo URLs for the object (optional)';
COMMENT ON COLUMN shared_objects.location_details IS 'Pickup location details (optional, e.g., "Basement storage room")';
COMMENT ON COLUMN shared_objects.usage_instructions IS 'Usage instructions for the object (optional, e.g., "Charge battery before use")';

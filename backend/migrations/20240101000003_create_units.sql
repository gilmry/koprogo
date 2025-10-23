-- Create units table
CREATE TYPE unit_type AS ENUM ('apartment', 'parking', 'cellar', 'commercial', 'other');

CREATE TABLE units (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    unit_number VARCHAR(50) NOT NULL,
    unit_type unit_type NOT NULL,
    floor INTEGER,
    surface_area DOUBLE PRECISION NOT NULL CHECK (surface_area > 0),
    quota DOUBLE PRECISION NOT NULL CHECK (quota > 0 AND quota <= 1000),
    owner_id UUID REFERENCES owners(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(building_id, unit_number)
);

CREATE INDEX idx_units_building ON units(building_id);
CREATE INDEX idx_units_owner ON units(owner_id);
CREATE INDEX idx_units_unit_number ON units(unit_number);

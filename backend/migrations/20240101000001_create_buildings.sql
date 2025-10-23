-- Create buildings table
CREATE TABLE buildings (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    total_units INTEGER NOT NULL CHECK (total_units > 0),
    construction_year INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_buildings_city ON buildings(city);
CREATE INDEX idx_buildings_created_at ON buildings(created_at);

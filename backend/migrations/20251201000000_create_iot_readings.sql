-- IoT Readings and Linky Devices - Issue #133 (Linky/Ores API Integration)
-- Regular PostgreSQL tables for time-series IoT data
-- Note: TimescaleDB could be added later for better time-series performance

-- Device types (Linky smart meters, sensors, etc.)
CREATE TYPE device_type AS ENUM (
    'electricity_meter',
    'water_meter',
    'gas_meter',
    'temperature_sensor',
    'humidity_sensor',
    'power_meter'
);

-- Metric types (what is being measured)
CREATE TYPE metric_type AS ENUM (
    'electricity_consumption',
    'water_consumption',
    'gas_consumption',
    'temperature',
    'humidity',
    'power',
    'voltage'
);

-- Linky smart meter providers
CREATE TYPE linky_provider AS ENUM (
    'ores',   -- Belgium (Ores network)
    'enedis'  -- France (Enedis Linky)
);

-- ========================================
-- Table: linky_devices
-- ========================================
CREATE TABLE linky_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    prm VARCHAR(18) NOT NULL, -- Point Reference Measure (14 digits France, 18 digits Belgium)
    provider linky_provider NOT NULL,
    api_key_encrypted TEXT NOT NULL, -- OAuth2 access token (AES-256 encrypted at application level)
    refresh_token_encrypted TEXT, -- OAuth2 refresh token (AES-256 encrypted)
    token_expires_at TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ,
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for linky_devices
CREATE INDEX idx_linky_devices_building ON linky_devices(building_id);
CREATE UNIQUE INDEX idx_linky_devices_prm ON linky_devices(prm, provider); -- One device per PRM/provider
CREATE INDEX idx_linky_devices_sync_needed ON linky_devices(sync_enabled, last_sync_at)
    WHERE sync_enabled = true; -- Optimize daily sync job
CREATE INDEX idx_linky_devices_token_expiration ON linky_devices(token_expires_at)
    WHERE token_expires_at IS NOT NULL; -- Optimize token refresh job

-- ========================================
-- Table: iot_readings (TimescaleDB Hypertable)
-- ========================================
CREATE TABLE iot_readings (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    device_type device_type NOT NULL,
    metric_type metric_type NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL, -- Measurement timestamp (not created_at)
    source VARCHAR(50) NOT NULL, -- 'linky_ores', 'linky_enedis', 'netatmo', etc.
    metadata JSONB, -- Additional context (granularity, quality, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Note: TimescaleDB hypertable conversion disabled (extension not available)
-- SELECT create_hypertable('iot_readings', 'timestamp', if_not_exists => TRUE, chunk_time_interval => INTERVAL '7 days');

-- Indexes for iot_readings
CREATE INDEX idx_iot_readings_building_time ON iot_readings(building_id, timestamp DESC);
CREATE INDEX idx_iot_readings_metric_time ON iot_readings(metric_type, timestamp DESC);
CREATE INDEX idx_iot_readings_device_time ON iot_readings(device_type, timestamp DESC);
CREATE INDEX idx_iot_readings_source ON iot_readings(source);
CREATE INDEX idx_iot_readings_metadata_gin ON iot_readings USING GIN(metadata); -- Search metadata

-- Primary key constraint
ALTER TABLE iot_readings ADD CONSTRAINT iot_readings_pkey PRIMARY KEY (id);

-- ========================================
-- TimescaleDB Features (DISABLED - extension not available)
-- ========================================
-- Note: The following TimescaleDB features are disabled:
-- 1. Retention policies (automatic data deletion)
-- 2. Compression policies (10-20x storage savings)
-- 3. Continuous aggregates (pre-computed materialized views)
--
-- These can be re-enabled by installing TimescaleDB extension.
-- For now, we use standard PostgreSQL tables with regular indexes.

-- ========================================
-- Triggers
-- ========================================

-- Update linky_devices.updated_at on modification
CREATE OR REPLACE FUNCTION update_linky_device_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_linky_devices_updated
    BEFORE UPDATE ON linky_devices
    FOR EACH ROW
    EXECUTE FUNCTION update_linky_device_timestamp();

-- ========================================
-- Comments for Documentation
-- ========================================

COMMENT ON TABLE linky_devices IS 'Linky smart meter device configurations (Ores Belgium, Enedis France)';
COMMENT ON COLUMN linky_devices.prm IS 'Point Reference Measure: 14 digits (France) or 18 digits (Belgium EAN)';
COMMENT ON COLUMN linky_devices.api_key_encrypted IS 'OAuth2 access token (AES-256 encrypted at application level)';
COMMENT ON COLUMN linky_devices.refresh_token_encrypted IS 'OAuth2 refresh token for automatic token renewal';
COMMENT ON COLUMN linky_devices.last_sync_at IS 'Last successful data sync with Linky API';
COMMENT ON COLUMN linky_devices.sync_enabled IS 'Enable/disable automatic daily sync';

COMMENT ON TABLE iot_readings IS 'Time-series IoT sensor readings (standard PostgreSQL table)';
COMMENT ON COLUMN iot_readings.timestamp IS 'Measurement timestamp (when data was recorded by sensor)';
COMMENT ON COLUMN iot_readings.source IS 'Data source: linky_ores, linky_enedis, netatmo, etc.';
COMMENT ON COLUMN iot_readings.metadata IS 'Additional context (granularity: 30min, quality: good, etc.)';
COMMENT ON COLUMN iot_readings.value IS 'Numeric measurement value (validated by application)';
COMMENT ON COLUMN iot_readings.unit IS 'Unit of measurement (kWh, m3, °C, %, W, V)';

-- ========================================
-- Sample Data (for development/testing)
-- ========================================

-- Uncomment for local development:
-- INSERT INTO linky_devices (building_id, prm, provider, api_key_encrypted, refresh_token_encrypted, token_expires_at, sync_enabled)
-- SELECT
--     id,
--     '12345678901234', -- Sample PRM (France)
--     'enedis'::linky_provider,
--     'ENCRYPTED_ACCESS_TOKEN_SAMPLE',
--     'ENCRYPTED_REFRESH_TOKEN_SAMPLE',
--     NOW() + INTERVAL '1 hour',
--     true
-- FROM buildings LIMIT 1;

-- ========================================
-- Performance Notes
-- ========================================
-- 1. Standard PostgreSQL table with B-tree indexes on timestamp
-- 2. Expected storage: ~100 readings/day/building = 36k readings/year
-- 3. Query performance: < 100ms for 30-day range queries with proper indexes
-- 4. For production optimization, consider:
--    - Installing TimescaleDB extension for automatic compression/retention
--    - Adding BRIN indexes for very large datasets
--    - Implementing manual partitioning by month/year
--    - Creating manual aggregation tables for dashboards

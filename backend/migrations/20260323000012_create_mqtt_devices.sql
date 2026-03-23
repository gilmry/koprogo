-- Migration: MQTT Device registry for IoT (Issue #300)
-- Supports Home Assistant, Linky, and custom MQTT topics
-- Date: 2026-03-23

CREATE TABLE IF NOT EXISTS mqtt_devices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id     UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    device_name     VARCHAR(255) NOT NULL,
    device_type     VARCHAR(50) NOT NULL DEFAULT 'HomeAssistant',
    mqtt_topic      VARCHAR(500) NOT NULL UNIQUE,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen_at    TIMESTAMPTZ,
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mqtt_devices_building
    ON mqtt_devices (building_id, is_active);

CREATE INDEX IF NOT EXISTS idx_mqtt_devices_org
    ON mqtt_devices (organization_id);

CREATE INDEX IF NOT EXISTS idx_mqtt_devices_topic
    ON mqtt_devices (mqtt_topic);

CREATE TABLE IF NOT EXISTS mqtt_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id       UUID NOT NULL REFERENCES mqtt_devices(id) ON DELETE CASCADE,
    topic           VARCHAR(500) NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    qos             SMALLINT NOT NULL DEFAULT 0,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mqtt_messages_device_time
    ON mqtt_messages (device_id, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_mqtt_messages_topic_time
    ON mqtt_messages (topic, received_at DESC);

-- Trigger to auto-update mqtt_devices.updated_at on changes
CREATE OR REPLACE FUNCTION update_mqtt_devices_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_mqtt_devices_timestamp_trigger ON mqtt_devices;
CREATE TRIGGER update_mqtt_devices_timestamp_trigger
    BEFORE UPDATE ON mqtt_devices
    FOR EACH ROW
    EXECUTE FUNCTION update_mqtt_devices_timestamp();

COMMENT ON TABLE mqtt_devices IS 'MQTT device registry for Home Assistant and IoT sensors (Issue #300)';
COMMENT ON TABLE mqtt_messages IS 'MQTT message log for IoT device readings (time-series)';
COMMENT ON COLUMN mqtt_devices.device_type IS 'Device type: HomeAssistant, Linky, OresSmartMeter, GenericMQTT';
COMMENT ON COLUMN mqtt_devices.mqtt_topic IS 'MQTT topic pattern (e.g., koprogo/{org_id}/{building_id}/energy/linky)';
COMMENT ON COLUMN mqtt_messages.qos IS 'MQTT QoS level: 0 (at most once), 1 (at least once), 2 (exactly once)';

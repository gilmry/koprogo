-- Migration: Public API v2 — API Key authentication (Issues #111, #232)
-- Enables third-party integrations (PropTech, notaries, energy providers)
-- Created: 2026-03-23

-- Main API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by      UUID NOT NULL REFERENCES users(id),
    key_prefix      VARCHAR(12) NOT NULL,           -- e.g. "kpg_live_"
    key_hash        VARCHAR(64) NOT NULL UNIQUE,    -- SHA-256 of full key
    name            VARCHAR(255) NOT NULL,          -- human label
    description     TEXT,
    permissions     TEXT[] NOT NULL DEFAULT '{}',  -- e.g. {"read:buildings", "read:expenses"}
    rate_limit      INTEGER NOT NULL DEFAULT 100,  -- req/minute
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,                   -- NULL = no expiry
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for query optimization
CREATE INDEX IF NOT EXISTS idx_api_keys_org ON api_keys (organization_id, is_active);
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys (key_hash) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_api_keys_organization_active ON api_keys (organization_id, is_active, created_at DESC);

-- API usage audit log (for rate limiting and analytics)
CREATE TABLE IF NOT EXISTS api_key_usage (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id      UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    endpoint        VARCHAR(255) NOT NULL,
    method          VARCHAR(10) NOT NULL,          -- GET, POST, PUT, DELETE
    status_code     SMALLINT NOT NULL,
    response_ms     INTEGER,                       -- Response time in milliseconds
    ip_address      INET,
    user_agent      TEXT,                          -- For audit purposes
    request_body_size INTEGER,                     -- Payload size in bytes
    response_body_size INTEGER,
    called_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for usage analytics
CREATE INDEX IF NOT EXISTS idx_api_usage_key_time
    ON api_key_usage (api_key_id, called_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_usage_status
    ON api_key_usage (api_key_id, status_code, called_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_usage_endpoint
    ON api_key_usage (api_key_id, endpoint, called_at DESC);

-- Table for tracking API key revocations/suspensions
CREATE TABLE IF NOT EXISTS api_key_audit (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id      UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    action          VARCHAR(50) NOT NULL,          -- "created", "revoked", "rotated", "expired"
    actor_id        UUID REFERENCES users(id),     -- Who performed the action
    reason          TEXT,
    ip_address      INET,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_audit_key
    ON api_key_audit (api_key_id, created_at DESC);

-- Comments for documentation
COMMENT ON TABLE api_keys IS 'Public API v2 authentication keys for third-party integrations (Issue #111)';
COMMENT ON COLUMN api_keys.key_prefix IS 'Prefix of the key (e.g., "kpg_live_"), only shows 12 chars for security';
COMMENT ON COLUMN api_keys.key_hash IS 'SHA-256 hash of the full API key for secure storage (key never stored in plaintext)';
COMMENT ON COLUMN api_keys.permissions IS 'Array of allowed permissions (e.g., {"read:buildings", "write:etats-dates"})';
COMMENT ON COLUMN api_keys.rate_limit IS 'Maximum requests per minute allowed for this key';
COMMENT ON COLUMN api_keys.expires_at IS 'Expiration date; NULL means no expiration';

COMMENT ON TABLE api_key_usage IS 'Audit log of API key usage for rate limiting, analytics, and debugging';
COMMENT ON COLUMN api_key_usage.response_ms IS 'Response time in milliseconds for performance monitoring';
COMMENT ON COLUMN api_key_usage.status_code IS 'HTTP status code returned (200, 401, 404, 429, etc.)';

COMMENT ON TABLE api_key_audit IS 'Audit trail for all API key lifecycle events (creation, revocation, rotation)';
COMMENT ON COLUMN api_key_audit.action IS 'Action performed: created, revoked, rotated, expired, suspended, reactivated';

-- Migration: Proxy/Procuration validation for AG votes (Issue #312)
-- Belgian law Art. 3.87 §6 CC: max 3 mandats per proxy holder
-- Exception: proxy holder cannot represent >10% of total voting quotas

-- Add proxy_count tracking to meetings for validation
ALTER TABLE meetings
    ADD COLUMN IF NOT EXISTS total_quotas INTEGER NOT NULL DEFAULT 1000,
    ADD COLUMN IF NOT EXISTS quorum_threshold_pct NUMERIC(5,2) DEFAULT 50.00;

-- Track proxy mandate counts per meeting
CREATE TABLE IF NOT EXISTS meeting_proxy_mandates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id      UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    proxy_holder_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    grantor_owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    voting_power    INTEGER NOT NULL DEFAULT 0,  -- tantièmes delegated
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (meeting_id, proxy_holder_id, grantor_owner_id)
);

-- Index for quick proxy count validation
CREATE INDEX IF NOT EXISTS idx_proxy_mandates_meeting_proxy
    ON meeting_proxy_mandates (meeting_id, proxy_holder_id)
    WHERE is_active = TRUE;

-- View for proxy mandate statistics per meeting
CREATE OR REPLACE VIEW proxy_mandate_stats AS
SELECT
    meeting_id,
    proxy_holder_id,
    COUNT(*) as mandate_count,
    SUM(voting_power) as total_delegated_quotas
FROM meeting_proxy_mandates
WHERE is_active = TRUE
GROUP BY meeting_id, proxy_holder_id;

COMMENT ON TABLE meeting_proxy_mandates IS 'Tracks proxy/procuration mandates per AG meeting (Art. 3.87 §6 CC)';
COMMENT ON VIEW proxy_mandate_stats IS 'Aggregate proxy mandate stats for validation (max 3 mandats, max 10% quotas)';

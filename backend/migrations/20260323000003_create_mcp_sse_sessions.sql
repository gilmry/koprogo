-- Migration: MCP SSE Session tracking (Issue #252)
-- Tracks active MCP connections from AI clients (Claude, etc.)

CREATE TABLE IF NOT EXISTS mcp_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL UNIQUE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    client_name     VARCHAR(100),                 -- e.g. "claude-desktop", "cursor"
    client_version  VARCHAR(50),
    protocol_version VARCHAR(20) DEFAULT '2024-11-05',
    connected_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    disconnected_at TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    tools_called    INTEGER NOT NULL DEFAULT 0,   -- number of tool calls in session
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding active sessions per user
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_user_id
    ON mcp_sessions (user_id)
    WHERE is_active = TRUE;

-- Index for session lookup by session_id
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_session_id
    ON mcp_sessions (session_id);

-- Index for organization-level analytics
CREATE INDEX IF NOT EXISTS idx_mcp_sessions_org_id
    ON mcp_sessions (organization_id, connected_at);

-- MCP tool usage audit log (per-tool call tracking)
CREATE TABLE IF NOT EXISTS mcp_tool_calls (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL REFERENCES mcp_sessions(session_id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    tool_name       VARCHAR(100) NOT NULL,
    arguments       JSONB DEFAULT '{}',
    success         BOOLEAN NOT NULL DEFAULT TRUE,
    error_code      INTEGER,
    error_message   TEXT,
    duration_ms     INTEGER,                      -- execution time in milliseconds
    called_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for usage analytics per tool
CREATE INDEX IF NOT EXISTS idx_mcp_tool_calls_tool_name
    ON mcp_tool_calls (tool_name, called_at);

-- Index for user activity audit
CREATE INDEX IF NOT EXISTS idx_mcp_tool_calls_user_id
    ON mcp_tool_calls (user_id, called_at);

-- Comments
COMMENT ON TABLE mcp_sessions IS 'Active MCP/SSE connections from AI clients (Issue #252)';
COMMENT ON TABLE mcp_tool_calls IS 'Audit log of all MCP tool calls — GDPR Art. 30 compliance (Issue #252)';
COMMENT ON COLUMN mcp_sessions.protocol_version IS 'MCP protocol version (e.g. 2024-11-05)';
COMMENT ON COLUMN mcp_sessions.tools_called IS 'Running count of tool calls in this session';

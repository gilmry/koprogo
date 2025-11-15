-- MCP Tables - Model Context Protocol persistence
-- Migration: 20250202000000_create_mcp_tables

-- MCP Models Registry
CREATE TABLE IF NOT EXISTS mcp_models (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider VARCHAR(50) NOT NULL, -- JSON serialized ModelProvider
    context_length INTEGER NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT true,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    edge_compatible BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_mcp_models_provider ON mcp_models(provider);
CREATE INDEX idx_mcp_models_available ON mcp_models(is_available);
CREATE INDEX idx_mcp_models_edge ON mcp_models(edge_compatible);

-- MCP Requests Log
CREATE TABLE IF NOT EXISTS mcp_requests (
    id UUID PRIMARY KEY,
    model VARCHAR(255) NOT NULL,
    messages JSONB NOT NULL, -- Array of Message objects
    context VARCHAR(255), -- e.g., "copro:123"
    max_tokens INTEGER,
    temperature REAL,
    stream BOOLEAN NOT NULL DEFAULT false,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_mcp_requests_user ON mcp_requests(user_id);
CREATE INDEX idx_mcp_requests_context ON mcp_requests(context);
CREATE INDEX idx_mcp_requests_model ON mcp_requests(model);
CREATE INDEX idx_mcp_requests_created ON mcp_requests(created_at DESC);

-- MCP Responses Log
CREATE TABLE IF NOT EXISTS mcp_responses (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES mcp_requests(id) ON DELETE CASCADE,
    model VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    finish_reason VARCHAR(50) NOT NULL, -- JSON serialized FinishReason
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    execution_info JSONB NOT NULL, -- ExecutionInfo object
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_mcp_responses_request ON mcp_responses(request_id);
CREATE INDEX idx_mcp_responses_created ON mcp_responses(created_at DESC);
CREATE INDEX idx_mcp_responses_exec_type ON mcp_responses((execution_info->>'execution_type'));

-- MCP Tasks (Grid Computing)
CREATE TABLE IF NOT EXISTS mcp_tasks (
    id UUID PRIMARY KEY,
    task_type VARCHAR(50) NOT NULL, -- JSON serialized TaskType
    input_data JSONB NOT NULL,
    copro_id UUID REFERENCES buildings(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- JSON serialized TaskStatus
    result JSONB,
    assigned_node VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_mcp_tasks_status ON mcp_tasks(status);
CREATE INDEX idx_mcp_tasks_copro ON mcp_tasks(copro_id);
CREATE INDEX idx_mcp_tasks_created ON mcp_tasks(created_at DESC);

-- Seed default models
INSERT INTO mcp_models (id, name, provider, context_length, is_available, supports_streaming, edge_compatible) VALUES
    ('llama3:8b', 'Llama 3 8B Instruct', '"local"', 8192, true, true, true),
    ('llama3:8b-instruct-q4', 'Llama 3 8B Instruct Q4', '"local"', 8192, true, true, true),
    ('mistral:7b-instruct-q4', 'Mistral 7B Instruct Q4', '"local"', 8192, true, true, true),
    ('phi-2:2.7b-q4', 'Phi-2 2.7B Q4', '"local"', 2048, true, true, true),
    ('claude-3-opus', 'Claude 3 Opus', '"anthropic"', 200000, true, true, false),
    ('claude-3-sonnet', 'Claude 3 Sonnet', '"anthropic"', 200000, true, true, false),
    ('claude-3-haiku', 'Claude 3 Haiku', '"anthropic"', 200000, true, true, false),
    ('gpt-4o', 'GPT-4o', '"openai"', 128000, true, true, false),
    ('gpt-4o-mini', 'GPT-4o Mini', '"openai"', 128000, true, true, false)
ON CONFLICT (id) DO NOTHING;

-- Comments for documentation
COMMENT ON TABLE mcp_models IS 'Registry of available AI models (local and cloud)';
COMMENT ON TABLE mcp_requests IS 'Log of all MCP chat requests';
COMMENT ON TABLE mcp_responses IS 'Log of all MCP responses with execution info';
COMMENT ON TABLE mcp_tasks IS 'Grid computing tasks for distributed AI workloads';

COMMENT ON COLUMN mcp_requests.context IS 'Resource context (e.g., copro:123, building:456)';
COMMENT ON COLUMN mcp_responses.execution_info IS 'Execution metadata (type: edge/cloud/grid, latency, node_id)';
COMMENT ON COLUMN mcp_tasks.task_type IS 'Type of task: ocr_invoice, translate_document, summarize_meeting, predict_expenses';

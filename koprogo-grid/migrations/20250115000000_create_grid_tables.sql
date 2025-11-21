-- KoproGo Grid Computing - Database Schema
-- This migration creates all tables needed for the decentralized green grid

-- Nodes table: Represents compute nodes in the grid
CREATE TABLE IF NOT EXISTS grid_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    cpu_cores INTEGER NOT NULL CHECK (cpu_cores > 0 AND cpu_cores <= 256),
    has_solar BOOLEAN NOT NULL DEFAULT FALSE,
    location VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    eco_score DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (eco_score >= 0.0 AND eco_score <= 1.0),
    total_energy_saved_wh DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (total_energy_saved_wh >= 0.0),
    total_carbon_credits DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (total_carbon_credits >= 0.0),
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_status CHECK (status IN ('active', 'idle', 'offline', 'suspended'))
);

-- Tasks table: Represents computational tasks
CREATE TABLE IF NOT EXISTS grid_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    assigned_node_id UUID REFERENCES grid_nodes(id) ON DELETE SET NULL,
    data_url TEXT NOT NULL,
    result_hash TEXT,
    deadline TIMESTAMP WITH TIME ZONE NOT NULL,
    energy_used_wh DOUBLE PRECISION CHECK (energy_used_wh >= 0.0),
    carbon_credits_awarded DOUBLE PRECISION CHECK (carbon_credits_awarded >= 0.0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_task_type CHECK (task_type IN ('ml_train', 'data_hash', 'render', 'scientific')),
    CONSTRAINT valid_task_status CHECK (status IN ('pending', 'assigned', 'in_progress', 'completed', 'failed', 'expired'))
);

-- Green Proofs table: Blockchain Proof of Green entries
CREATE TABLE IF NOT EXISTS grid_green_proofs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES grid_tasks(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES grid_nodes(id) ON DELETE CASCADE,
    block_hash VARCHAR(64) NOT NULL UNIQUE,
    previous_hash VARCHAR(64),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    energy_used_wh DOUBLE PRECISION NOT NULL CHECK (energy_used_wh >= 0.0),
    solar_contribution_wh DOUBLE PRECISION NOT NULL CHECK (solar_contribution_wh >= 0.0),
    carbon_saved_kg DOUBLE PRECISION NOT NULL CHECK (carbon_saved_kg >= 0.0),
    nonce BIGINT NOT NULL,

    CONSTRAINT solar_not_exceed_total CHECK (solar_contribution_wh <= energy_used_wh)
);

-- Carbon Credits table: Tracks carbon credits earned
CREATE TABLE IF NOT EXISTS grid_carbon_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES grid_nodes(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES grid_tasks(id) ON DELETE CASCADE,
    proof_id UUID NOT NULL REFERENCES grid_green_proofs(id) ON DELETE CASCADE,
    amount_kg_co2 DOUBLE PRECISION NOT NULL CHECK (amount_kg_co2 >= 0.0),
    euro_value DOUBLE PRECISION NOT NULL CHECK (euro_value >= 0.0),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    cooperative_share DOUBLE PRECISION NOT NULL CHECK (cooperative_share >= 0.0),
    node_share DOUBLE PRECISION NOT NULL CHECK (node_share >= 0.0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    redeemed_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT valid_credit_status CHECK (status IN ('pending', 'verified', 'redeemed', 'expired'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_nodes_status ON grid_nodes(status);
CREATE INDEX IF NOT EXISTS idx_nodes_location ON grid_nodes(location);
CREATE INDEX IF NOT EXISTS idx_nodes_eco_score ON grid_nodes(eco_score DESC);
CREATE INDEX IF NOT EXISTS idx_nodes_last_heartbeat ON grid_nodes(last_heartbeat);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON grid_tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_node ON grid_tasks(assigned_node_id);
CREATE INDEX IF NOT EXISTS idx_tasks_deadline ON grid_tasks(deadline);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON grid_tasks(created_at);

CREATE INDEX IF NOT EXISTS idx_proofs_task ON grid_green_proofs(task_id);
CREATE INDEX IF NOT EXISTS idx_proofs_node ON grid_green_proofs(node_id);
CREATE INDEX IF NOT EXISTS idx_proofs_timestamp ON grid_green_proofs(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_credits_node ON grid_carbon_credits(node_id);
CREATE INDEX IF NOT EXISTS idx_credits_status ON grid_carbon_credits(status);
CREATE INDEX IF NOT EXISTS idx_credits_created_at ON grid_carbon_credits(created_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for nodes table
DROP TRIGGER IF EXISTS update_grid_nodes_updated_at ON grid_nodes;
CREATE TRIGGER update_grid_nodes_updated_at
    BEFORE UPDATE ON grid_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE grid_nodes IS 'Compute nodes participating in the KoproGo green grid';
COMMENT ON TABLE grid_tasks IS 'Computational tasks distributed across the grid';
COMMENT ON TABLE grid_green_proofs IS 'Blockchain Proof of Green validations';
COMMENT ON TABLE grid_carbon_credits IS 'Carbon credits earned through green computing';

COMMENT ON COLUMN grid_nodes.eco_score IS 'Ecological score (0.0-1.0) based on CPU idle and solar contribution';
COMMENT ON COLUMN grid_green_proofs.block_hash IS 'SHA-256 hash with proof of work (minimum 1 leading zero)';
COMMENT ON COLUMN grid_carbon_credits.cooperative_share IS 'Cooperative solidarity fund share (30%)';
COMMENT ON COLUMN grid_carbon_credits.node_share IS 'Node owner share (70%)';

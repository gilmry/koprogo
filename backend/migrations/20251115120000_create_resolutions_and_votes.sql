-- Migration: Create Resolutions and Votes tables for Meeting Voting System (Issue #46)
-- Phase 2: K3s + Automation
-- Belgian copropriété law compliance for vote tracking

-- Create resolutions table
CREATE TABLE IF NOT EXISTS resolutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    resolution_type VARCHAR(50) NOT NULL, -- 'Ordinary', 'Extraordinary'
    majority_required VARCHAR(50) NOT NULL, -- 'Simple', 'Absolute', 'Qualified:0.67'
    vote_count_pour INT NOT NULL DEFAULT 0,
    vote_count_contre INT NOT NULL DEFAULT 0,
    vote_count_abstention INT NOT NULL DEFAULT 0,
    total_voting_power_pour DECIMAL(10,4) NOT NULL DEFAULT 0,
    total_voting_power_contre DECIMAL(10,4) NOT NULL DEFAULT 0,
    total_voting_power_abstention DECIMAL(10,4) NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending', -- 'Pending', 'Adopted', 'Rejected'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    voted_at TIMESTAMP,

    CONSTRAINT resolutions_title_not_empty CHECK (LENGTH(TRIM(title)) > 0),
    CONSTRAINT resolutions_description_not_empty CHECK (LENGTH(TRIM(description)) > 0),
    CONSTRAINT resolutions_valid_type CHECK (resolution_type IN ('Ordinary', 'Extraordinary')),
    CONSTRAINT resolutions_valid_status CHECK (status IN ('Pending', 'Adopted', 'Rejected')),
    CONSTRAINT resolutions_non_negative_votes CHECK (
        vote_count_pour >= 0 AND
        vote_count_contre >= 0 AND
        vote_count_abstention >= 0
    ),
    CONSTRAINT resolutions_non_negative_voting_power CHECK (
        total_voting_power_pour >= 0 AND
        total_voting_power_contre >= 0 AND
        total_voting_power_abstention >= 0
    )
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resolution_id UUID NOT NULL REFERENCES resolutions(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    vote_choice VARCHAR(50) NOT NULL, -- 'Pour', 'Contre', 'Abstention'
    voting_power DECIMAL(10,4) NOT NULL,
    proxy_owner_id UUID REFERENCES owners(id) ON DELETE SET NULL, -- Mandataire si vote par procuration
    voted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT votes_valid_choice CHECK (vote_choice IN ('Pour', 'Contre', 'Abstention')),
    CONSTRAINT votes_positive_voting_power CHECK (voting_power > 0),
    CONSTRAINT votes_max_voting_power CHECK (voting_power <= 1000), -- Max 1000 millièmes
    CONSTRAINT votes_no_self_proxy CHECK (owner_id != proxy_owner_id),
    CONSTRAINT votes_unique_per_unit_resolution UNIQUE (resolution_id, unit_id)
);

-- Create indexes for performance
CREATE INDEX idx_resolutions_meeting ON resolutions(meeting_id);
CREATE INDEX idx_resolutions_status ON resolutions(status);
CREATE INDEX idx_resolutions_created_at ON resolutions(created_at DESC);

CREATE INDEX idx_votes_resolution ON votes(resolution_id);
CREATE INDEX idx_votes_owner ON votes(owner_id);
CREATE INDEX idx_votes_unit ON votes(unit_id);
CREATE INDEX idx_votes_proxy ON votes(proxy_owner_id) WHERE proxy_owner_id IS NOT NULL;
CREATE INDEX idx_votes_voted_at ON votes(voted_at DESC);

-- Comments for documentation
COMMENT ON TABLE resolutions IS 'Résolutions soumises au vote lors des assemblées générales';
COMMENT ON TABLE votes IS 'Votes individuels des copropriétaires sur les résolutions';

COMMENT ON COLUMN resolutions.resolution_type IS 'Ordinary (majorité simple) ou Extraordinary (majorité qualifiée)';
COMMENT ON COLUMN resolutions.majority_required IS 'Type de majorité: Simple, Absolute, ou Qualified:threshold';
COMMENT ON COLUMN resolutions.total_voting_power_pour IS 'Somme des millièmes/tantièmes des votes "Pour"';
COMMENT ON COLUMN resolutions.total_voting_power_contre IS 'Somme des millièmes/tantièmes des votes "Contre"';
COMMENT ON COLUMN resolutions.total_voting_power_abstention IS 'Somme des millièmes/tantièmes des abstentions';

COMMENT ON COLUMN votes.voting_power IS 'Pouvoir de vote en millièmes/tantièmes du lot';
COMMENT ON COLUMN votes.proxy_owner_id IS 'ID du mandataire si vote par procuration (NULL si vote personnel)';
COMMENT ON CONSTRAINT votes_unique_per_unit_resolution ON votes IS 'Un seul vote par lot par résolution';

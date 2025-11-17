-- Create local_exchanges table for SEL (Système d'Échange Local)
-- Belgian Local Exchange Trading System with time-based currency

-- Create custom ENUM for exchange type
CREATE TYPE exchange_type AS ENUM (
    'Service',        -- Skills (plumbing, gardening, tutoring, etc.)
    'ObjectLoan',     -- Temporary loan (tools, books, equipment)
    'SharedPurchase'  -- Co-buying (bulk food, equipment rental)
);

-- Create custom ENUM for exchange status
CREATE TYPE exchange_status AS ENUM (
    'Offered',     -- Available for anyone to request
    'Requested',   -- Someone claimed it (pending provider acceptance)
    'InProgress',  -- Exchange is happening
    'Completed',   -- Both parties confirmed completion
    'Cancelled'    -- Exchange was cancelled
);

CREATE TABLE local_exchanges (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    requester_id UUID REFERENCES owners(id) ON DELETE SET NULL,
    exchange_type exchange_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    credits INT NOT NULL CHECK (credits > 0 AND credits <= 100),
    status exchange_status NOT NULL DEFAULT 'Offered',

    -- Timestamps
    offered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    requested_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,

    -- Ratings (1-5 stars)
    provider_rating INT CHECK (provider_rating >= 1 AND provider_rating <= 5),
    requester_rating INT CHECK (requester_rating >= 1 AND requester_rating <= 5),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_title_not_empty CHECK (LENGTH(TRIM(title)) > 0),
    CONSTRAINT check_description_not_empty CHECK (LENGTH(TRIM(description)) > 0),
    CONSTRAINT check_requester_not_provider CHECK (provider_id != requester_id)
);

-- Indexes for performance
CREATE INDEX idx_local_exchanges_building_id ON local_exchanges(building_id);
CREATE INDEX idx_local_exchanges_provider_id ON local_exchanges(provider_id);
CREATE INDEX idx_local_exchanges_requester_id ON local_exchanges(requester_id);
CREATE INDEX idx_local_exchanges_status ON local_exchanges(status);
CREATE INDEX idx_local_exchanges_exchange_type ON local_exchanges(exchange_type);
CREATE INDEX idx_local_exchanges_offered_at ON local_exchanges(offered_at DESC);
CREATE INDEX idx_local_exchanges_building_status ON local_exchanges(building_id, status);

-- Partial index for active exchanges (marketplace view optimization)
CREATE INDEX idx_local_exchanges_active ON local_exchanges(building_id, offered_at DESC)
WHERE status IN ('Offered', 'Requested', 'InProgress');

-- Partial index for completed exchanges (statistics optimization)
CREATE INDEX idx_local_exchanges_completed ON local_exchanges(building_id)
WHERE status = 'Completed';

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_local_exchanges_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_local_exchanges_updated_at
    BEFORE UPDATE ON local_exchanges
    FOR EACH ROW
    EXECUTE FUNCTION update_local_exchanges_updated_at();

-- Comments for documentation
COMMENT ON TABLE local_exchanges IS 'SEL (Système d''Échange Local) - Local Exchange Trading System with time-based currency (1 hour = 1 credit)';
COMMENT ON COLUMN local_exchanges.credits IS 'Time-based credits (1 hour = 1 credit, max 100 hours per exchange)';
COMMENT ON COLUMN local_exchanges.status IS 'Exchange lifecycle: Offered → Requested → InProgress → Completed / Cancelled';
COMMENT ON COLUMN local_exchanges.provider_rating IS 'Rating of provider by requester (1-5 stars)';
COMMENT ON COLUMN local_exchanges.requester_rating IS 'Rating of requester by provider (1-5 stars)';

-- Create owner_credit_balances table
CREATE TABLE owner_credit_balances (
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    credits_earned INT NOT NULL DEFAULT 0 CHECK (credits_earned >= 0),
    credits_spent INT NOT NULL DEFAULT 0 CHECK (credits_spent >= 0),
    balance INT NOT NULL DEFAULT 0, -- Can be negative (trust model)
    total_exchanges INT NOT NULL DEFAULT 0 CHECK (total_exchanges >= 0),
    average_rating REAL CHECK (average_rating >= 1.0 AND average_rating <= 5.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (owner_id, building_id)
);

-- Indexes for credit balances
CREATE INDEX idx_owner_credit_balances_building_id ON owner_credit_balances(building_id);
CREATE INDEX idx_owner_credit_balances_owner_id ON owner_credit_balances(owner_id);
CREATE INDEX idx_owner_credit_balances_balance ON owner_credit_balances(building_id, balance DESC);

-- Partial index for active participants (statistics optimization)
CREATE INDEX idx_owner_credit_balances_active ON owner_credit_balances(building_id)
WHERE total_exchanges > 0;

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_owner_credit_balances_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_owner_credit_balances_updated_at
    BEFORE UPDATE ON owner_credit_balances
    FOR EACH ROW
    EXECUTE FUNCTION update_owner_credit_balances_updated_at();

-- Comments for documentation
COMMENT ON TABLE owner_credit_balances IS 'Credit balance tracking for SEL (Local Exchange Trading System)';
COMMENT ON COLUMN owner_credit_balances.credits_earned IS 'Total credits earned by providing services';
COMMENT ON COLUMN owner_credit_balances.credits_spent IS 'Total credits spent by receiving services';
COMMENT ON COLUMN owner_credit_balances.balance IS 'Current balance (earned - spent), can be negative (trust model)';
COMMENT ON COLUMN owner_credit_balances.total_exchanges IS 'Total number of completed exchanges (as provider or requester)';
COMMENT ON COLUMN owner_credit_balances.average_rating IS 'Average rating received (1-5 stars) across all completed exchanges';

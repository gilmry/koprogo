-- Create Budgets table (Annual budgets for buildings - Belgian legal requirement)
-- Must be voted by General Assembly each fiscal year

-- Create ENUM for budget status
CREATE TYPE budget_status AS ENUM ('draft', 'submitted', 'approved', 'rejected', 'archived');

-- Create main table
CREATE TABLE budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Fiscal year (e.g., 2025)
    fiscal_year INT NOT NULL CHECK (fiscal_year >= 2000 AND fiscal_year <= 2100),

    -- Budget amounts (in EUR)
    ordinary_budget DECIMAL(12,2) NOT NULL CHECK (ordinary_budget >= 0),
    extraordinary_budget DECIMAL(12,2) NOT NULL CHECK (extraordinary_budget >= 0),
    total_budget DECIMAL(12,2) NOT NULL CHECK (total_budget >= 0),

    -- Workflow status
    status budget_status NOT NULL DEFAULT 'draft',

    -- Workflow dates
    submitted_date TIMESTAMPTZ,
    approved_date TIMESTAMPTZ,

    -- Legal traceability: which AG meeting approved this budget
    approved_by_meeting_id UUID REFERENCES meetings(id) ON DELETE SET NULL,

    -- Monthly provision amount (total_budget / 12)
    monthly_provision_amount DECIMAL(12,2) NOT NULL CHECK (monthly_provision_amount >= 0),

    -- Optional notes
    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Business constraint: one budget per building per fiscal year
    UNIQUE(building_id, fiscal_year)
);

-- Indexes for performance
CREATE INDEX idx_budgets_organization ON budgets(organization_id);
CREATE INDEX idx_budgets_building ON budgets(building_id);
CREATE INDEX idx_budgets_fiscal_year ON budgets(fiscal_year);
CREATE INDEX idx_budgets_status ON budgets(status);
CREATE INDEX idx_budgets_building_fiscal_year ON budgets(building_id, fiscal_year);

-- Partial index for active budgets (approved, most recent)
CREATE INDEX idx_budgets_active ON budgets(building_id, fiscal_year DESC)
    WHERE status = 'approved';

-- Comments for documentation
COMMENT ON TABLE budgets IS 'Annual budgets for buildings (must be voted by AG each fiscal year)';
COMMENT ON COLUMN budgets.fiscal_year IS 'Fiscal year (e.g., 2025)';
COMMENT ON COLUMN budgets.ordinary_budget IS 'Budget for ordinary charges (routine maintenance, insurance, etc.)';
COMMENT ON COLUMN budgets.extraordinary_budget IS 'Budget for extraordinary charges (major works, renovations, etc.)';
COMMENT ON COLUMN budgets.total_budget IS 'Total budget (ordinary + extraordinary)';
COMMENT ON COLUMN budgets.monthly_provision_amount IS 'Monthly provision amount per unit (total_budget / 12)';
COMMENT ON COLUMN budgets.approved_by_meeting_id IS 'Reference to AG meeting that approved this budget (legal traceability)';
COMMENT ON COLUMN budgets.status IS 'Workflow: draft → submitted → approved/rejected → archived';

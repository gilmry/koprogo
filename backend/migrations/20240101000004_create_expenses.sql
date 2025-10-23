-- Create expenses table
CREATE TYPE expense_category AS ENUM ('maintenance', 'repairs', 'insurance', 'utilities', 'cleaning', 'administration', 'works', 'other');
CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'overdue', 'cancelled');

CREATE TABLE expenses (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    category expense_category NOT NULL,
    description TEXT NOT NULL,
    amount DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    expense_date TIMESTAMPTZ NOT NULL,
    payment_status payment_status NOT NULL DEFAULT 'pending',
    supplier VARCHAR(255),
    invoice_number VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_expenses_building ON expenses(building_id);
CREATE INDEX idx_expenses_date ON expenses(expense_date);
CREATE INDEX idx_expenses_status ON expenses(payment_status);
CREATE INDEX idx_expenses_category ON expenses(category);

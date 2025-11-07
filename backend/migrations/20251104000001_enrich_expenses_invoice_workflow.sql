-- Migration: Enrich Expenses for Complete Invoice/Billing System with Workflow
-- Issue #73: Système Complet d'Encodage de Factures avec Workflow de Validation
-- Date: 2025-11-04

-- ============================================================================
-- 1. Create approval_status ENUM for workflow
-- ============================================================================
CREATE TYPE approval_status AS ENUM ('draft', 'pending_approval', 'approved', 'rejected');

-- ============================================================================
-- 2. Add new columns to expenses table
-- ============================================================================

-- VAT management fields
ALTER TABLE expenses
    ADD COLUMN amount_excl_vat DECIMAL(10,2), -- Montant HT
    ADD COLUMN vat_rate DECIMAL(5,2),         -- Taux TVA (ex: 21.00 pour 21%)
    ADD COLUMN vat_amount DECIMAL(10,2),      -- Montant TVA
    ADD COLUMN amount_incl_vat DECIMAL(10,2); -- Montant TTC (explicite)

-- Multiple dates fields
ALTER TABLE expenses
    ADD COLUMN invoice_date TIMESTAMPTZ,   -- Date de la facture
    ADD COLUMN due_date TIMESTAMPTZ,       -- Date d'échéance
    ADD COLUMN paid_date TIMESTAMPTZ;      -- Date de paiement effectif

-- Workflow fields
ALTER TABLE expenses
    ADD COLUMN approval_status approval_status NOT NULL DEFAULT 'draft',
    ADD COLUMN submitted_at TIMESTAMPTZ,                    -- Date de soumission
    ADD COLUMN approved_by UUID REFERENCES users(id),       -- User ID qui a approuvé/rejeté
    ADD COLUMN approved_at TIMESTAMPTZ,                     -- Date d'approbation/rejet
    ADD COLUMN rejection_reason TEXT;                       -- Raison du rejet

-- ============================================================================
-- 3. Create invoice_line_items table
-- ============================================================================
CREATE TABLE invoice_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity DECIMAL(10,2) NOT NULL DEFAULT 1.0,
    unit_price DECIMAL(10,2) NOT NULL,

    -- Calculated amounts
    amount_excl_vat DECIMAL(10,2) NOT NULL,
    vat_rate DECIMAL(5,2) NOT NULL,
    vat_amount DECIMAL(10,2) NOT NULL,
    amount_incl_vat DECIMAL(10,2) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT positive_quantity CHECK (quantity > 0),
    CONSTRAINT non_negative_unit_price CHECK (unit_price >= 0),
    CONSTRAINT valid_vat_rate CHECK (vat_rate >= 0 AND vat_rate <= 100)
);

-- ============================================================================
-- 4. Create charge_distributions table
-- ============================================================================
CREATE TABLE charge_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,

    quota_percentage DECIMAL(5,4) NOT NULL, -- Quote-part (ex: 0.2500 pour 25%)
    amount_due DECIMAL(10,2) NOT NULL,      -- Montant à payer

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_expense_unit UNIQUE(expense_id, unit_id),
    CONSTRAINT valid_quota_percentage CHECK (quota_percentage >= 0 AND quota_percentage <= 1),
    CONSTRAINT non_negative_amount_due CHECK (amount_due >= 0)
);

-- ============================================================================
-- 5. Create indexes for performance
-- ============================================================================

-- Indexes on expenses table
CREATE INDEX idx_expenses_approval_status ON expenses(approval_status);
CREATE INDEX idx_expenses_invoice_date ON expenses(invoice_date) WHERE invoice_date IS NOT NULL;
CREATE INDEX idx_expenses_due_date ON expenses(due_date) WHERE due_date IS NOT NULL;
CREATE INDEX idx_expenses_paid_date ON expenses(paid_date) WHERE paid_date IS NOT NULL;
CREATE INDEX idx_expenses_approved_by ON expenses(approved_by) WHERE approved_by IS NOT NULL;

-- Indexes on invoice_line_items
CREATE INDEX idx_invoice_line_items_expense ON invoice_line_items(expense_id);

-- Indexes on charge_distributions
CREATE INDEX idx_charge_distributions_expense ON charge_distributions(expense_id);
CREATE INDEX idx_charge_distributions_unit ON charge_distributions(unit_id);
CREATE INDEX idx_charge_distributions_owner ON charge_distributions(owner_id);

-- ============================================================================
-- 6. Add comments for documentation
-- ============================================================================

COMMENT ON COLUMN expenses.amount_excl_vat IS 'Montant hors TVA (HT)';
COMMENT ON COLUMN expenses.vat_rate IS 'Taux de TVA en pourcentage (ex: 21.00 pour 21%)';
COMMENT ON COLUMN expenses.vat_amount IS 'Montant de la TVA';
COMMENT ON COLUMN expenses.amount_incl_vat IS 'Montant toutes taxes comprises (TTC)';
COMMENT ON COLUMN expenses.invoice_date IS 'Date de la facture';
COMMENT ON COLUMN expenses.due_date IS 'Date d''échéance de paiement';
COMMENT ON COLUMN expenses.paid_date IS 'Date de paiement effectif';
COMMENT ON COLUMN expenses.approval_status IS 'Statut du workflow de validation (draft/pending_approval/approved/rejected)';
COMMENT ON COLUMN expenses.submitted_at IS 'Date de soumission pour validation';
COMMENT ON COLUMN expenses.approved_by IS 'ID de l''utilisateur qui a approuvé ou rejeté';
COMMENT ON COLUMN expenses.approved_at IS 'Date d''approbation ou de rejet';
COMMENT ON COLUMN expenses.rejection_reason IS 'Raison du rejet (si rejected)';

COMMENT ON TABLE invoice_line_items IS 'Lignes de facture détaillées pour chaque expense';
COMMENT ON COLUMN invoice_line_items.quantity IS 'Quantité (ex: 2.5 pour 2.5 heures ou 2.5m²)';
COMMENT ON COLUMN invoice_line_items.unit_price IS 'Prix unitaire hors TVA';

COMMENT ON TABLE charge_distributions IS 'Répartition automatique des charges par lot/propriétaire basée sur les quotes-parts';
COMMENT ON COLUMN charge_distributions.quota_percentage IS 'Quote-part du lot (entre 0 et 1, ex: 0.25 pour 25%)';
COMMENT ON COLUMN charge_distributions.amount_due IS 'Montant dû par ce propriétaire pour cette charge';

-- ============================================================================
-- 7. Migrate existing data (backward compatibility)
-- ============================================================================

-- Copier le montant existant dans amount_incl_vat pour les expenses existantes
UPDATE expenses
SET amount_incl_vat = amount
WHERE amount_incl_vat IS NULL AND amount IS NOT NULL;

-- Copier expense_date dans invoice_date pour les expenses existantes
UPDATE expenses
SET invoice_date = expense_date
WHERE invoice_date IS NULL AND expense_date IS NOT NULL;

-- Mettre à jour paid_date pour les expenses déjà payées
-- (on utilise updated_at comme approximation)
UPDATE expenses
SET paid_date = updated_at
WHERE paid_date IS NULL
  AND payment_status = 'paid';

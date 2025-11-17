-- Migration: Create payments and payment_methods tables
-- Issue #84 - Online Payment Integration (Stripe + SEPA)
-- Belgian property management payment processing

-- =====================================================================
-- CUSTOM TYPES
-- =====================================================================

CREATE TYPE transaction_status AS ENUM (
    'pending',
    'processing',
    'requires_action',
    'succeeded',
    'failed',
    'cancelled',
    'refunded'
);

CREATE TYPE payment_method_type AS ENUM (
    'card',
    'sepa_debit',
    'bank_transfer',
    'cash'
);

-- =====================================================================
-- PAYMENTS TABLE
-- =====================================================================

CREATE TABLE payments (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    building_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    expense_id UUID, -- Optional: could be general account credit

    -- Payment amount (in cents, EUR)
    amount_cents BIGINT NOT NULL CHECK (amount_cents > 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'EUR' CHECK (currency = 'EUR'),

    -- Payment status
    status transaction_status NOT NULL DEFAULT 'pending',
    payment_method_type payment_method_type NOT NULL,

    -- Stripe integration
    stripe_payment_intent_id VARCHAR(255) UNIQUE, -- Stripe payment intent ID
    stripe_customer_id VARCHAR(255),
    payment_method_id UUID, -- Reference to stored payment method

    -- Idempotency (prevents duplicate charges)
    idempotency_key VARCHAR(255) NOT NULL,

    -- Metadata
    description TEXT,
    metadata JSONB, -- Extensible metadata

    -- Failure tracking
    failure_reason TEXT,

    -- Refund tracking
    refunded_amount_cents BIGINT NOT NULL DEFAULT 0 CHECK (refunded_amount_cents >= 0),

    -- Timestamp tracking
    succeeded_at TIMESTAMP WITH TIME ZONE,
    failed_at TIMESTAMP WITH TIME ZONE,
    cancelled_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys
    CONSTRAINT fk_payments_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_payments_building FOREIGN KEY (building_id) REFERENCES buildings(id) ON DELETE CASCADE,
    CONSTRAINT fk_payments_owner FOREIGN KEY (owner_id) REFERENCES owners(id) ON DELETE CASCADE,
    CONSTRAINT fk_payments_expense FOREIGN KEY (expense_id) REFERENCES expenses(id) ON DELETE SET NULL,

    -- Business constraints
    CONSTRAINT chk_refund_not_exceed_amount CHECK (refunded_amount_cents <= amount_cents),
    CONSTRAINT chk_succeeded_at_when_succeeded CHECK (
        (status = 'succeeded' AND succeeded_at IS NOT NULL) OR
        (status != 'succeeded' AND succeeded_at IS NULL)
    ),
    CONSTRAINT chk_failed_at_when_failed CHECK (
        (status = 'failed' AND failed_at IS NOT NULL) OR
        (status != 'failed' AND failed_at IS NULL)
    ),
    CONSTRAINT chk_cancelled_at_when_cancelled CHECK (
        (status = 'cancelled' AND cancelled_at IS NOT NULL) OR
        (status != 'cancelled' AND cancelled_at IS NULL)
    ),
    CONSTRAINT chk_idempotency_key_length CHECK (LENGTH(idempotency_key) >= 16)
);

-- Indexes for performance
CREATE INDEX idx_payments_organization_id ON payments(organization_id);
CREATE INDEX idx_payments_building_id ON payments(building_id);
CREATE INDEX idx_payments_owner_id ON payments(owner_id);
CREATE INDEX idx_payments_expense_id ON payments(expense_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_stripe_payment_intent_id ON payments(stripe_payment_intent_id);
CREATE INDEX idx_payments_stripe_customer_id ON payments(stripe_customer_id);
CREATE INDEX idx_payments_idempotency_key ON payments(organization_id, idempotency_key); -- Composite for uniqueness check
CREATE INDEX idx_payments_created_at ON payments(created_at DESC); -- For recent payments queries

-- Unique constraint for idempotency (per organization)
CREATE UNIQUE INDEX idx_payments_idempotency_unique ON payments(organization_id, idempotency_key);

-- =====================================================================
-- PAYMENT_METHODS TABLE
-- =====================================================================

CREATE TABLE payment_methods (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    owner_id UUID NOT NULL,

    -- Payment method type
    method_type payment_method_type NOT NULL,

    -- Stripe integration (PCI-DSS compliant - no raw card data)
    stripe_payment_method_id VARCHAR(255) NOT NULL, -- Stripe payment method ID
    stripe_customer_id VARCHAR(255) NOT NULL,

    -- Display information
    display_label VARCHAR(255) NOT NULL, -- e.g., "Visa •••• 4242", "SEPA BE68 5390 0754"

    -- Flags
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Metadata (stores last4, brand, expiry, etc. - encrypted by Stripe)
    metadata JSONB,

    -- Expiry (for cards only)
    expires_at TIMESTAMP WITH TIME ZONE,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys
    CONSTRAINT fk_payment_methods_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_payment_methods_owner FOREIGN KEY (owner_id) REFERENCES owners(id) ON DELETE CASCADE,

    -- Business constraints
    CONSTRAINT chk_stripe_payment_method_id_not_empty CHECK (LENGTH(TRIM(stripe_payment_method_id)) > 0),
    CONSTRAINT chk_stripe_customer_id_not_empty CHECK (LENGTH(TRIM(stripe_customer_id)) > 0),
    CONSTRAINT chk_display_label_not_empty CHECK (LENGTH(TRIM(display_label)) > 0)
);

-- Indexes for performance
CREATE INDEX idx_payment_methods_organization_id ON payment_methods(organization_id);
CREATE INDEX idx_payment_methods_owner_id ON payment_methods(owner_id);
CREATE INDEX idx_payment_methods_stripe_customer_id ON payment_methods(stripe_customer_id);
CREATE INDEX idx_payment_methods_active ON payment_methods(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_payment_methods_default ON payment_methods(owner_id, is_default) WHERE is_default = TRUE;

-- Ensure only one default payment method per owner
CREATE UNIQUE INDEX idx_payment_methods_one_default_per_owner ON payment_methods(owner_id)
    WHERE is_default = TRUE AND is_active = TRUE;

-- Foreign key from payments to payment_methods
ALTER TABLE payments
    ADD CONSTRAINT fk_payments_payment_method
    FOREIGN KEY (payment_method_id) REFERENCES payment_methods(id) ON DELETE SET NULL;

-- =====================================================================
-- COMMENTS
-- =====================================================================

COMMENT ON TABLE payments IS 'Payment transactions for expenses (Stripe + SEPA integration)';
COMMENT ON COLUMN payments.amount_cents IS 'Amount in cents (EUR) - Stripe uses smallest currency unit';
COMMENT ON COLUMN payments.idempotency_key IS 'Prevents duplicate charges - must be unique per organization';
COMMENT ON COLUMN payments.stripe_payment_intent_id IS 'Stripe payment intent ID (pi_xxx)';
COMMENT ON COLUMN payments.refunded_amount_cents IS 'Total refunded amount in cents';

COMMENT ON TABLE payment_methods IS 'Stored payment methods (cards, SEPA mandates) - PCI-DSS compliant';
COMMENT ON COLUMN payment_methods.stripe_payment_method_id IS 'Stripe payment method ID (pm_xxx for cards, sepa_debit_xxx for SEPA)';
COMMENT ON COLUMN payment_methods.display_label IS 'User-friendly label for UI (e.g., "Visa •••• 4242")';
COMMENT ON COLUMN payment_methods.metadata IS 'Encrypted card/SEPA details from Stripe (last4, brand, expiry)';
COMMENT ON COLUMN payment_methods.expires_at IS 'Expiry date for cards (NULL for SEPA)';

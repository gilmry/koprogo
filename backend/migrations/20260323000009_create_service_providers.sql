-- Issue #276: Marketplace corps de métier + ContractEvaluation
-- Service Providers and Contract Evaluations for marketplace system

CREATE TYPE IF NOT EXISTS trade_category AS ENUM (
    'Syndic', 'BureauEtude', 'Architecte', 'AssistantMaitreOeuvre', 'IngenieurStabilite',
    'Plombier', 'Electricien', 'Chauffagiste', 'Menuisier', 'Peintre', 'Maconnerie',
    'Etancheite', 'Ascensoriste', 'Jardinier', 'Nettoyage', 'Securite', 'Deboucheur',
    'Couvreur', 'Carreleur', 'TechniquesSpeciales'
);

CREATE TABLE IF NOT EXISTS service_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    company_name VARCHAR(200) NOT NULL,
    trade_category trade_category NOT NULL,
    specializations TEXT[] DEFAULT '{}',
    service_zone_postal_codes TEXT[] DEFAULT '{}',
    certifications TEXT[] DEFAULT '{}',
    ipi_registration VARCHAR(50),
    bce_number VARCHAR(20),
    rating_avg NUMERIC(3,2),
    reviews_count INTEGER NOT NULL DEFAULT 0,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    public_profile_slug VARCHAR(200) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS contract_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    service_provider_id UUID NOT NULL REFERENCES service_providers(id) ON DELETE CASCADE,
    quote_id UUID REFERENCES quotes(id) ON DELETE SET NULL,
    ticket_id UUID REFERENCES tickets(id) ON DELETE SET NULL,
    evaluator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    criteria JSONB NOT NULL DEFAULT '{}',
    global_score NUMERIC(3,2) NOT NULL DEFAULT 0,
    comments TEXT,
    would_recommend BOOLEAN NOT NULL DEFAULT TRUE,
    is_legal_evaluation BOOLEAN NOT NULL DEFAULT FALSE,
    is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for service_providers
CREATE INDEX IF NOT EXISTS idx_service_providers_org ON service_providers(organization_id);
CREATE INDEX IF NOT EXISTS idx_service_providers_trade ON service_providers(trade_category);
CREATE INDEX IF NOT EXISTS idx_service_providers_rating ON service_providers(rating_avg DESC NULLS LAST);
CREATE INDEX IF NOT EXISTS idx_service_providers_verified ON service_providers(is_verified) WHERE is_verified = TRUE;
CREATE INDEX IF NOT EXISTS idx_service_providers_slug ON service_providers(public_profile_slug);

-- Indexes for contract_evaluations
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_provider ON contract_evaluations(service_provider_id);
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_building ON contract_evaluations(building_id);
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_legal ON contract_evaluations(building_id, is_legal_evaluation) WHERE is_legal_evaluation = TRUE;
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_evaluator ON contract_evaluations(evaluator_id);
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_quote ON contract_evaluations(quote_id);
CREATE INDEX IF NOT EXISTS idx_contract_evaluations_ticket ON contract_evaluations(ticket_id);

-- Comments for documentation
COMMENT ON TABLE service_providers IS 'Marketplace corps de métier — Issue #276';
COMMENT ON TABLE contract_evaluations IS 'Évaluations contracteurs — Art. 3.89 §5 12° Code Civil Belge (L13 annual report)';
COMMENT ON COLUMN service_providers.trade_category IS '20 professional trade categories (plumber, electrician, etc.)';
COMMENT ON COLUMN service_providers.rating_avg IS 'Average rating from 0.0 to 5.0 (5-star system)';
COMMENT ON COLUMN service_providers.is_verified IS 'Verified by organization (confirmed credentials)';
COMMENT ON COLUMN contract_evaluations.criteria IS 'JSON object with criteria scores: {qualite, delai, prix, communication, proprete, conformite_devis}';
COMMENT ON COLUMN contract_evaluations.is_legal_evaluation IS 'Is part of legal L13 annual compliance report';
COMMENT ON COLUMN contract_evaluations.is_anonymous IS 'GDPR-compliant anonymous evaluation (no evaluator identification)';

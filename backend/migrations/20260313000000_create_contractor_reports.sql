-- Migration: BC16 Backoffice Prestataires PWA
-- Rapport de travaux soumis par le corps de métier via magic link

-- ENUM statut rapport
CREATE TYPE contractor_report_status AS ENUM (
    'draft',
    'submitted',
    'under_review',
    'validated',
    'rejected',
    'requires_correction'
);

-- Table principale
CREATE TABLE contractor_reports (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id         UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id             UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,

    -- Lien ticket et/ou devis (au moins l'un requis)
    ticket_id               UUID REFERENCES tickets(id) ON DELETE SET NULL,
    quote_id                UUID REFERENCES quotes(id) ON DELETE SET NULL,

    -- Identité du prestataire
    contractor_user_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    contractor_name         TEXT NOT NULL CHECK (length(trim(contractor_name)) > 0),

    -- Contenu du rapport
    work_date               TIMESTAMPTZ,
    compte_rendu            TEXT,

    -- Photos avant/après (tableau de document UUIDs)
    photos_before           UUID[] NOT NULL DEFAULT '{}',
    photos_after            UUID[] NOT NULL DEFAULT '{}',

    -- Pièces remplacées (JSON : [{name, reference, quantity, photo_document_id}])
    parts_replaced          JSONB NOT NULL DEFAULT '[]',

    -- Statut machine d'état
    status                  contractor_report_status NOT NULL DEFAULT 'draft',

    -- Magic link PWA
    magic_token_hash        TEXT UNIQUE,
    magic_token_expires_at  TIMESTAMPTZ,

    -- Timestamps de workflow
    submitted_at            TIMESTAMPTZ,
    validated_at            TIMESTAMPTZ,
    validated_by            UUID REFERENCES users(id) ON DELETE SET NULL,
    review_comments         TEXT,

    -- Métadonnées
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Contrainte : au moins ticket_id ou quote_id
    CONSTRAINT ticket_or_quote_required CHECK (ticket_id IS NOT NULL OR quote_id IS NOT NULL)
);

-- Index principaux
CREATE INDEX idx_contractor_reports_organization ON contractor_reports(organization_id);
CREATE INDEX idx_contractor_reports_building ON contractor_reports(building_id);
CREATE INDEX idx_contractor_reports_ticket ON contractor_reports(ticket_id) WHERE ticket_id IS NOT NULL;
CREATE INDEX idx_contractor_reports_quote ON contractor_reports(quote_id) WHERE quote_id IS NOT NULL;
CREATE INDEX idx_contractor_reports_status ON contractor_reports(organization_id, status);
-- Index partiel pour les rapports en attente de validation CdC
CREATE INDEX idx_contractor_reports_pending_review ON contractor_reports(building_id)
    WHERE status IN ('submitted', 'under_review');
-- Index sur magic token pour accès PWA
CREATE INDEX idx_contractor_reports_magic_token ON contractor_reports(magic_token_hash)
    WHERE magic_token_hash IS NOT NULL;

-- Trigger de mise à jour automatique de updated_at
CREATE OR REPLACE FUNCTION update_contractor_report_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_contractor_report_updated_at
    BEFORE UPDATE ON contractor_reports
    FOR EACH ROW
    EXECUTE FUNCTION update_contractor_report_timestamp();

-- Commentaires documentation
COMMENT ON TABLE contractor_reports IS 'BC16: Rapports de travaux soumis par corps de métier via PWA magic link';
COMMENT ON COLUMN contractor_reports.magic_token_hash IS 'Token UUID v4 pour accès PWA sans auth classique (72h)';
COMMENT ON COLUMN contractor_reports.parts_replaced IS 'JSON: [{name, reference, quantity, photo_document_id}]';
COMMENT ON COLUMN contractor_reports.photos_before IS 'UUIDs de documents (photos avant travaux)';
COMMENT ON COLUMN contractor_reports.photos_after IS 'UUIDs de documents (photos après travaux)';
COMMENT ON COLUMN contractor_reports.validated_by IS 'Membre CdC qui a validé le rapport';

-- Migration: BC17 - AGE Agile & Concertation
-- Art. 3.87 §2 Code Civil Belge : droit des copropriétaires représentant
-- au moins 1/5 des quotes-parts de demander une AGE au syndic.
-- Le syndic dispose de 15 jours pour convoquer l'assemblée ou répondre.

-- ENUM: statuts d'une demande d'AGE
CREATE TYPE age_request_status AS ENUM (
    'draft',       -- En cours de rédaction
    'open',        -- Ouverte pour signatures
    'reached',     -- Seuil 1/5 atteint
    'submitted',   -- Soumise au syndic (délai 15j)
    'accepted',    -- Syndic a accepté de convoquer
    'expired',     -- Délai syndic dépassé → auto-convocation
    'rejected',    -- Syndic a refusé
    'withdrawn'    -- Retirée par les demandeurs
);

-- Table principale des demandes d'AGE
CREATE TABLE age_requests (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id             UUID NOT NULL,
    building_id                 UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    title                       TEXT NOT NULL CHECK (char_length(title) > 0 AND char_length(title) <= 255),
    description                 TEXT,
    status                      age_request_status NOT NULL DEFAULT 'draft',
    created_by                  UUID NOT NULL,  -- owner_id de l'initiateur

    -- Progression vers le seuil légal 1/5
    total_shares_pct            NUMERIC(8, 6) NOT NULL DEFAULT 0.0
                                    CHECK (total_shares_pct >= 0.0 AND total_shares_pct <= 1.0),
    threshold_pct               NUMERIC(8, 6) NOT NULL DEFAULT 0.2
                                    CHECK (threshold_pct > 0.0 AND threshold_pct <= 1.0),
    threshold_reached           BOOLEAN NOT NULL DEFAULT FALSE,
    threshold_reached_at        TIMESTAMPTZ,

    -- Suivi délais syndic (Art. 3.87 §2 CC : 15 jours)
    submitted_to_syndic_at      TIMESTAMPTZ,
    syndic_deadline_at          TIMESTAMPTZ,
    syndic_response_at          TIMESTAMPTZ,
    syndic_notes                TEXT,
    auto_convocation_triggered  BOOLEAN NOT NULL DEFAULT FALSE,

    -- Liens vers d'autres entités
    meeting_id                  UUID,  -- AG convoquée suite à cette demande
    concertation_poll_id        UUID,  -- Sondage de concertation pré-AGE

    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Contraintes métier
    CONSTRAINT chk_submitted_deadline
        CHECK (syndic_deadline_at IS NULL OR submitted_to_syndic_at IS NOT NULL),
    CONSTRAINT chk_response_after_submission
        CHECK (syndic_response_at IS NULL OR submitted_to_syndic_at IS NOT NULL)
);

-- Table des cosignataires d'une demande d'AGE
CREATE TABLE age_request_cosignatories (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    age_request_id      UUID NOT NULL REFERENCES age_requests(id) ON DELETE CASCADE,
    owner_id            UUID NOT NULL,
    shares_pct          NUMERIC(8, 6) NOT NULL
                            CHECK (shares_pct > 0.0 AND shares_pct <= 1.0),
    signed_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Un copropriétaire ne peut signer qu'une fois par demande
    UNIQUE (age_request_id, owner_id)
);

-- Index pour les requêtes fréquentes
CREATE INDEX idx_age_requests_building ON age_requests (building_id);
CREATE INDEX idx_age_requests_organization ON age_requests (organization_id);
CREATE INDEX idx_age_requests_status ON age_requests (status);
CREATE INDEX idx_age_requests_created_by ON age_requests (created_by);
CREATE INDEX idx_age_request_cosignatories_request ON age_request_cosignatories (age_request_id);
CREATE INDEX idx_age_request_cosignatories_owner ON age_request_cosignatories (owner_id);

-- Index partiel pour le job de fond (demandes soumises avec délai dépassé)
CREATE INDEX idx_age_requests_expired_deadlines ON age_requests (syndic_deadline_at)
    WHERE status = 'submitted' AND syndic_deadline_at IS NOT NULL;

-- Trigger pour updated_at
CREATE OR REPLACE FUNCTION update_age_request_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER age_requests_updated_at
    BEFORE UPDATE ON age_requests
    FOR EACH ROW EXECUTE FUNCTION update_age_request_timestamp();

-- Commentaires de documentation légale
COMMENT ON TABLE age_requests IS 'Demandes d''AGE par les copropriétaires (Art. 3.87 §2 CC). Seuil légal: 1/5 des quotes-parts. Délai syndic: 15 jours.';
COMMENT ON COLUMN age_requests.threshold_pct IS 'Seuil légal (0.20 = 20% = 1/5). Art. 3.87 §2 CC.';
COMMENT ON COLUMN age_requests.syndic_deadline_at IS 'Délai légal pour le syndic: 15 jours après soumission. Art. 3.87 §2 CC.';
COMMENT ON COLUMN age_requests.auto_convocation_triggered IS 'TRUE si les copropriétaires ont convoqué eux-mêmes l''AGE faute de réponse syndic dans le délai légal.';

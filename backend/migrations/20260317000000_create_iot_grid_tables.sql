-- Migration: BOINC consent + Grid tasks tables
-- Created: 2026-03-17
-- Issues: IoT Phase 1 (MQTT Home Assistant) + BOINC Grid Computing

-- ─────────────────────────────────────────────────────────────────────────────
-- Table des consentements BOINC (GDPR Article 6.1.a + Article 7)
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE boinc_consents (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id          UUID        NOT NULL,
    organization_id   UUID        NOT NULL,
    granted           BOOLEAN     NOT NULL DEFAULT FALSE,
    granted_at        TIMESTAMPTZ,
    revoked_at        TIMESTAMPTZ,
    -- IPv4 ou IPv6 pour audit GDPR Article 30
    consent_ip        VARCHAR(45),
    -- Version de la clause acceptée (pour évolutions légales)
    consent_version   VARCHAR(20) NOT NULL DEFAULT 'v1.0',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Un seul enregistrement par propriétaire (upsert on conflict)
    UNIQUE (owner_id)
);

CREATE INDEX idx_boinc_consents_owner
    ON boinc_consents(owner_id);
CREATE INDEX idx_boinc_consents_granted
    ON boinc_consents(granted) WHERE granted = TRUE;
CREATE INDEX idx_boinc_consents_organization
    ON boinc_consents(organization_id);

COMMENT ON TABLE boinc_consents IS
    'GDPR Article 7 - Consentement explicite et révocable pour participation calcul distribué BOINC';
COMMENT ON COLUMN boinc_consents.consent_ip IS
    'IP capturée lors du consentement - obligatoire GDPR Article 30 (records of processing)';
COMMENT ON COLUMN boinc_consents.consent_version IS
    'Version de la clause de consentement acceptée - permet suivi évolutions légales';

-- ─────────────────────────────────────────────────────────────────────────────
-- Table des tâches de calcul distribué BOINC
-- IMPORTANT: toutes les données sont anonymisées — pas de PII dans kind_json
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE grid_tasks (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Isolation multi-tenant
    copropriete_id   UUID        NOT NULL,
    organization_id  UUID        NOT NULL,
    -- Type et paramètres de calcul (JSON anonymisé - GridTaskKind sérialisé)
    kind_json        JSONB       NOT NULL,
    -- Workflow: queued → running → completed/failed/cancelled
    status           VARCHAR(20) NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
    priority         SMALLINT    NOT NULL DEFAULT 5
        CHECK (priority BETWEEN 0 AND 10),
    deadline_at      TIMESTAMPTZ NOT NULL,
    -- Timestamps de progression
    started_at       TIMESTAMPTZ,
    completed_at     TIMESTAMPTZ,
    failed_at        TIMESTAMPTZ,
    failure_reason   TEXT,
    -- Résultats agrégés anonymisés (JSON)
    result_json      JSONB,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grid_tasks_copropriete
    ON grid_tasks(copropriete_id);
CREATE INDEX idx_grid_tasks_organization
    ON grid_tasks(organization_id);
CREATE INDEX idx_grid_tasks_status
    ON grid_tasks(status) WHERE status IN ('queued', 'running');
CREATE INDEX idx_grid_tasks_deadline
    ON grid_tasks(deadline_at) WHERE status = 'queued';

COMMENT ON TABLE grid_tasks IS
    'Tâches de calcul distribué BOINC - données anonymisées uniquement, sans PII';
COMMENT ON COLUMN grid_tasks.kind_json IS
    'GridTaskKind sérialisé - DOIT être anonymisé (agrégats kWh, pas de données personnelles)';
COMMENT ON COLUMN grid_tasks.result_json IS
    'Résultats de calcul BOINC agrégés - anonymisés, stockés pour affichage dans dashboard énergie';

-- ─────────────────────────────────────────────────────────────────────────────
-- Triggers updated_at
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION update_boinc_consent_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER boinc_consents_updated_at
    BEFORE UPDATE ON boinc_consents
    FOR EACH ROW EXECUTE FUNCTION update_boinc_consent_timestamp();

CREATE OR REPLACE FUNCTION update_grid_task_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER grid_tasks_updated_at
    BEFORE UPDATE ON grid_tasks
    FOR EACH ROW EXECUTE FUNCTION update_grid_task_timestamp();

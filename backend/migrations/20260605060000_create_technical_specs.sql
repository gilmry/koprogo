-- Story 3.8 (FR33) — TechnicalSpec versionnable + signatures multi-parties.
--
-- A `technical_spec` materialises a cahier des charges produced by the
-- syndic for an ACP (or a specific building within the ACP). It is
-- versionnable (SemVer-like major.minor.patch) and signed off by multiple
-- parties (syndic, AMO, lawyer, architect, ACP representative).
--
-- Signatures are append-only: a DB trigger blocks UPDATE/DELETE so the
-- legal/audit trail is ironclad even if a privileged direct SQL access
-- attempts to mutate a recorded signature.
--
-- Major bumps invalidate previous signatures (handled application-side via
-- `TechnicalSpec::requires_resignature`). Minor/patch bumps logically keep
-- the previous signatures relevant; the new spec still starts in `draft`.

-- 1) Specs table.
CREATE TABLE technical_specs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    acp_id              UUID NOT NULL REFERENCES acps(id) ON DELETE CASCADE,
    building_id         UUID REFERENCES buildings(id) ON DELETE CASCADE,
    title               VARCHAR(200) NOT NULL CHECK (length(title) >= 5),
    description         TEXT NOT NULL CHECK (length(description) >= 50 AND length(description) <= 10000),
    version_major       INTEGER NOT NULL CHECK (version_major >= 0),
    version_minor       INTEGER NOT NULL CHECK (version_minor >= 0),
    version_patch       INTEGER NOT NULL CHECK (version_patch >= 0),
    status              VARCHAR(20) NOT NULL CHECK (status IN ('draft','pending_signatures','approved','superseded')),
    deliverables        TEXT[] NOT NULL,
    required_signatures VARCHAR(32)[] NOT NULL,
    attachments         TEXT[] NOT NULL DEFAULT '{}',
    previous_version_id UUID REFERENCES technical_specs(id),
    created_by          UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_technical_specs_acp ON technical_specs(acp_id);
CREATE INDEX idx_technical_specs_status ON technical_specs(status);
CREATE INDEX idx_technical_specs_building ON technical_specs(building_id) WHERE building_id IS NOT NULL;

-- 2) Signatures table (append-only, trigger interdit UPDATE/DELETE).
CREATE TABLE technical_spec_signatures (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    technical_spec_id UUID NOT NULL REFERENCES technical_specs(id) ON DELETE CASCADE,
    signatory_user_id UUID NOT NULL REFERENCES users(id),
    role              VARCHAR(32) NOT NULL,
    mandate_id        UUID REFERENCES mandates(id),
    signed_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (technical_spec_id, signatory_user_id, role)
);

CREATE INDEX idx_tech_spec_sig_spec ON technical_spec_signatures(technical_spec_id);

-- 3) Audit immutability : explicit DB-level guard against UPDATE/DELETE on
--    technical_spec_signatures. Mirrors the Story 3.7 SyndicResponse trigger.
CREATE OR REPLACE FUNCTION reject_tech_spec_sig_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'TechnicalSpecSignature is append-only (Story 3.8 FR33)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tech_spec_sig_no_mutation
    BEFORE UPDATE OR DELETE ON technical_spec_signatures
    FOR EACH ROW
    EXECUTE FUNCTION reject_tech_spec_sig_mutation();

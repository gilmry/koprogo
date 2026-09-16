-- Issue #855 / ADR 0048 / ADR 0051 — lien signé à durée limitée pour l'accès
-- notaire à un état daté. Table dédiée plutôt que réutilisation de
-- `magic_links` : `magic_links.subject_user_id` est NOT NULL REFERENCES
-- users, et un notaire consulté par ce mécanisme n'a pas de compte KoproGo.
CREATE TABLE notary_links (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash      VARCHAR(64) NOT NULL UNIQUE,
    etat_date_id    UUID NOT NULL REFERENCES etats_dates(id) ON DELETE CASCADE,
    issued_by       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notary_links_token_hash ON notary_links (token_hash);
CREATE INDEX idx_notary_links_etat_date_id ON notary_links (etat_date_id);
CREATE INDEX idx_notary_links_expires_at ON notary_links (expires_at);

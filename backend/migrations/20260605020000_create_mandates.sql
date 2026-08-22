-- Story 3.4 (FR7 INV-14) — Mandates table.
--
-- A `mandate` materialises a juridical/technical delegation issued by a
-- syndic to an external professional (notaire, avocat, AMO, architecte,
-- BET, gardien). It carries mandatory temporal validity (`valid_until`)
-- and is bound to a scope (`scope_kind` + `scope_id`, building or whole ACP).

CREATE TABLE mandates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind            VARCHAR(32) NOT NULL CHECK (
        kind IN ('lawyer', 'notary', 'amo', 'architect', 'bet', 'warden')
    ),
    scope_kind      VARCHAR(16) NOT NULL CHECK (
        scope_kind IN ('building', 'acp')
    ),
    scope_id        UUID NOT NULL,
    issued_by       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason          VARCHAR(500) NOT NULL CHECK (length(reason) >= 10),
    valid_from      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until     TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT mandates_window_chk CHECK (valid_until > valid_from),
    CONSTRAINT mandates_no_self_chk CHECK (subject_user_id <> issued_by)
);

CREATE INDEX idx_mandates_subject_active
    ON mandates (subject_user_id)
    WHERE revoked_at IS NULL;

CREATE INDEX idx_mandates_scope
    ON mandates (scope_kind, scope_id);

CREATE INDEX idx_mandates_valid_until
    ON mandates (valid_until);

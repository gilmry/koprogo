-- Story 3.2 (FR6 INV-13 INV-17) — MagicLink table.
--
-- Public-access tokens issued by syndic/superadmin so external recipients
-- (contractors, third parties) can consult a single ticket / quote / invoice /
-- contractor-evaluation without creating an account.
--
-- Security:
-- - `token_hash` holds the SHA-256 hex digest (64 chars) of the clear token.
--   The clear token is returned to the issuer ONCE in the HTTP response and
--   is NEVER persisted.
-- - `consumed_at` enforces single-use: the validate-and-consume endpoint sets
--   it atomically; a second attempt returns 403 magic_link_consumed.
-- - `expires_at` enforces a TTL bounded at the use-case layer (1 minute -> 30 days).
-- - `subject_user_id` / `issued_by` are both FK to users with ON DELETE CASCADE
--   so an orphan link cannot survive its issuer or recipient.

CREATE TABLE magic_links (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash      VARCHAR(64) NOT NULL UNIQUE,
    subject_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope_kind      VARCHAR(32) NOT NULL CHECK (
        scope_kind IN ('ticket', 'quote', 'invoice', 'contractor_evaluation')
    ),
    scope_id        UUID NOT NULL,
    issued_by       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at      TIMESTAMPTZ NOT NULL,
    consumed_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_magic_links_token_hash ON magic_links (token_hash);
CREATE INDEX idx_magic_links_expires_at ON magic_links (expires_at);
CREATE INDEX idx_magic_links_subject_user_id ON magic_links (subject_user_id);
CREATE INDEX idx_magic_links_issued_by ON magic_links (issued_by);

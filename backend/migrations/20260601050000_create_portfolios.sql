-- Story 2.1 — Portfolio backend (Slice 2 Refonte UX multi-rôle ACP).
-- Sources :
--   - docs/maury/refonte-ux-multi-role-acp/architecture.md §2.5 (BC Portfolio)
--   - docs/maury/refonte-ux-multi-role-acp/architecture.md ADR-0011
--   - docs/maury/refonte-ux-multi-role-acp/stories.md §4 Story 2.1
--
-- Crée trois tables :
--   * portfolios               : favoris/portefeuilles d'un utilisateur (1..N par user)
--   * portfolio_buildings      : liaison M:N portfolio ↔ building (avec flag favorite)
--   * portfolio_shares         : partage d'un portfolio avec d'autres users (can_edit)
--
-- INV / FR :
--   - FR36 (portefeuille immeubles équipe cabinet)
--   - Mémoire `koprogo-modular-toolbox` (favoris star)
--
-- Rollback : `20260601050000_create_portfolios.down.sql` (convention sqlx
-- native `.down.sql` — cf. fix commit de97d17).

CREATE TABLE portfolios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(120) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT portfolios_name_not_blank CHECK (length(btrim(name)) >= 2)
);

CREATE TABLE portfolio_buildings (
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (portfolio_id, building_id)
);

CREATE TABLE portfolio_shares (
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    shared_with_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    can_edit BOOLEAN NOT NULL DEFAULT FALSE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (portfolio_id, shared_with_user_id)
);

CREATE INDEX idx_portfolios_owner ON portfolios(owner_user_id);
CREATE INDEX idx_portfolio_buildings_building ON portfolio_buildings(building_id);
CREATE INDEX idx_portfolio_shares_user ON portfolio_shares(shared_with_user_id);

COMMENT ON TABLE portfolios IS 'Portefeuilles immeubles (favoris/équipe) — Story 2.1, ADR-0011.';
COMMENT ON COLUMN portfolio_buildings.is_favorite IS 'Star/épinglage (favoris d''abord dans le listing — AC @happy Story 2.1).';
COMMENT ON TABLE portfolio_shares IS 'Partage portefeuille équipe cabinet — Story 2.1, ADR-0011.';

-- Rollback de la migration 20260601050000_create_portfolios.sql (Story 2.1).
--
-- Convention sqlx native `.down.sql` (cf. fix commit de97d17 — story 1.1
-- a déjà été renommée du `_DOWN.sql` propriétaire).
--
-- Ordre de drop important : tables enfants (portfolio_buildings,
-- portfolio_shares) AVANT le parent (portfolios) pour respecter les FKs ;
-- en pratique `ON DELETE CASCADE` rend l'ordre indifférent, mais on
-- explicite pour rester lisible.

DROP INDEX IF EXISTS idx_portfolio_shares_user;
DROP INDEX IF EXISTS idx_portfolio_buildings_building;
DROP INDEX IF EXISTS idx_portfolios_owner;

DROP TABLE IF EXISTS portfolio_shares;
DROP TABLE IF EXISTS portfolio_buildings;
DROP TABLE IF EXISTS portfolios;

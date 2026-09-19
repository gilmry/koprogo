-- La table `fund_reassignments` que le dépôt de fonds écrivait déjà.
--
-- Troisième occurrence du même défaut que #581 et #48, trouvée par une
-- battue systématique après les deux premières : l'entité
-- `FundReassignment`, son port, son INSERT et son SELECT ont été livrés —
-- sans migration. Ici ce n'est pas une colonne qui manquait, c'est la
-- TABLE ENTIÈRE. Le dépôt emploie des requêtes vérifiées à l'exécution :
-- toute réaffectation de fonds échouait.
--
-- Colonnes et types repris de l'entité (`domain/comptabilite/fund.rs:160`)
-- et de l'INSERT (`fund_repository_impl.rs:155`) — rien n'est inventé ici,
-- la forme était déjà décidée par le code qui s'en sert.

CREATE TABLE IF NOT EXISTS fund_reassignments (
    id UUID PRIMARY KEY,
    fund_id UUID NOT NULL REFERENCES funds(id) ON DELETE CASCADE,
    previous_purpose VARCHAR(255) NOT NULL,
    new_purpose VARCHAR(255) NOT NULL,
    -- ADR-0007/0008 : la monnaie est un NUMERIC exact, jamais un flottant.
    amount NUMERIC(14, 2) NOT NULL,
    -- Art. 3.88 — une réaffectation n'existe que si une AG l'a décidée.
    -- La clé étrangère est donc ce qui rend la règle juridique
    -- structurellement vraie, et pas seulement vérifiée dans le use-case.
    resolution_id UUID NOT NULL REFERENCES resolutions(id) ON DELETE RESTRICT,
    reassigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Lecture principale : « l'historique des réaffectations de ce fonds »,
-- rendu dans l'ordre chronologique par `list_reassignments`.
CREATE INDEX IF NOT EXISTS fund_reassignments_par_fonds
    ON fund_reassignments (fund_id, reassigned_at);

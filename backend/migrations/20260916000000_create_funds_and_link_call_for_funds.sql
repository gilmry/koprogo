-- Les fonds de l'ACP, et le lien qui manquait aux appels de fonds.
--
-- ── Pourquoi cette migration existe, et ce qu'elle répare ──────────────────
--
-- La fusion des vingt-huit branches d'agent du 2026-09-15 a introduit le
-- domaine `Fund` — entité, dépôt, cas d'usage — et un champ `fund_id` sur
-- `CallForFunds`. Le code lisait et écrivait donc deux choses que la base
-- ne connaissait pas :
--
--     column "fund_id" of relation "call_for_funds" does not exist
--
-- Dix scénarios BDD de `bdd_financial` échouaient là-dessus, et le gate
-- `BDD Tests` avec eux. Aucune revue de branche ne pouvait le voir : la
-- branche qui a ajouté le domaine ne touchait pas aux migrations, et celle
-- qui aurait dû les écrire n'existait pas. C'est un défaut de JONCTION,
-- comme le DTO sans `ToSchema` du même jour.
--
-- ── Ce que le domaine impose, et que le schéma doit tenir ─────────────────
--
-- `FundKind` a trois variantes (`fund.rs:34`) et deux d'entre elles ont des
-- règles que la loi porte :
--
--   working_capital  fonds de roulement, dépenses périodiques
--   reserve          fonds de réserve légal — Art. 3.86 § 3, ADR-0012,
--                    exigible à cinq ans, plancher de 5 %
--   earmarked        fonds affecté — épargne vers un chantier NOMMÉ
--                    (`purpose`), voté aux 2/3, Art. 3.88
--
-- `purpose` et `target_amount` ne valent que pour `earmarked` : la
-- contrainte le dit en SQL plutôt que de laisser le domaine seul garant.
-- Un fonds affecté sans objet n'est pas un fonds affecté, c'est un compte.

CREATE TABLE IF NOT EXISTS funds (
    id            UUID PRIMARY KEY,
    acp_id        UUID NOT NULL REFERENCES acps(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL CHECK (kind IN ('working_capital', 'reserve', 'earmarked')),
    name          TEXT NOT NULL,
    purpose       TEXT,
    target_amount NUMERIC(14, 2),
    balance       NUMERIC(14, 2) NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Un fonds affecté DOIT nommer son objet (Art. 3.88) ; les deux autres
    -- ne doivent pas en porter — sinon « affecté » perdrait son sens.
    CONSTRAINT fonds_affecte_nomme_son_objet CHECK (
        (kind = 'earmarked' AND purpose IS NOT NULL)
        OR (kind <> 'earmarked' AND purpose IS NULL)
    )
);

-- Une ACP n'a qu'UN fonds de roulement et qu'UN fonds de réserve. Elle peut
-- avoir autant de fonds affectés que de chantiers votés : l'index partiel
-- dit exactement ça, là où un UNIQUE simple aurait interdit le second
-- chantier.
CREATE UNIQUE INDEX IF NOT EXISTS idx_funds_un_seul_par_type_structurel
    ON funds (acp_id, kind)
    WHERE kind IN ('working_capital', 'reserve');

CREATE INDEX IF NOT EXISTS idx_funds_acp ON funds (acp_id);

-- Le lien qui manquait. NULLABLE à dessein : les appels de fonds antérieurs
-- au domaine `Fund` n'en désignent aucun, et les réécrire rétroactivement
-- inventerait une imputation que personne n'a votée.
ALTER TABLE call_for_funds
    ADD COLUMN IF NOT EXISTS fund_id UUID REFERENCES funds(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_call_for_funds_fund ON call_for_funds (fund_id);

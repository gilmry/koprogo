-- #845 / ADR 0048 / ADR 0051 — lien notaire (accès signé à un état daté).
--
-- `GET /etats-dates/reference/{reference_number}` ne vérifiait aucune
-- identité : un état daté porte les dettes d'un copropriétaire nommé, et sa
-- référence n'est pas un secret (elle circule dans des courriels, des
-- dossiers de vente). ADR 0051 tranche la modalité : un jeton signé, émis
-- par le syndic, qui vaut sept jours calendaires, autorise plusieurs
-- lectures pendant cette fenêtre, et que le syndic peut renouveler ou
-- révoquer avant terme.
--
-- Différences volontaires avec `magic_links` (Story 3.2) :
-- - PAS de `consumed_at` : ce lien est multi-lecture par construction.
-- - `revoked_at` / `revoked_by` : révocation explicite, absente de
--   `magic_links`.
-- - `renewed_at` : trace du dernier renouvellement (le jeton ne change pas).
-- - `scope_id` est ici `etat_date_id`, à cardinalité 1 (pas de `scope_kind`
--   polymorphe : ce lien ne sert qu'un seul type de ressource).

CREATE TABLE liens_notaire (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash      VARCHAR(64) NOT NULL UNIQUE,
    etat_date_id    UUID NOT NULL REFERENCES etats_dates(id) ON DELETE CASCADE,
    emis_par        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cree_le         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expire_le       TIMESTAMPTZ NOT NULL,
    revoque_le      TIMESTAMPTZ,
    revoque_par     UUID REFERENCES users(id) ON DELETE SET NULL,
    renouvele_le    TIMESTAMPTZ,
    mis_a_jour_le   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT liens_notaire_revocation_coherente_chk CHECK (
        (revoque_le IS NULL AND revoque_par IS NULL)
        OR (revoque_le IS NOT NULL AND revoque_par IS NOT NULL)
    )
);

CREATE INDEX idx_liens_notaire_token_hash ON liens_notaire (token_hash);
CREATE INDEX idx_liens_notaire_etat_date_id ON liens_notaire (etat_date_id);

-- Accélère `find_active_by_etat_date_id` (renouvellement / révocation) : un
-- seul lien non révoqué est manipulé à la fois par ces deux opérations.
CREATE INDEX idx_liens_notaire_actifs_par_etat_date
    ON liens_notaire (etat_date_id, cree_le DESC)
    WHERE revoque_le IS NULL;

-- Story 1.1 — Refonte UX multi-rôle ACP
-- Source : docs/maury/refonte-ux-multi-role-acp/architecture.md §5.2, ADR-0010
--
-- Crée la table `acps` (racine d'agrégat « Association des Copropriétaires »,
-- Art. 3.84-3.89 Code Civil belge). L'ACP est juridiquement distincte du
-- cabinet syndic qui la gère ; `organization_id` est NULLABLE pour supporter
-- les ACPs auto-gérées (sans syndic professionnel).
--
-- Rollback manuel — cf. fichier `20260601010000_create_acps_DOWN.sql`
-- (la version sqlx-cli installée ici n'utilise PAS la convention pair
-- `.up.sql`/`.down.sql`, donc on conserve un script de rollback nommé
-- explicitement, applicable à la main : `psql ... -f *_DOWN.sql`).

CREATE TABLE IF NOT EXISTS acps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NULL REFERENCES organizations(id) ON DELETE SET NULL,
    name VARCHAR(160) NOT NULL,
    slug VARCHAR(80) NOT NULL UNIQUE,
    legal_status VARCHAR(32) NOT NULL DEFAULT 'copropriete_belge',
    bce_number VARCHAR(20) NULL,
    address_street VARCHAR(200) NOT NULL,
    address_postal_code VARCHAR(10) NOT NULL,
    address_city VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT acps_name_min_length CHECK (char_length(trim(name)) >= 2),
    CONSTRAINT acps_legal_status_known CHECK (legal_status IN ('copropriete_belge'))
);

CREATE INDEX IF NOT EXISTS idx_acps_organization_id ON acps(organization_id);
CREATE INDEX IF NOT EXISTS idx_acps_slug ON acps(slug);

COMMENT ON TABLE  acps                  IS 'Association des Coproprietaires (Art. 3.84 CC). ADR-0010, Story 1.1.';
COMMENT ON COLUMN acps.organization_id  IS 'Cabinet syndic gestionnaire. NULL = ACP auto-geree.';
COMMENT ON COLUMN acps.slug             IS 'Slug kebab-case unique, derive du name.';
COMMENT ON COLUMN acps.legal_status     IS 'Statut juridique (v0.1.0 = copropriete_belge uniquement).';
COMMENT ON COLUMN acps.bce_number       IS 'Numero BCE belge si ACP immatriculee (optionnel).';

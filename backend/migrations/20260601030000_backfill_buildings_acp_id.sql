-- Story 1.2 — Backfill data (Étape 2/3)
-- Source : docs/maury/refonte-ux-multi-role-acp/architecture.md §5.2
--
-- Pour chaque organization_id DISTINCT présent dans `buildings` :
--   1. Crée une ACP miroir dans `acps` (name = organization.name,
--      slug = kebab-case(name) + suffixe court UUID pour éviter collision,
--      address = address de référence — pris du PREMIER building de l'org
--      car organizations n'a pas de champs address ; ACP miroir = entité
--      juridique factice, l'admin pourra l'éditer ensuite),
--      organization_id = org.id.
--   2. UPDATE buildings.acp_id = nouvelle_acp_id WHERE organization_id = org.id.
--   3. INSERT audit_logs.event_type='BuildingAcpBackfilled' pour chaque building.
--
-- Guard @security : ce script suppose que le runner externe a vérifié
-- `BACKUP_CONFIRMED=true` avant l'invocation (cf. test integration).
--
-- Idempotence : ON CONFLICT DO NOTHING sur le slug ACP. Si une ACP miroir
-- existe déjà (re-run partiel), on la réutilise via la sous-requête de
-- l'UPDATE (FROM acps WHERE organization_id = X).
--
-- Rollback : `20260601_030000_backfill_buildings_acp_id.down.sql`.
--
-- Pré-check final : SELECT COUNT(*) FROM buildings WHERE acp_id IS NULL → 0.

-- Étape 2.1 : créer une ACP miroir par organization référencée dans buildings.
WITH orgs_with_buildings AS (
    SELECT DISTINCT
        o.id              AS org_id,
        o.name            AS org_name,
        o.slug            AS org_slug,
        (
            SELECT b2.address
            FROM buildings b2
            WHERE b2.organization_id = o.id
            ORDER BY b2.created_at
            LIMIT 1
        )                 AS ref_address,
        (
            SELECT b2.city
            FROM buildings b2
            WHERE b2.organization_id = o.id
            ORDER BY b2.created_at
            LIMIT 1
        )                 AS ref_city,
        (
            SELECT b2.postal_code
            FROM buildings b2
            WHERE b2.organization_id = o.id
            ORDER BY b2.created_at
            LIMIT 1
        )                 AS ref_postal
    FROM organizations o
    WHERE EXISTS (SELECT 1 FROM buildings b WHERE b.organization_id = o.id)
)
INSERT INTO acps (
    id, organization_id, name, slug, legal_status,
    address_street, address_postal_code, address_city,
    created_at, updated_at
)
SELECT
    gen_random_uuid(),
    org_id,
    org_name,
    -- Slug : kebab-case(name) + suffixe -mirror-<8 hex de l'org id> pour
    -- éviter toute collision avec un slug existant (ACPs déjà créées via
    -- l'API Story 1.1, ou organizations homonymes après re-run).
    substring(
        regexp_replace(lower(trim(org_name)), '[^a-z0-9]+', '-', 'g')
        FROM 1 FOR 60
    ) || '-mirror-' || substring(replace(org_id::text, '-', '') FROM 1 FOR 8),
    'copropriete_belge',
    COALESCE(NULLIF(trim(ref_address), ''), 'Adresse à compléter'),
    COALESCE(NULLIF(trim(ref_postal),  ''), '0000'),
    COALESCE(NULLIF(trim(ref_city),    ''), 'À compléter'),
    now(),
    now()
FROM orgs_with_buildings
ON CONFLICT (slug) DO NOTHING;

-- Étape 2.2 : backfill acp_id pour chaque building, en pointant sur l'ACP
-- miroir correspondant à son organization_id. Si plusieurs ACPs existent
-- pour la même org (ex: ACPs déjà créées par l'API Story 1.1), on prend
-- la première (ORDER BY created_at) — idempotent et déterministe.
UPDATE buildings b
SET acp_id = (
    SELECT a.id
    FROM acps a
    WHERE a.organization_id = b.organization_id
    ORDER BY a.created_at
    LIMIT 1
)
WHERE b.organization_id IS NOT NULL
  AND b.acp_id IS NULL;

-- Étape 2.3 : audit log par building backfillé.
INSERT INTO audit_logs (
    id, timestamp, event_type, user_id, organization_id,
    resource_type, resource_id, success, created_at, metadata
)
SELECT
    gen_random_uuid(),
    now(),
    'BuildingAcpBackfilled',
    NULL,
    b.organization_id,
    'Building',
    b.id,
    true,
    now(),
    jsonb_build_object(
        'acp_id',           b.acp_id,
        'organization_id',  b.organization_id,
        'story',            '1.2',
        'migration',        '20260601_030000_backfill_buildings_acp_id'
    )
FROM buildings b
WHERE b.acp_id IS NOT NULL
  -- Idempotence : on n'audit pas deux fois le même building.
  AND NOT EXISTS (
      SELECT 1 FROM audit_logs al
      WHERE al.event_type = 'BuildingAcpBackfilled'
        AND al.resource_id = b.id
  );

-- Pré-check final (assertion via constraint) : aucun building avec
-- organization_id IS NOT NULL ne doit rester sans acp_id.
DO $$
DECLARE
    orphan_count INT;
BEGIN
    SELECT COUNT(*) INTO orphan_count
    FROM buildings
    WHERE organization_id IS NOT NULL
      AND acp_id IS NULL;
    IF orphan_count > 0 THEN
        RAISE EXCEPTION 'Backfill incomplet : % buildings orphelins (organization_id NOT NULL & acp_id NULL)', orphan_count;
    END IF;
END $$;

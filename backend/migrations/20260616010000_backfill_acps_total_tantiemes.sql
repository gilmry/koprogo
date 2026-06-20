-- Track H Conformité légale — Story H4 (WP-CL1) — ADR-0010
-- Backfill `acps.total_tantiemes` depuis les buildings rattachés.
--   - Mono-building : acps.total_tantiemes = building.total_tantiemes.
--   - Multi-building : acps.total_tantiemes = SUM(buildings.total_tantiemes)
--     (les sous-totaux de blocs composent le dénominateur global de l'acte).
--   - Aucun building : reste au défaut (1000).
-- WARNING si une ACP multi-blocs a des dénominateurs hétérogènes entre blocs
-- (acte de base unique attendu — validation admin manuelle requise, cf.
-- mémoire admin-publishes-conform-buildings). On ne bloque PAS la migration.

UPDATE acps a
SET total_tantiemes = sub.sum_tantiemes
FROM (
    SELECT b.acp_id,
           SUM(b.total_tantiemes)::INT AS sum_tantiemes
    FROM buildings b
    WHERE b.acp_id IS NOT NULL
    GROUP BY b.acp_id
) sub
WHERE a.id = sub.acp_id
  AND sub.sum_tantiemes > 0;

DO $$
DECLARE r RECORD;
BEGIN
    FOR r IN
        SELECT b.acp_id,
               COUNT(*)                          AS n_blocs,
               COUNT(DISTINCT b.total_tantiemes)  AS distinct_bases
        FROM buildings b
        WHERE b.acp_id IS NOT NULL
        GROUP BY b.acp_id
        HAVING COUNT(*) > 1 AND COUNT(DISTINCT b.total_tantiemes) > 1
    LOOP
        RAISE WARNING
            'ACP % : % blocs aux denominateurs heterogenes — total_tantiemes ACP = SUM, a valider manuellement (acte de base unique attendu, Art. 3.84 CC).',
            r.acp_id, r.n_blocs;
    END LOOP;
END $$;

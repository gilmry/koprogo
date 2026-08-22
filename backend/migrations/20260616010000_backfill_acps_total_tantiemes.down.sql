-- Rollback backfill Story H4 — réinitialise au défaut. La colonne est de
-- toute façon supprimée par le `.down` de 20260616000000 (exécuté ensuite).
UPDATE acps SET total_tantiemes = 1000;

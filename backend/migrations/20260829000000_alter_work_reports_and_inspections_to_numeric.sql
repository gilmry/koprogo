-- Migration: ADR-0008 — work_reports.cost et technical_inspections.cost
-- DOUBLE PRECISION -> NUMERIC(12,2)
--
-- Suite de #661, même lot de dette que `payment_reminders` (migration
-- 20260826000000) : des `f64` monétaires révélés par le gate
-- `scripts/check-no-f64-money.sh` puis GELÉS dans son allowlist sous la
-- rubrique « DETTE CONNUE ET TRACÉE (à résorber, pas un carve-out accordé) ».
-- Une entrée de dette se supprime, elle ne se transforme pas en carve-out.
--
-- Ancrage métier : ces deux colonnes portent des coûts de travaux et
-- d'inspections en euros, facturés à la copropriété. Ils alimentent le carnet
-- d'entretien numérique, sont refacturés aux copropriétaires via la
-- répartition des charges (Art. 3.86 CC), et un coût de travaux entre dans le
-- calcul du fonds de réserve. Ce ne sont pas des valeurs d'affichage.
--
-- Précision NUMERIC(12,2) : alignée sur `expenses.amount` (migration
-- 20260502000000). Une rénovation de façade se chiffre en centaines de
-- milliers d'euros ; 10 chiffres avant la virgule laissent la marge voulue.
--
-- ── Ordre des opérations, et pourquoi ────────────────────────────────────
--
-- `ALTER COLUMN TYPE` NE RÉÉCRIT PAS l'expression d'un CHECK : PostgreSQL y
-- insère un cast vers l'ancien type. Vérifié sur une base réelle avant
-- d'écrire cette migration :
--
--     CHECK (cost >= 0)
--   devient
--     CHECK ((cost)::double precision >= (0)::double precision)
--
-- La positivité resterait donc évaluée en binary64 APRÈS une migration dont
-- c'est précisément l'objet. Les CHECK sont donc droppés puis recréés
-- explicitement avec des littéraux NUMERIC.
--
-- Les noms visés ci-dessous ne sont pas devinés : ils ont été relevés dans
-- `pg_constraint` sur une base reconstruite depuis le DDL d'origine
-- (20251203000000 / 20251203000001). Ce sont des contraintes de COLONNE,
-- donc nommées `<table>_<colonne>_check` de façon déterministe — contrairement
-- aux CHECK de TABLE anonymes de `payment_reminders`, que PostgreSQL avait
-- nommés `payment_reminders_check` / `_check2` et dont les noms inventés
-- avaient laissé survivre une contrainte en 20260826000000.
--
-- ADR : 0007 (Decimal vs f64), 0008 (NUMERIC vs DOUBLE PRECISION).

-- ── work_reports.cost (NOT NULL) ─────────────────────────────────────────

ALTER TABLE work_reports DROP CONSTRAINT IF EXISTS work_reports_cost_check;

ALTER TABLE work_reports
    ALTER COLUMN cost TYPE NUMERIC(12,2)
    USING cost::NUMERIC(12,2);

ALTER TABLE work_reports
    ADD CONSTRAINT work_reports_cost_check CHECK (cost >= 0);

COMMENT ON COLUMN work_reports.cost IS
    'Coût total des travaux en EUR (NUMERIC(12,2) — montant refacturé via la répartition des charges, précision exacte requise — ADR-0007/0008)';

-- ── technical_inspections.cost (nullable) ────────────────────────────────

ALTER TABLE technical_inspections DROP CONSTRAINT IF EXISTS technical_inspections_cost_check;

ALTER TABLE technical_inspections
    ALTER COLUMN cost TYPE NUMERIC(12,2)
    USING cost::NUMERIC(12,2);

ALTER TABLE technical_inspections
    ADD CONSTRAINT technical_inspections_cost_check CHECK (cost IS NULL OR cost >= 0);

COMMENT ON COLUMN technical_inspections.cost IS
    'Coût de l''inspection en EUR (NUMERIC(12,2) — montant refacturé via la répartition des charges, précision exacte requise — ADR-0007/0008)';

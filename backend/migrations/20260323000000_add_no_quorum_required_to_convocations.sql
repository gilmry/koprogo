-- Migration: Add no_quorum_required flag to convocations
-- Legal anchor: Art. 3.87 §5 CC — 2e convocation après quorum non atteint
-- "La deuxième assemblée délibère valablement quel que soit le nombre de présents."

ALTER TABLE convocations
    ADD COLUMN IF NOT EXISTS no_quorum_required BOOLEAN DEFAULT FALSE NOT NULL;

-- Index pour retrouver les convocations sans quorum requis (2e convocations)
CREATE INDEX IF NOT EXISTS idx_convocations_no_quorum_required
    ON convocations (no_quorum_required)
    WHERE no_quorum_required = TRUE;

-- Comment documenting the legal basis
COMMENT ON COLUMN convocations.no_quorum_required IS
    'Art. 3.87 §5 CC: Pour les 2e convocations, délibérations valables quel que soit le nombre de présents. '
    'TRUE only for second convocations where quorum is NOT required. '
    'FALSE for first convocations which require >50% quorum.';

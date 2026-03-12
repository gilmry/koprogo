-- Migration: Add quorum validation fields to meetings
-- Legal anchor: Art. 3.87 §5 CC — AG valide si >50% des quotes-parts présentes/représentées
-- Lacune CRITIQUE identifiée dans audit_conformite.rst — corrigée ici

ALTER TABLE meetings
    ADD COLUMN IF NOT EXISTS quorum_validated BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS quorum_percentage DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS total_quotas DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS present_quotas DOUBLE PRECISION;

-- Contrainte : le pourcentage de quorum doit être cohérent (0-100)
ALTER TABLE meetings
    ADD CONSTRAINT chk_quorum_percentage_range
    CHECK (quorum_percentage IS NULL OR (quorum_percentage >= 0.0 AND quorum_percentage <= 100.0));

-- Contrainte : les millièmes présents ne peuvent pas dépasser le total
ALTER TABLE meetings
    ADD CONSTRAINT chk_present_quotas_le_total
    CHECK (present_quotas IS NULL OR total_quotas IS NULL OR present_quotas <= total_quotas);

-- Contrainte : si quorum_validated = TRUE, le pourcentage doit être > 50%
ALTER TABLE meetings
    ADD CONSTRAINT chk_quorum_validated_consistency
    CHECK (
        NOT quorum_validated OR (quorum_percentage IS NOT NULL AND quorum_percentage > 50.0)
    );

-- Index pour retrouver les AG sans quorum validé (utile pour le reporting)
CREATE INDEX IF NOT EXISTS idx_meetings_quorum_not_validated
    ON meetings (building_id, scheduled_date)
    WHERE quorum_validated = FALSE AND status = 'Scheduled';

-- Commentaires de documentation
COMMENT ON COLUMN meetings.quorum_validated IS
    'Art. 3.87 §5 CC: TRUE si >50% des quotes-parts sont présentes/représentées';
COMMENT ON COLUMN meetings.quorum_percentage IS
    'Pourcentage des quotes-parts présentes/représentées (0.0-100.0)';
COMMENT ON COLUMN meetings.total_quotas IS
    'Total des millièmes du bâtiment (généralement 1000)';
COMMENT ON COLUMN meetings.present_quotas IS
    'Millièmes présents + représentés par procuration';

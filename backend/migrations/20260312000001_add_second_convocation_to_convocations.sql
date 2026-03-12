-- Migration: Add first_meeting_id to convocations for 2e convocation workflow
-- Legal anchor: Art. 3.87 §5 CC — 2e convocation après quorum non atteint
-- La 2e AG délibère valablement quel que soit le nombre de présents (aucun quorum)
-- La 2e AG doit avoir lieu au moins 15 jours après la 1ère (Art. 3.87 §3)

ALTER TABLE convocations
    ADD COLUMN IF NOT EXISTS first_meeting_id UUID REFERENCES meetings(id) ON DELETE SET NULL;

-- Index pour retrouver les 2e convocations associées à une 1ère AG
CREATE INDEX IF NOT EXISTS idx_convocations_first_meeting_id
    ON convocations (first_meeting_id)
    WHERE first_meeting_id IS NOT NULL;

-- Contrainte : une 2e convocation DOIT avoir un first_meeting_id
-- (appliquée au niveau applicatif + commentaire documentaire ici)
COMMENT ON COLUMN convocations.first_meeting_id IS
    'Art. 3.87 §5 CC: ID de la 1ère AG dont le quorum n''a pas été atteint. '
    'Renseigné uniquement pour les convocations de type second_convocation. '
    'La 2e AG se tient sans quorum minimum.';

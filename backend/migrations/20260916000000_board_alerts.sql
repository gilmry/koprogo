-- Story 4.7 — CdC : action create_alert(text, severity, target=AG_next).
--
-- Art. 3.90 §1er CC confie au conseil de copropriété la mission de veiller à
-- la bonne exécution par le syndic de ses missions, sans lui donner de canal
-- pour agir sur ce constat. `board_alerts` est cet instrument minimal : un
-- membre en mandat signale un fait, rattaché à l'AG suivante qui en décide.
--
-- `raised_by_board_member_id` référence `board_members.id` (pas `owners.id`)
-- : c'est le mandat précis qui a émis l'alerte, pas seulement la personne
-- (une même personne peut avoir plusieurs mandats successifs).
--
-- `target_meeting_id` référence `meetings.id` — la traduction opérationnelle
-- de `target=AG_next` : la couche application fige la résolution de « la
-- prochaine AG » au moment de la création (cf. `CdcUseCases::create_alert`).

CREATE TABLE IF NOT EXISTS board_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    raised_by_board_member_id UUID NOT NULL REFERENCES board_members(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    severity VARCHAR(16) NOT NULL,
    target_meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_board_alerts_text_not_empty CHECK (LENGTH(TRIM(text)) > 0),
    CONSTRAINT chk_board_alerts_severity_valeurs
        CHECK (severity IN ('info', 'warning', 'critical'))
);

CREATE INDEX IF NOT EXISTS idx_board_alerts_target_meeting_id
    ON board_alerts(target_meeting_id);
CREATE INDEX IF NOT EXISTS idx_board_alerts_building_id
    ON board_alerts(building_id);

COMMENT ON TABLE board_alerts IS
    'Alertes émises par le conseil de copropriété à destination de la prochaine AG (Art. 3.90 §1er CC, Story 4.7).';
COMMENT ON COLUMN board_alerts.raised_by_board_member_id IS
    'Le mandat (board_members.id) qui a émis l''alerte — pas seulement la personne.';
COMMENT ON COLUMN board_alerts.target_meeting_id IS
    'AG à laquelle l''alerte est visible — résolution de target=AG_next figée à la création.';

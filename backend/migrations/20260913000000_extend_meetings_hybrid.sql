-- Story 4.1 — Meeting.mode (in_person/remote/hybrid) + quorum agrégé Decimal.
--
-- Art. 3.87 §1er CC : « Chaque copropriétaire peut participer à l'assemblée
-- générale physiquement ou à distance au moyen d'une communication
-- électronique ». `mode` porte l'annonce de la modalité ; `videoconf_url`
-- porte le lien effectif de connexion — sans lui, l'annonce d'un mode
-- distanciel/hybride est creuse (cf. `Meeting::set_mode()`).
--
-- Le CHECK ci-dessous est une garde en profondeur (defense-in-depth) : la
-- même règle est déjà appliquée côté domain, AVANT toute tentative
-- d'écriture (422 typé). Elle protège contre une écriture qui contournerait
-- le domain (migration de données, script one-off).

ALTER TABLE meetings
    ADD COLUMN IF NOT EXISTS mode VARCHAR(16) NOT NULL DEFAULT 'in_person',
    ADD COLUMN IF NOT EXISTS videoconf_url TEXT;

ALTER TABLE meetings
    ADD CONSTRAINT chk_meetings_mode_valeurs
    CHECK (mode IN ('in_person', 'remote', 'hybrid'));

ALTER TABLE meetings
    ADD CONSTRAINT chk_meetings_mode_videoconf_url
    CHECK (
        mode = 'in_person'
        OR (videoconf_url IS NOT NULL AND btrim(videoconf_url) <> '')
    );

COMMENT ON COLUMN meetings.mode IS
    'Art. 3.87 §1er CC — modalité de tenue : in_person, remote ou hybrid.';
COMMENT ON COLUMN meetings.videoconf_url IS
    'URL de connexion à la session distancielle. Obligatoire si mode ∈ {remote, hybrid}.';

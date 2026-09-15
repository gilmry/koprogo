ALTER TABLE meetings DROP CONSTRAINT IF EXISTS chk_meetings_mode_videoconf_url;
ALTER TABLE meetings DROP CONSTRAINT IF EXISTS chk_meetings_mode_valeurs;
ALTER TABLE meetings DROP COLUMN IF EXISTS videoconf_url;
ALTER TABLE meetings DROP COLUMN IF EXISTS mode;

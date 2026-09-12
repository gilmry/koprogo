-- Une résolution sait de quelle mission elle traite, donc qui ne peut y voter.
--
-- Art. 3.87 § 9 : « Aucune personne mandatée ou employée par l'association des
-- copropriétaires, ou prestant pour elle des services dans le cadre de tout
-- autre contrat, ne peut participer personnellement ou par procuration aux
-- délibérations et aux votes RELATIFS À LA MISSION QUI LUI A ÉTÉ CONFIÉE. »
--
-- La règle est étroite et la colonne l'est aussi : elle n'écarte pas le
-- prestataire de toute l'assemblée, seulement des points qui le concernent.
-- NULL est le cas courant — la plupart des résolutions ne portent la mission
-- de personne.

ALTER TABLE resolutions
    ADD COLUMN IF NOT EXISTS prestataire_de_la_mission UUID;

COMMENT ON COLUMN resolutions.prestataire_de_la_mission IS
    'Personne dont la mission fait l''objet de la délibération (Art. 3.87 § 9). Elle ne peut y prendre part, ni personnellement ni par procuration.';

-- Migration : resolutions.agenda_item_index — Issue #310, Art. 3.87 CC
--
-- Le champ existe deja dans TOUTE l'application :
--
--   entite            resolution.rs      pub agenda_item_index: Option<usize>
--   DTO de requete    resolution_dto.rs  Optional link to agenda item
--   DTO de reponse    resolution_dto.rs  agenda_item_index
--   handler           create_resolution passe la valeur au use case
--   validation        resolution_use_cases.rs verifie que l'indice designe un
--                     point REEL et non vide de l'ordre du jour
--
--   « Resolution must correspond to a valid agenda item (Art. 3.87 CC) »
--
-- ... sauf en base. La colonne n'a jamais ete creee, et
-- `resolution_repository_impl.rs` code `agenda_item_index: None` en dur dans
-- ses trois mappages.
--
-- Consequence : le lien entre une resolution et son point a l'ordre du jour est
-- VALIDE a la creation puis immediatement PERDU. Un proces-verbal ne peut donc
-- pas indiquer a quel point chaque resolution se rattache, alors que c'est
-- precisement ce que l'article exige pour qu'elle soit opposable.
--
-- Detecte le 2026-08-27 par `e2e_resolution_agenda`, harnais ecrit pour
-- l'issue #310 et jamais cable en CI : il n'avait donc jamais pu signaler que
-- la fonctionnalite qu'il teste n'etait pas persistee.
--
-- Additive et forward-only (convention du projet : pas de fichier `down`).
-- Colonne NULLABLE : les resolutions existantes n'ont pas cette information et
-- rien ne permet de la reconstituer apres coup. Un defaut a 0 serait faux — il
-- affirmerait un rattachement au premier point de l'ordre du jour.

ALTER TABLE resolutions
    ADD COLUMN IF NOT EXISTS agenda_item_index INTEGER;

-- L'indice reference `meetings.agenda[]`, tableau ordonne : il ne peut pas
-- etre negatif. La borne haute depend de la longueur de l'ordre du jour de
-- CHAQUE assemblee et releve donc du domaine, ou elle est deja verifiee.
ALTER TABLE resolutions
    ADD CONSTRAINT resolutions_agenda_item_index_non_negative
        CHECK (agenda_item_index IS NULL OR agenda_item_index >= 0);

COMMENT ON COLUMN resolutions.agenda_item_index IS
    'Index (base 0) du point de l''ordre du jour auquel la resolution se rattache. Art. 3.87 CC : une resolution doit correspondre a un point inscrit. NULL pour les resolutions anterieures a cette migration.';

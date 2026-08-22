-- Story H17 (Track H, CL3) — représentant de vote / suspension (Art. 3.87 §1).
--
-- Un lot peut appartenir à plusieurs titulaires (indivision) OU être démembré
-- (usufruit/nue-propriété, emphytéose, superficie). Dans ce cas le droit de
-- vote est SUSPENDU jusqu'à désignation d'un représentant unique (mandataire
-- commun) qui exercera le vote. Cf. ADR-0011 + spec H17.
--
-- On qualifie chaque ligne `unit_owners` :
--  - `ownership_type` : nature de la titularité (défaut `full_owner` =
--    rétro-compat des relations existantes, traitées comme pleine propriété).
--  - `is_voting_representative` : le titulaire désigné pour voter au nom du lot
--    démembré/indivis (au plus un par lot — invariant domaine, pas DB).
--
-- NOT NULL + DEFAULT : les lignes existantes deviennent `full_owner` / non
-- représentant, donc voting_right_status = Active (aucune régression de vote).
-- `is_voting_representative` distinct de `is_primary_contact` (contact
-- administratif/facturation ≠ représentant de vote légal).

ALTER TABLE unit_owners
    ADD COLUMN ownership_type TEXT NOT NULL DEFAULT 'full_owner';

ALTER TABLE unit_owners
    ADD CONSTRAINT valid_ownership_type
    CHECK (ownership_type IN (
        'full_owner', 'usufruct', 'bare_owner',
        'indivisaire', 'emphyteote', 'superficiaire'
    ));

ALTER TABLE unit_owners
    ADD COLUMN is_voting_representative BOOLEAN NOT NULL DEFAULT false;

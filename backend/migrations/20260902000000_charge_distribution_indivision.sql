-- Rend la répartition des charges possible sur un lot en indivision.
--
-- POURQUOI
--   `charge_distributions` portait `UNIQUE (expense_id, unit_id)`. Un lot
--   détenu par deux copropriétaires — un couple, une succession, un
--   démembrement usufruit/nue-propriété — produit DEUX lignes de répartition
--   pour ce même lot. La contrainte les refusait.
--
--   L'insertion étant groupée (`create_bulk`), l'échec ne portait pas sur la
--   seule ligne fautive : AUCUNE répartition n'était enregistrée pour
--   l'immeuble entier. Mesuré en production le 2026-09-02 :
--
--     POST /invoices/{id}/calculate-distribution
--       → 400  duplicate key value violates unique constraint
--              "unique_expense_unit"
--
--   L'indivision est le cas ORDINAIRE en copropriété belge, pas un cas
--   limite : un appartement acheté par un couple s'y trouve d'office.
--
--   Le domaine, lui, la gère : `ChargeDistribution::resolve_owner_quota`
--   multiplie explicitement par `ownership_percentage` et son test couvre
--   « Indivision 50/50 : 0,25 × 0,5 = 0,125 ». Le modèle supportait ce que
--   le schéma interdisait.
--
-- CE QUE LA NOUVELLE CONTRAINTE GARANTIT ENCORE
--   Un même copropriétaire ne peut pas se voir répartir deux fois la même
--   charge sur le même lot — ce qui reste le doublon à empêcher. Seule la
--   pluralité de détenteurs d'un lot devient possible.
--
--   Le garde-fou sur la SOMME des quotités reste porté ailleurs, et à deux
--   niveaux : `validate_unit_ownership_total` (trigger sur `unit_owners`,
--   somme des détentions d'un lot ≤ 100 %) et `QuotaSumExceeds` dans le
--   domaine (somme des quotes-parts d'un immeuble ≤ 100 %).

ALTER TABLE charge_distributions
    DROP CONSTRAINT IF EXISTS unique_expense_unit;

ALTER TABLE charge_distributions
    DROP CONSTRAINT IF EXISTS unique_expense_unit_owner;

ALTER TABLE charge_distributions
    ADD CONSTRAINT unique_expense_unit_owner
        UNIQUE (expense_id, unit_id, owner_id);

COMMENT ON CONSTRAINT unique_expense_unit_owner ON charge_distributions IS
    'Une ligne par (charge, lot, coproprietaire). Plusieurs lignes par lot sont attendues en indivision.';

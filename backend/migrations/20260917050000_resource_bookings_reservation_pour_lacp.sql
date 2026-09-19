-- Story #588 — les trois colonnes de la réservation « pour le compte de
-- l'ACP », que le dépôt lisait et écrivait déjà.
--
-- Quatrième occurrence du même défaut (#581, #48, fund_reassignments) : le
-- code, l'entité et les requêtes livrés, la migration jamais écrite. Le
-- dépôt emploie des requêtes vérifiées à l'exécution — toute création de
-- réservation rendait 400, ce que trois specs `Bookings.spec.ts`
-- constataient sans pouvoir le nommer.
--
-- Types repris de l'entité (`domain/economie_circulaire/resource_booking.rs:69`) :
--   booked_by_user_id : Option<Uuid>  → nullable
--   on_behalf_of_acp  : bool          → NOT NULL, faux par défaut
--   motif             : Option<String> → nullable

ALTER TABLE resource_bookings
    ADD COLUMN IF NOT EXISTS booked_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS on_behalf_of_acp BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS motif TEXT;

-- « motif obligatoire si on_behalf_of_acp » est une règle du domaine
-- (commentaire de `ResourceBooking::motif`). Elle est posée ICI aussi, parce
-- qu'une règle qui ne vit que dans le use-case laisse la base accepter des
-- lignes que le domaine juge impossibles — et ce sont ces lignes-là qu'on
-- retrouve six mois plus tard sans savoir qui les a écrites.
--
-- NOT VALID : la contrainte s'applique aux écritures FUTURES sans exiger que
-- l'existant s'y conforme. Les réservations déjà en base ont
-- `on_behalf_of_acp = FALSE` par le défaut ci-dessus, donc la satisfont de
-- toute façon ; `NOT VALID` évite simplement un scan complet au déploiement.
ALTER TABLE resource_bookings
    ADD CONSTRAINT resource_bookings_motif_si_pour_lacp CHECK (
        NOT on_behalf_of_acp OR motif IS NOT NULL
    ) NOT VALID;

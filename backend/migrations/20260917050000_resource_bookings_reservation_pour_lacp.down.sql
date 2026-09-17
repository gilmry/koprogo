ALTER TABLE resource_bookings
    DROP CONSTRAINT IF EXISTS resource_bookings_motif_si_pour_lacp;
ALTER TABLE resource_bookings
    DROP COLUMN IF EXISTS motif,
    DROP COLUMN IF EXISTS on_behalf_of_acp,
    DROP COLUMN IF EXISTS booked_by_user_id;

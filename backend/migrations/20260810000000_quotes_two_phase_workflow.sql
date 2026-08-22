-- Migration: Quotes become a true 2-phase workflow (Requested -> Received)
--
-- POST /quotes ("Demander un devis", QuoteList.svelte) only ever sends
-- building_id/contractor_id/project_title/project_description/work_category —
-- it never asks the syndic for a price, because at request time nobody knows
-- it yet. But amount_excl_vat/vat_rate/amount_incl_vat/validity_date/
-- estimated_duration_days were NOT NULL, so every single "Demander un devis"
-- failed with a 400 (missing field). The real price arrives later via
-- POST /quotes/{id}/submit (contractor's actual quote).
ALTER TABLE quotes
    ALTER COLUMN amount_excl_vat DROP NOT NULL,
    ALTER COLUMN vat_rate DROP NOT NULL,
    ALTER COLUMN amount_incl_vat DROP NOT NULL,
    ALTER COLUMN validity_date DROP NOT NULL,
    ALTER COLUMN estimated_duration_days DROP NOT NULL;

-- work_category: collected by the create form and displayed on quote
-- cards/detail (QuoteList.svelte, QuoteDetail.svelte), but never had a
-- column to persist to — silently dropped by serde on every request.
ALTER TABLE quotes ADD COLUMN work_category VARCHAR(100);

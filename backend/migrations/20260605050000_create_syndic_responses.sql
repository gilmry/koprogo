-- Story 3.7 (FR32, INV-23) — SyndicResponse (append-only) + SLA tracking.
--
-- A `SyndicResponse` materialises a syndic's structured reply to a ticket
-- (typically a Complaint, but the relation accepts any ticket kind). Replies
-- are append-only: trying to UPDATE or DELETE a row raises a typed error.
-- Audit fidelity (INV-23 — "what the syndic said and when") is a strict
-- invariant.
--
-- Tickets also receive two SLA tracking columns:
--   - `sla_due_at`        : computed at ticket creation from severity tier
--   - `sla_escalated_at`  : set by the SLA escalation cron job OR by the
--                           use-case when a syndic response arrives before
--                           the deadline (cancels the upcoming escalation,
--                           idempotent).

-- 1) SyndicResponse table (append-only — pas de UPDATE ni DELETE).
CREATE TABLE syndic_responses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id       UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    syndic_user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body            TEXT NOT NULL CHECK (length(body) >= 10 AND length(body) <= 5000),
    action_proposed VARCHAR(64),  -- whitelist enforced application-side (Story 3.7)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_syndic_responses_ticket
    ON syndic_responses(ticket_id, created_at);

-- 2) Ticket.sla_due_at calculated at create-time from severity, plus
--    sla_escalated_at set by the cron when the deadline is missed (or
--    pre-empted by the use-case when a response arrives in time).
ALTER TABLE tickets
    ADD COLUMN sla_due_at TIMESTAMPTZ,
    ADD COLUMN sla_escalated_at TIMESTAMPTZ;

-- Partial index lets the cron job scan only the open SLA window cheaply.
CREATE INDEX idx_tickets_sla_pending
    ON tickets(sla_due_at)
    WHERE sla_escalated_at IS NULL AND sla_due_at IS NOT NULL;

-- 3) Audit immutability : explicit DB-level guard against UPDATE/DELETE on
--    syndic_responses. Even a privileged direct SQL access cannot mutate
--    a recorded reply (INV-23 ironclad).
CREATE OR REPLACE FUNCTION reject_syndic_response_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'SyndicResponse is append-only (Story 3.7 INV-23)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER syndic_responses_no_update
    BEFORE UPDATE OR DELETE ON syndic_responses
    FOR EACH ROW
    EXECUTE FUNCTION reject_syndic_response_mutation();

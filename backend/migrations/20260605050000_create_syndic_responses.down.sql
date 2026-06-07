-- Story 3.7 — DOWN migration for SyndicResponse + SLA tracking.

DROP TRIGGER IF EXISTS syndic_responses_no_update ON syndic_responses;
DROP FUNCTION IF EXISTS reject_syndic_response_mutation();

DROP INDEX IF EXISTS idx_tickets_sla_pending;

ALTER TABLE tickets
    DROP COLUMN IF EXISTS sla_escalated_at,
    DROP COLUMN IF EXISTS sla_due_at;

DROP INDEX IF EXISTS idx_syndic_responses_ticket;
DROP TABLE IF EXISTS syndic_responses;

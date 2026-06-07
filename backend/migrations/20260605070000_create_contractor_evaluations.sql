-- Story 3.9 (FR34 FR35, INV-21 INV-24) — ContractorEvaluation (append-only)
-- gated by an approved TechnicalSpec (Story 3.8).
--
-- A `contractor_evaluation` records the quality of a contractor's prestation
-- on a previously-signed TechnicalSpec. Rows are append-only: trying to
-- UPDATE or DELETE raises a typed error. Audit fidelity (INV-24 — "what was
-- rated, by whom, when") is a strict invariant.
--
-- The application layer additionally enforces FR34 by refusing to create a
-- row unless the referenced `technical_spec_id` is in status `approved`
-- (use-case-level guard; the FK below only proves the spec EXISTS, not its
-- workflow state).

CREATE TABLE contractor_evaluations (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contractor_user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    technical_spec_id     UUID NOT NULL REFERENCES technical_specs(id) ON DELETE RESTRICT,
    linked_ticket_ids     UUID[] NOT NULL DEFAULT '{}',
    evaluator_user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    score_quality         SMALLINT NOT NULL CHECK (score_quality BETWEEN 1 AND 5),
    score_timeliness      SMALLINT NOT NULL CHECK (score_timeliness BETWEEN 1 AND 5),
    score_communication   SMALLINT NOT NULL CHECK (score_communication BETWEEN 1 AND 5),
    score_cost_compliance SMALLINT NOT NULL CHECK (score_cost_compliance BETWEEN 1 AND 5),
    score_overall         SMALLINT NOT NULL CHECK (score_overall BETWEEN 1 AND 5),
    comment               TEXT NOT NULL CHECK (length(comment) >= 10 AND length(comment) <= 2000),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT contractor_eval_no_self_eval CHECK (evaluator_user_id <> contractor_user_id)
);

CREATE INDEX idx_contractor_eval_contractor ON contractor_evaluations(contractor_user_id);
CREATE INDEX idx_contractor_eval_spec ON contractor_evaluations(technical_spec_id);

-- Append-only guard (INV-24). Mirrors the Story 3.7 SyndicResponse trigger
-- and the Story 3.8 TechnicalSpecSignature trigger.
CREATE OR REPLACE FUNCTION reject_contractor_eval_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ContractorEvaluation is append-only (Story 3.9 INV-24)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER contractor_eval_no_mutation
    BEFORE UPDATE OR DELETE ON contractor_evaluations
    FOR EACH ROW
    EXECUTE FUNCTION reject_contractor_eval_mutation();

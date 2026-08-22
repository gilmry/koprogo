-- Story 3.9 (FR34 FR35, INV-24) — DOWN migration for ContractorEvaluation.

DROP TRIGGER IF EXISTS contractor_eval_no_mutation ON contractor_evaluations;
DROP FUNCTION IF EXISTS reject_contractor_eval_mutation();

DROP INDEX IF EXISTS idx_contractor_eval_spec;
DROP INDEX IF EXISTS idx_contractor_eval_contractor;
DROP TABLE IF EXISTS contractor_evaluations;

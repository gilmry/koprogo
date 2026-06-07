-- Story 3.8 (FR33) — DOWN migration for TechnicalSpec + signatures.

DROP TRIGGER IF EXISTS tech_spec_sig_no_mutation ON technical_spec_signatures;
DROP FUNCTION IF EXISTS reject_tech_spec_sig_mutation();

DROP INDEX IF EXISTS idx_tech_spec_sig_spec;
DROP TABLE IF EXISTS technical_spec_signatures;

DROP INDEX IF EXISTS idx_technical_specs_building;
DROP INDEX IF EXISTS idx_technical_specs_status;
DROP INDEX IF EXISTS idx_technical_specs_acp;
DROP TABLE IF EXISTS technical_specs;

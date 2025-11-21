-- Migration: Add unit ownership validation trigger (Issue #29)
-- Description: Enforce that total ownership percentage for a unit equals 100% (with 0.01% tolerance)
-- Date: 2025-11-17
-- Belgian Legal Requirement: Total quotes-parts must equal 100% for each lot

-- =========================================
-- Function: Validate total ownership = 100%
-- =========================================
CREATE OR REPLACE FUNCTION validate_unit_ownership_total()
RETURNS TRIGGER AS $$
DECLARE
    total_percentage DECIMAL(10,4);
    active_owners_count INTEGER;
BEGIN
    -- Calculate total active ownership percentage for the unit
    SELECT COALESCE(SUM(ownership_percentage), 0), COUNT(*)
    INTO total_percentage, active_owners_count
    FROM unit_owners
    WHERE unit_id = COALESCE(NEW.unit_id, OLD.unit_id)
      AND end_date IS NULL;  -- Only active ownerships

    -- Allow empty state (no owners yet)
    IF active_owners_count = 0 THEN
        RETURN NEW;
    END IF;

    -- Validate total equals 100% with ±0.01% tolerance for rounding
    IF total_percentage > 1.0001 THEN
        RAISE EXCEPTION 'Total ownership for unit % exceeds 100%% (current: %.2f%%)',
            COALESCE(NEW.unit_id, OLD.unit_id),
            total_percentage * 100
            USING ERRCODE = '23514',  -- check_violation
                  HINT = 'Reduce ownership percentages to not exceed 100% total';
    END IF;

    -- Warning: Total is less than 100% but don't block (allow transitional states)
    IF total_percentage < 0.9999 AND active_owners_count > 0 THEN
        RAISE WARNING 'Total ownership for unit % is below 100%% (current: %.2f%%). This should be corrected.',
            COALESCE(NEW.unit_id, OLD.unit_id),
            total_percentage * 100;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =========================================
-- Trigger: Fire after INSERT/UPDATE/DELETE
-- =========================================
DROP TRIGGER IF EXISTS trigger_validate_unit_ownership ON unit_owners;

CREATE TRIGGER trigger_validate_unit_ownership
    AFTER INSERT OR UPDATE OF ownership_percentage, end_date OR DELETE
    ON unit_owners
    FOR EACH ROW
    EXECUTE FUNCTION validate_unit_ownership_total();

-- =========================================
-- Comments for documentation
-- =========================================
COMMENT ON FUNCTION validate_unit_ownership_total() IS
'Validates that total active ownership percentage for a unit does not exceed 100%.
Allows transitional states below 100% with WARNING.
Tolerance: ±0.01% for floating-point rounding errors.
Belgian Legal Requirement: Article 577-2 §4 Code Civil (quotes-parts must equal 100%)';

COMMENT ON TRIGGER trigger_validate_unit_ownership ON unit_owners IS
'Enforces ownership percentage validation after any change to unit_owners table.
Prevents exceeding 100%, warns when below 100% (Issue #29)';

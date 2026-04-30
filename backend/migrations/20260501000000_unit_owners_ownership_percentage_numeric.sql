-- Migration: unit_owners.ownership_percentage DOUBLE PRECISION → NUMERIC(6,5)
-- Description: Align SQL with ADR-0008 (NUMERIC for monetary-adjacent fields).
--              Quote-parts drive charge distribution: exactness Decimal critical.
-- Date: 2026-05-01
-- Refs: SQL-MIGRATION-001 (#438), ADR-0008
-- Story follow-up: EXP-003 (#437) — supprime helpers decimal_to_f64/f64_to_decimal

-- =========================================
-- Step 1: DROP view + trigger dépendants
-- =========================================
-- PG refuse ALTER COLUMN si view ou trigger référence la colonne (par nom)
DROP VIEW IF EXISTS v_current_unit_owners;
DROP TRIGGER IF EXISTS trigger_validate_unit_ownership ON unit_owners;

-- =========================================
-- Step 2: ALTER TABLE column type
-- =========================================
-- NUMERIC(6,5) = max 1.00000, 5 décimales (1/10000ème de %).
-- Value range conservé : 0 < x <= 1.
ALTER TABLE unit_owners
    ALTER COLUMN ownership_percentage TYPE NUMERIC(6, 5)
    USING ownership_percentage::NUMERIC(6, 5);

-- Default value (1.0 = 100%) reste compatible
ALTER TABLE unit_owners
    ALTER COLUMN ownership_percentage SET DEFAULT 1.00000;

-- =========================================
-- Step 3: Re-créer la view avec le nouveau type propagé
-- =========================================
CREATE OR REPLACE VIEW v_current_unit_owners AS
SELECT
    uo.id,
    uo.unit_id,
    uo.owner_id,
    uo.ownership_percentage,
    uo.is_primary_contact,
    u.building_id,
    u.unit_number,
    u.unit_type,
    o.first_name,
    o.last_name,
    o.email,
    o.phone,
    o.organization_id
FROM unit_owners uo
JOIN units u ON uo.unit_id = u.id
JOIN owners o ON uo.owner_id = o.id
WHERE uo.end_date IS NULL;

-- =========================================
-- Step 4: Re-créer le trigger (function inchangée)
-- =========================================
CREATE TRIGGER trigger_validate_unit_ownership
    AFTER INSERT OR UPDATE OF ownership_percentage, end_date OR DELETE
    ON unit_owners
    FOR EACH ROW
    EXECUTE FUNCTION validate_unit_ownership_total();

-- =========================================
-- Step 5: Re-créer les fonctions PG (return type changed)
-- =========================================

DROP FUNCTION IF EXISTS get_unit_owners(UUID);

CREATE FUNCTION get_unit_owners(p_unit_id UUID)
RETURNS TABLE (
    owner_id UUID,
    first_name VARCHAR,
    last_name VARCHAR,
    email VARCHAR,
    phone VARCHAR,
    ownership_percentage NUMERIC(6, 5),
    is_primary_contact BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        o.id,
        o.first_name,
        o.last_name,
        o.email,
        o.phone,
        uo.ownership_percentage,
        uo.is_primary_contact
    FROM unit_owners uo
    JOIN owners o ON uo.owner_id = o.id
    WHERE uo.unit_id = p_unit_id
      AND uo.end_date IS NULL
    ORDER BY uo.is_primary_contact DESC, o.last_name, o.first_name;
END;
$$ LANGUAGE plpgsql;

DROP FUNCTION IF EXISTS get_owner_units(UUID);

CREATE FUNCTION get_owner_units(p_owner_id UUID)
RETURNS TABLE (
    unit_id UUID,
    building_id UUID,
    building_name VARCHAR,
    unit_number VARCHAR,
    unit_type TEXT,
    ownership_percentage NUMERIC(6, 5),
    is_primary_contact BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.building_id,
        b.name as building_name,
        u.unit_number,
        u.unit_type::TEXT,
        uo.ownership_percentage,
        uo.is_primary_contact
    FROM unit_owners uo
    JOIN units u ON uo.unit_id = u.id
    JOIN buildings b ON u.building_id = b.id
    WHERE uo.owner_id = p_owner_id
      AND uo.end_date IS NULL
    ORDER BY b.name, u.unit_number;
END;
$$ LANGUAGE plpgsql;

-- =========================================
-- Notes
-- =========================================
-- - View `v_current_unit_owners` recréée explicitement.
-- - Trigger `trigger_validate_unit_ownership` recréé explicitement
--   (function `validate_unit_ownership_total` inchangée — DECIMAL(10,4)
--   interne, SUM auto-adapt).
-- - CHECK constraint (`> 0 AND <= 1.0`) reste valide pour NUMERIC.

COMMENT ON COLUMN unit_owners.ownership_percentage IS
'Quote-part NUMERIC(6,5) (cf. ADR-0008). Range 0 < x <= 1. Drives charge distribution.';

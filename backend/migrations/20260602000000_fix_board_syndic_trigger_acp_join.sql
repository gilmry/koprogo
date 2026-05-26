-- Hotfix #604 — Migrate board/syndic incompatibility triggers to acps.organization_id
--
-- The migration `20260601040000_buildings_acp_id_not_null.sql` (Story 1.2)
-- dropped `buildings.organization_id`. Two PL/pgSQL triggers from
-- `20251101000002_create_board_system.sql` still referenced that column :
--
--   - `check_syndic_board_incompatibility()` (board_members trigger) :
--     SELECT organization_id FROM buildings WHERE id = ...
--
--   - `check_board_syndic_incompatibility()` (user_roles trigger) :
--     b.organization_id = NEW.organization_id
--
-- Symptom : `POST /auth/register role=syndic` returns 400 ("legal
-- incompatibility" misfires OR raw SQL error "column organization_id
-- does not exist") which blocked seed/demo + characterization E2E gate.
--
-- Fix : DROP + CREATE OR REPLACE both functions with JOIN via `acps`
-- (canonical post-#602 path to organization_id). Triggers are recreated
-- identical (same name, same table, same timing).
--
-- Rollback : `20260602000000_fix_board_syndic_trigger_acp_join.down.sql`
-- restores the previous broken-but-original function bodies.

-- ============================================================================
-- Function 1 : check_syndic_board_incompatibility
-- Trigger : enforce_syndic_board_incompatibility on board_members
-- ============================================================================

CREATE OR REPLACE FUNCTION check_syndic_board_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if the owner is linked to a user who is syndic for the same building
    -- Hotfix #604 : resolve building.organization_id via acps JOIN
    IF EXISTS (
        SELECT 1
        FROM owners o
        INNER JOIN user_roles ur ON ur.user_id = o.user_id
        WHERE o.id = NEW.owner_id
          AND ur.role = 'syndic'
          AND ur.organization_id = (
              SELECT a.organization_id
              FROM buildings b
              JOIN acps a ON a.id = b.acp_id
              WHERE b.id = NEW.building_id
          )
    ) THEN
        RAISE EXCEPTION 'A syndic cannot be a board member for the same building (legal incompatibility)';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Function 2 : check_board_syndic_incompatibility
-- Trigger : enforce_board_syndic_incompatibility on user_roles
-- ============================================================================

CREATE OR REPLACE FUNCTION check_board_syndic_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    -- Only check if role is syndic
    IF NEW.role = 'syndic' THEN
        -- Check if user is linked to an owner who is a board member for any building in the same organization
        -- Hotfix #604 : resolve building.organization_id via acps JOIN
        IF EXISTS (
            SELECT 1
            FROM owners o
            INNER JOIN board_members bm ON bm.owner_id = o.id
            INNER JOIN buildings b ON bm.building_id = b.id
            INNER JOIN acps a ON a.id = b.acp_id
            WHERE o.user_id = NEW.user_id
              AND a.organization_id = NEW.organization_id
              AND bm.mandate_end > CURRENT_TIMESTAMP -- Only active mandates
        ) THEN
            RAISE EXCEPTION 'A board member cannot be a syndic for the same organization (legal incompatibility)';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Hotfix #604 — DOWN : restore the original (broken-but-pre-#602) function bodies.
--
-- WARNING : after applying this DOWN, both triggers will reference the dropped
-- `buildings.organization_id` column and will fail at runtime. This DOWN is
-- intended only for rolling back to a state where `buildings.organization_id`
-- has been re-created (e.g. full rollback chain 040000.down → 030000.down → ...).

CREATE OR REPLACE FUNCTION check_syndic_board_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM owners o
        INNER JOIN user_roles ur ON ur.user_id = o.user_id
        WHERE o.id = NEW.owner_id
          AND ur.role = 'syndic'
          AND ur.organization_id = (
              SELECT organization_id
              FROM buildings
              WHERE id = NEW.building_id
          )
    ) THEN
        RAISE EXCEPTION 'A syndic cannot be a board member for the same building (legal incompatibility)';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION check_board_syndic_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.role = 'syndic' THEN
        IF EXISTS (
            SELECT 1
            FROM owners o
            INNER JOIN board_members bm ON bm.owner_id = o.id
            INNER JOIN buildings b ON bm.building_id = b.id
            WHERE o.user_id = NEW.user_id
              AND b.organization_id = NEW.organization_id
              AND bm.mandate_end > CURRENT_TIMESTAMP
        ) THEN
            RAISE EXCEPTION 'A board member cannot be a syndic for the same organization (legal incompatibility)';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

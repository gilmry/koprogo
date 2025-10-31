-- Migration to restore superadmin role for admin@koprogo.com
-- This fixes cases where the superadmin role was accidentally removed or replaced

-- First, ensure the superadmin user exists and has the correct role in users table
UPDATE users
SET
    role = 'superadmin',
    organization_id = NULL,
    is_active = true,
    updated_at = NOW()
WHERE email = 'admin@koprogo.com';

-- Get the user_id for admin@koprogo.com
DO $$
DECLARE
    admin_user_id UUID;
    superadmin_role_exists BOOLEAN;
BEGIN
    -- Get admin user ID
    SELECT id INTO admin_user_id FROM users WHERE email = 'admin@koprogo.com';

    IF admin_user_id IS NULL THEN
        RAISE EXCEPTION 'Admin user not found';
    END IF;

    -- Check if superadmin role exists
    SELECT EXISTS(
        SELECT 1 FROM user_roles
        WHERE user_id = admin_user_id AND role = 'superadmin'
    ) INTO superadmin_role_exists;

    -- First, set all existing roles to non-primary
    UPDATE user_roles
    SET is_primary = false, updated_at = NOW()
    WHERE user_id = admin_user_id;

    -- If superadmin role exists, make it primary
    IF superadmin_role_exists THEN
        UPDATE user_roles
        SET is_primary = true, updated_at = NOW()
        WHERE user_id = admin_user_id AND role = 'superadmin';

        RAISE NOTICE '✅ Existing superadmin role set as primary';
    ELSE
        -- Insert new superadmin role as primary
        INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
        VALUES (gen_random_uuid(), admin_user_id, 'superadmin', NULL, true, NOW(), NOW());

        RAISE NOTICE '✅ New superadmin role created and set as primary';
    END IF;

    -- Remove any non-superadmin roles if needed (optional - uncomment to clean up)
    -- DELETE FROM user_roles
    -- WHERE user_id = admin_user_id AND role != 'superadmin';

END $$;

-- Log the fix
DO $$
BEGIN
    RAISE NOTICE '✅ Superadmin role restored for admin@koprogo.com';
END $$;

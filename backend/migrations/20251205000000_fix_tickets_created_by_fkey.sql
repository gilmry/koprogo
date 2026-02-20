-- Fix tickets.created_by FK: reference users(id) instead of owners(id)
-- The handler uses AuthenticatedUser.user_id (from users table),
-- not an owner_id. Any authenticated user can create a ticket.

ALTER TABLE tickets DROP CONSTRAINT IF EXISTS tickets_created_by_fkey;
ALTER TABLE tickets ADD CONSTRAINT tickets_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE;

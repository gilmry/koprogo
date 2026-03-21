-- Fix: notices.author_id should reference users(id), not owners(id)
-- A syndic is not an owner but must be able to post building notices.

-- Drop the old FK constraint (implicit name: notices_author_id_fkey)
ALTER TABLE notices DROP CONSTRAINT notices_author_id_fkey;

-- Add new FK pointing to users
ALTER TABLE notices
    ADD CONSTRAINT notices_author_id_fkey
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE;

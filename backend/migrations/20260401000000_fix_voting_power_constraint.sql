-- Migration: Relax voting_power constraint from 1000 to 10000
-- Bug: BUG-WF2-1 — Seed data has lots with >1000 tantièmes (Emmanuel = 1280)
-- The domain entity already validates <= 10000 (dix-millièmes)
-- This migration aligns the DB constraint with the domain rule

-- Drop the old constraint
ALTER TABLE votes DROP CONSTRAINT IF EXISTS votes_max_voting_power;

-- Add new constraint with higher limit
ALTER TABLE votes ADD CONSTRAINT votes_max_voting_power CHECK (voting_power <= 10000);

-- Update comment
COMMENT ON COLUMN votes.voting_power IS 'Pouvoir de vote en tantièmes du lot (max 10000 dix-millièmes)';

-- Migration: Add building_id and journal_type to journal_entries
--
-- CREDITS & ATTRIBUTION:
-- This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
-- Noalyss is a free accounting software for Belgian and French accounting
-- License: GPL-2.0-or-later (GNU General Public License version 2 or later)
-- Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
-- Copyright: Dany De Bontridder <dany@alchimerys.eu>
--
-- Noalyss features:
-- - Journal types (ACH=Purchases, VEN=Sales, FIN=Financial, ODS=Miscellaneous)
-- - Building-specific accounting for multi-property management

-- Add building_id to link journal entries to specific buildings
ALTER TABLE journal_entries
ADD COLUMN building_id UUID REFERENCES buildings(id) ON DELETE SET NULL;

CREATE INDEX idx_journal_entries_building ON journal_entries(building_id);

-- Add journal_type to categorize entries (Noalyss-inspired)
-- ACH = Achats (Purchases)
-- VEN = Ventes (Sales)
-- FIN = Financial operations
-- ODS = Opérations Diverses (Miscellaneous)
ALTER TABLE journal_entries
ADD COLUMN journal_type VARCHAR(10);

CREATE INDEX idx_journal_entries_journal_type ON journal_entries(journal_type);

-- Comments for documentation
COMMENT ON COLUMN journal_entries.building_id IS 'Optional link to building for building-specific accounting (multi-property management)';
COMMENT ON COLUMN journal_entries.journal_type IS 'Journal type: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous) - inspired by Noalyss';

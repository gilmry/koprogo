-- Update majority_required values in resolutions table
-- Old: Simple, Absolute, Qualified:<threshold>
-- New: Absolute, TwoThirds, FourFifths, Unanimity
-- Issue #350: Belgian copropriété voting rules (Art. 3.88 §1 Code Civil)

-- Rename 'Simple' to 'Absolute' (they were the same thing in Belgian law)
UPDATE resolutions SET majority_required = 'Absolute' WHERE majority_required = 'Simple';

-- Convert qualified thresholds to named types
-- Qualified was stored as "Qualified:<threshold>"
UPDATE resolutions SET majority_required = 'TwoThirds' WHERE majority_required LIKE 'Qualified:0.6%';
UPDATE resolutions SET majority_required = 'TwoThirds' WHERE majority_required LIKE 'Qualified:0.7%';
UPDATE resolutions SET majority_required = 'FourFifths' WHERE majority_required LIKE 'Qualified:0.8%';
UPDATE resolutions SET majority_required = 'Unanimity' WHERE majority_required LIKE 'Qualified:1%';

-- Any remaining qualified values default to two_thirds
UPDATE resolutions SET majority_required = 'TwoThirds' WHERE majority_required LIKE 'Qualified:%';

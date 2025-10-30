-- Add total_tantiemes column to buildings table
-- In Belgium, tantiemes (quote-parts) are typically expressed in thousandths (millièmes)
-- Default value is 1000 (representing 1000/1000ths)
ALTER TABLE buildings
ADD COLUMN total_tantiemes INTEGER NOT NULL DEFAULT 1000 CHECK (total_tantiemes > 0);

COMMENT ON COLUMN buildings.total_tantiemes IS 'Total number of shares (tantiemes/millièmes) for the building, typically 1000';

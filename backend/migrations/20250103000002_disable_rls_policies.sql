-- Disable Row-Level Security policies
-- RLS is too restrictive for SuperAdmin access
-- Organization isolation will be enforced at the application layer instead

ALTER TABLE buildings DISABLE ROW LEVEL SECURITY;
ALTER TABLE units DISABLE ROW LEVEL SECURITY;
ALTER TABLE owners DISABLE ROW LEVEL SECURITY;
ALTER TABLE expenses DISABLE ROW LEVEL SECURITY;
ALTER TABLE meetings DISABLE ROW LEVEL SECURITY;
ALTER TABLE documents DISABLE ROW LEVEL SECURITY;

-- Drop the isolation policies
DROP POLICY IF EXISTS buildings_isolation ON buildings;
DROP POLICY IF EXISTS units_isolation ON units;
DROP POLICY IF EXISTS owners_isolation ON owners;
DROP POLICY IF EXISTS expenses_isolation ON expenses;
DROP POLICY IF EXISTS meetings_isolation ON meetings;
DROP POLICY IF EXISTS documents_isolation ON documents;

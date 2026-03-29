-- Update quota constraint from 1000 (milliemes) to 10000 (dix-milliemes)
-- Belgian copropriete uses dix-milliemes (10000ths) for larger buildings (182+ lots)
-- Old constraint: quota > 0 AND quota <= 1000
-- New constraint: quota > 0 AND quota <= 10000

ALTER TABLE units DROP CONSTRAINT IF EXISTS units_quota_check;
ALTER TABLE units ADD CONSTRAINT units_quota_check CHECK (quota > 0 AND quota <= 10000);

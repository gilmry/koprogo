-- Add GDPR anonymization fields to users and owners tables
-- Supports GDPR Article 17 (Right to Erasure) compliance

-- Add anonymization tracking fields to users table
ALTER TABLE users
ADD COLUMN IF NOT EXISTS is_anonymized BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS anonymized_at TIMESTAMPTZ;

-- Add anonymization tracking fields to owners table
ALTER TABLE owners
ADD COLUMN IF NOT EXISTS is_anonymized BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS anonymized_at TIMESTAMPTZ;

-- Add indexes for efficient GDPR queries
CREATE INDEX IF NOT EXISTS idx_users_is_anonymized ON users(is_anonymized);
CREATE INDEX IF NOT EXISTS idx_owners_is_anonymized ON owners(is_anonymized);

-- Add comments for documentation
COMMENT ON COLUMN users.is_anonymized IS 'GDPR flag: indicates if user personal data has been anonymized';
COMMENT ON COLUMN users.anonymized_at IS 'GDPR timestamp: when user data was anonymized';
COMMENT ON COLUMN owners.is_anonymized IS 'GDPR flag: indicates if owner personal data has been anonymized';
COMMENT ON COLUMN owners.anonymized_at IS 'GDPR timestamp: when owner data was anonymized';

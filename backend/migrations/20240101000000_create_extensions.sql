-- Ensure required extensions for UUID generation are available
CREATE EXTENSION IF NOT EXISTS pgcrypto; -- provides gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; -- alternative uuid_generate_v4()


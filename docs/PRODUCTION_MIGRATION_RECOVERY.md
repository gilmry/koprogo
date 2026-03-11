# Production Migration Recovery Guide

## Problem: Superadmin Seed Constraint Violation

### Root Cause
The `seed_superadmin()` function was attempting to update `is_primary = true` on conflict, which violated the unique partial index `idx_user_roles_primary_per_user` on subsequent application restarts.

### Solution Applied
Modified `backend/src/infrastructure/database/seed.rs` line 60-74 to:
```sql
ON CONFLICT (user_id, role, organization_id)
DO UPDATE SET
    updated_at = NOW()
    -- REMOVED: is_primary = true (prevents constraint violation)
```

**Rationale**: `is_primary` should only be set on INSERT (first run), never on UPDATE (subsequent runs).

### Production Deployment Checklist

#### ✅ Pre-Deployment
1. **Backup Database**: `pg_dump -U koprogo koprogo_db > backup_$(date +%Y%m%d_%H%M%S).sql`
2. **Test on Staging**: Deploy to staging environment first
3. **Run Idempotence Tests**: `cargo test test_seed_superadmin_is_idempotent`

#### ✅ Deployment
1. **Stop Backend**: `systemctl stop koprogo-backend` (or Docker: `docker compose stop backend`)
2. **Verify Migrations**:
   ```sql
   SELECT version, checksum FROM _sqlx_migrations ORDER BY version DESC LIMIT 10;
   ```
3. **Apply New Code**: `git pull && cargo build --release`
4. **Start Backend**: Application will auto-seed superadmin idempotently

#### ✅ Post-Deployment Verification
1. **Check Logs**: `journalctl -u koprogo-backend | grep superadmin`
   - Expected: `✅ Superadmin ready: admin@koprogo.com`
   - **NOT** expected: `duplicate key value violates unique constraint`

2. **Verify Database State**:
   ```sql
   -- Should return exactly 1 row
   SELECT COUNT(*) FROM user_roles
   WHERE user_id = '00000000-0000-0000-0000-000000000001'
   AND is_primary = true;

   -- Should return 1 (superadmin role)
   SELECT role, is_primary FROM user_roles
   WHERE user_id = '00000000-0000-0000-0000-000000000001';
   ```

3. **Test Login**: `curl -X POST http://localhost:8080/api/v1/auth/login -d '{"email":"admin@koprogo.com","password":"admin123"}'`

### Rollback Plan
If deployment fails:
1. **Stop Backend**: `systemctl stop koprogo-backend`
2. **Restore Backup**: `psql -U koprogo koprogo_db < backup_YYYYMMDD_HHMMSS.sql`
3. **Revert Code**: `git checkout <previous-commit>`
4. **Start Backend**: `systemctl start koprogo-backend`

### Migration State Recovery (If Needed)

#### Scenario: Migrations table accidentally modified

**⚠️ WARNING**: Only use this in extreme cases. Never run in production without full backup.

```sql
-- Check current state
SELECT version FROM _sqlx_migrations ORDER BY version DESC;

-- If migrations are missing (e.g., only showing 20240101000006):
-- Option 1: Restore from backup (RECOMMENDED)
-- Option 2: Re-insert migration records (DANGEROUS - only if backup unavailable)

-- DO NOT RUN THIS unless you fully understand the implications:
-- This would require manually re-inserting all missing migration records
-- with correct checksums. Contact DevOps team instead.
```

### Testing Idempotence

Run tests with real PostgreSQL instance:
```bash
# Local dev
cargo test --test integration test_seed_superadmin_is_idempotent

# CI/CD
docker compose -f docker-compose.test.yml run --rm backend cargo test --lib test_seed_superadmin
```

### Monitoring

Add alerting for:
- Application startup failures (check for constraint violations in logs)
- Unexpected `user_roles` record counts for superadmin
- Failed login attempts for `admin@koprogo.com`

### Related Files
- Fix: `backend/src/infrastructure/database/seed.rs:60-74`
- Tests: `backend/src/infrastructure/database/seed.rs:2963-3097`
- Migration: `backend/migrations/20250130000000_add_user_roles.sql:15-17`

### Questions?
Contact: DevOps Team | Created: 2025-12-06

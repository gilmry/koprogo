# ADR 0003: PostgreSQL as Primary Database

- **Status**: Accepted
- **Date**: 2025-01-25
- **Track**: Software

## Context

KoproGo requires a database that supports:
1. **ACID transactions** for financial operations (double-entry accounting, payments)
2. **Complex queries** (ownership history, financial reports, aggregations)
3. **JSON/JSONB** for flexible schema evolution (user preferences, metadata)
4. **Foreign keys & constraints** to enforce referential integrity
5. **Multi-tenancy** with organization-level data isolation
6. **Mature Rust ecosystem** (compile-time query verification)

We evaluated PostgreSQL 15, MySQL 8.0, ScyllaDB, and MongoDB.

## Decision

We chose **PostgreSQL 15** as the primary database.

**Reasons**:
- ✅ **Rock-solid ACID guarantees** (critical for financial data)
- ✅ **Rich constraint system**: CHECK, UNIQUE, FK, custom domains
- ✅ **JSON/JSONB support**: Flexible schema for evolving features
- ✅ **Powerful query capabilities**: CTEs, window functions, aggregates
- ✅ **Excellent Rust integration**: sqlx with compile-time query verification
- ✅ **Mature ecosystem**: Proven at scale (Instagram, GitHub, Discord)
- ✅ **Open-source**: No vendor lock-in, AGPL-compatible
- ✅ **Cost-effective**: Runs efficiently on modest hardware (€33/month OVH VPS)

**sqlx integration**:
```rust
// Compile-time query verification
let building = sqlx::query_as!(
    Building,
    "SELECT * FROM buildings WHERE id = $1",
    id
)
.fetch_one(&pool)
.await?;
```

Queries are validated against actual database schema at compile time, catching typos and schema mismatches early.

## Consequences

**Positive**:
- ✅ **Strong data integrity**: CHECK constraints enforce business rules (quote-part ≤ 100%)
- ✅ **Compile-time query safety**: sqlx macros catch SQL errors before deployment
- ✅ **Rich querying**: Complex financial reports without ORMs
- ✅ **Proven reliability**: MVCC handles concurrency without application-level locking
- ✅ **Future-proof**: Supports partitioning, replication, sharding if needed

**Negative**:
- ⚠️ **Vertical scaling limits**: Single-server for MVP (acceptable for 5k copropriétés)
- ⚠️ **No native multi-region**: Requires replication setup (not needed until Phase 3)
- ⚠️ **Complex migrations**: Schema changes require careful migration strategy

**Performance** (October 2025 load tests):
- Connection pool: 10 connections
- Query latency P99: < 50ms
- Throughput: 287 req/s (single instance, bottleneck is business logic not DB)

## Alternatives Considered

1. **MySQL 8.0**:
   - ✅ Similar features, wide adoption
   - ❌ Weaker constraint system (no CHECK until 8.0)
   - ❌ Less robust JSON support
   - **Verdict**: PostgreSQL's constraints and JSON support preferred

2. **ScyllaDB** (planned for Phase 2):
   - ✅ Horizontal scalability, low latency
   - ❌ No transactions across partitions
   - ❌ Eventual consistency (not acceptable for accounting)
   - **Verdict**: Future addition for hot-path reads (metrics, logs), not primary DB

3. **MongoDB**:
   - ✅ Flexible schema, easy development
   - ❌ Weaker ACID guarantees (even with transactions)
   - ❌ No foreign keys (enforced application-side)
   - **Verdict**: Rejected due to financial data requirements

## Schema Design Highlights

**Multi-tenancy**:
- Every table has `organization_id` column
- Row-Level Security (RLS) enforces isolation (planned for Phase 2)

**Key constraints**:
```sql
-- Quote-part validation
CHECK (ownership_percentage > 0 AND ownership_percentage <= 1)

-- PCMN code format
CHECK (code ~ '^\d{2,6}$')

-- VAT rates
CHECK (vat_rate IN (0.06, 0.12, 0.21))

-- Temporal validity
CHECK (end_date IS NULL OR end_date > start_date)
```

**Migration strategy**:
- sqlx migrations (`backend/migrations/`)
- Semantic versioning (20250127000000_description.sql)
- Reversible migrations via `down.sql` (where possible)
- CI/CD validation before merge

## Configuration

**Connection string** (`backend/.env`):
```env
DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_db
```

**Production settings**:
- SSL mode: `require`
- Connection pool: 10 connections
- Statement timeout: 30s
- Idle timeout: 10 minutes

## Next Steps

- ✅ Implement core schema (Buildings, Units, Owners) (**Done**)
- ✅ Add PCMN accounting tables (**Done**, Issue #79)
- ⏳ Implement Row-Level Security for multi-tenancy (Phase 2)
- ⏳ Evaluate read replicas for horizontal scaling (Phase 3)
- ⏳ Consider adding ScyllaDB for hot-path reads (metrics, audit logs)

## References

- PostgreSQL 15 Documentation: https://www.postgresql.org/docs/15/
- sqlx GitHub: https://github.com/launchbadge/sqlx
- KoproGo migrations: `backend/migrations/`

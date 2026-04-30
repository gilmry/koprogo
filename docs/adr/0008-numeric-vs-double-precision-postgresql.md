# ADR 0008: PostgreSQL `NUMERIC` for Monetary Columns (not `DOUBLE PRECISION`)

- **Status**: Accepted
- **Date**: 2026-04-30
- **Track**: Software / Database / Finance
- **Authors**: `rust-expert` persona + `platform-engineer` review + @gilmry sign-off
- **Related**: [ADR 0007](0007-decimal-vs-f64-for-money.md) (Rust), [ADR 0003](0003-postgresql-database.md) (DB choice)

## Context

The audit of 2026-04-30 found **9 SQL migrations** declaring monetary columns as `DOUBLE PRECISION`:

| Migration | Tables impacted |
|---|---|
| `20240101000003_create_units.sql` | `units.area_m2` |
| `20240101000004_create_expenses.sql` | `expenses.amount`, `amount_excl_vat`, `vat_amount`, etc. — **CRITICAL** |
| `20250127000000_refactor_owners_multitenancy.sql` | `unit_owners.percentage` (quotité copro) |
| `20251107120000_create_payment_reminders.sql` | `payment_reminders.amount_due`, `penalty_amount`, etc. |
| `20251201000000_create_iot_readings.sql` | `iot_readings.value` (acceptable — see ADR 0009) |
| `20251203000000_create_work_reports.sql` | montants travaux |
| `20251203000001_create_technical_inspections.sql` | montants inspections |
| `20251204000000_create_energy_buying_groups.sql` | consommations (acceptable — see ADR 0009) |
| `20260312000000_add_quorum_to_meetings.sql` | quorum percentage |

PostgreSQL `DOUBLE PRECISION` is IEEE 754 binary64 — same drift issue as Rust `f64` (cf. [ADR 0007](0007-decimal-vs-f64-for-money.md)). Selecting a `DOUBLE PRECISION` column into a Rust `Decimal` via `sqlx` involves an **implicit `f64` round-trip** that loses precision regardless of the Rust type used.

PCMN compliance requires exactness end-to-end (DB column → repository → use case → DTO → API → frontend rendering).

## Decision

**Monetary columns in PostgreSQL MUST use `NUMERIC(precision, scale)`**, not `DOUBLE PRECISION`.

### Standard sizes

| Use case | Type | Rationale |
|---|---|---|
| Montants en EUR (factures, paiements, budgets, contributions) | `NUMERIC(15, 2)` | Up to 9,999,999,999,999.99 € — sufficient for any single transaction in copropriete context |
| Pourcentages (quotités, taux TVA, taux pénalité, voting power) | `NUMERIC(7, 4)` | Up to 999.9999% — preserves 4 decimal digits (e.g., 21.5000%, 8.0000%) |
| Surfaces (m²) | `NUMERIC(10, 2)` | Up to 99,999,999.99 m² — sufficient for buildings |
| Cents only (legacy `i64 cents` columns) | `BIGINT` | Acceptable for `payment.amount_cents` already in this format |

### Conversion via sqlx

`sqlx` natively maps `NUMERIC` ↔ `rust_decimal::Decimal` when the `bigdecimal` or `rust_decimal` feature is enabled in `Cargo.toml`. Verify in migration:

```toml
sqlx = { version = "0.8", features = ["postgres", "rust_decimal"] }
```

### Migration pattern

For each story migrating a Rust entity to `Decimal` (cf. [#433](https://github.com/gilmry/koprogo/issues/433)), include a paired SQL migration:

```sql
-- migrations/YYYYMMDDHHMMSS_alter_<table>_amounts_to_numeric.sql

ALTER TABLE expenses
    ALTER COLUMN amount TYPE NUMERIC(15, 2) USING amount::NUMERIC(15, 2),
    ALTER COLUMN amount_excl_vat TYPE NUMERIC(15, 2) USING amount_excl_vat::NUMERIC(15, 2),
    ALTER COLUMN vat_rate TYPE NUMERIC(7, 4) USING vat_rate::NUMERIC(7, 4),
    ALTER COLUMN vat_amount TYPE NUMERIC(15, 2) USING vat_amount::NUMERIC(15, 2),
    ALTER COLUMN amount_incl_vat TYPE NUMERIC(15, 2) USING amount_incl_vat::NUMERIC(15, 2);
```

The `USING column::NUMERIC(...)` cast is **lossy** (existing `DOUBLE PRECISION` values may have already drifted). For production data, an additional reconciliation step is required to verify aggregate sums match expected ledger totals after migration.

## Consequences

### Positive

- **End-to-end exactness** from DB to API
- **Native `sqlx` ↔ `rust_decimal::Decimal` mapping** without IEEE 754 intermediary
- **Aggregate query precision** : `SUM(amount)` returns exact `NUMERIC` instead of accumulating `DOUBLE PRECISION` errors
- **Index efficiency** : PostgreSQL indexes `NUMERIC` columns equivalently to `DOUBLE PRECISION` for monetary ranges
- **Constraint expressiveness** : `CHECK (amount >= 0)` works naturally on `NUMERIC`

### Negative

- **Migration risk** : `ALTER COLUMN ... TYPE` rewrites the table for non-trivial type changes. Long lock duration on large tables — schedule during low-traffic windows
- **Production data reconciliation** : existing `DOUBLE PRECISION` values may have drifted; may require correction migration after type change
- **Slight storage increase** : `NUMERIC(15, 2)` typically 8-10 bytes vs 8 bytes for `DOUBLE PRECISION`. Negligible for KoproGo scale (target 5,000 copropriétés)

### Neutral

- Performance cost for arithmetic ops on `NUMERIC` is software-emulated. PostgreSQL handles this efficiently; the bottleneck remains network/disk I/O, not arithmetic

## Alternatives Considered

### Keep `DOUBLE PRECISION`, validate exactness in application layer
Rejected. Cumulative drift makes application-level validation impossible without reconciling each operation against an exact log. Defense-in-depth requires DB-level type correctness.

### Use `MONEY` PostgreSQL type
Rejected. PostgreSQL `MONEY` has localization issues (currency symbol, locale), depends on `lc_monetary` GUC which can change between database restarts, and is generally discouraged in modern PG documentation.

### Store everything as cents (`BIGINT`)
Considered. Acceptable for legacy modules (`payment.amount_cents`). New code prefers `NUMERIC` for flexibility (handles percentages and non-EUR units without cent base assumption). Mixing both in the same schema is acceptable as long as the application layer types match (`i64` ↔ `BIGINT`, `Decimal` ↔ `NUMERIC`).

## Implementation

- Each migration in [#433](https://github.com/gilmry/koprogo/issues/433) follow-up stories includes the corresponding `ALTER TABLE` SQL
- New columns added in any future migration **must default to `NUMERIC(...)` for monetary fields** — `DOUBLE PRECISION` is reserved for IoT/energy measurements (cf. [ADR 0009](0009-iot-energy-keep-f64.md))

## Enforcement

- **CI lint** : `grep -E 'DOUBLE PRECISION' backend/migrations/*.sql | grep -v iot | grep -v energy` must return only legacy migrations awaiting follow-up story migration
- **PR review** : `platform-engineer` persona rejects any new migration introducing `DOUBLE PRECISION` for monetary fields
- **Code review** : `code-reviewer` flags PRs that mix `Decimal` Rust + `DOUBLE PRECISION` SQL in the same vertical

## References

- ADR 0007 (Rust side) : [`0007-decimal-vs-f64-for-money.md`](0007-decimal-vs-f64-for-money.md)
- Audit : [`docs/audit/2026-04-30-f64-monetary-audit.md`](../audit/2026-04-30-f64-monetary-audit.md) §Migrations SQL
- PostgreSQL docs `NUMERIC` : https://www.postgresql.org/docs/15/datatype-numeric.html#DATATYPE-NUMERIC-DECIMAL
- `sqlx` rust_decimal mapping : https://docs.rs/sqlx/latest/sqlx/types/index.html
- KoproGo : [#425](https://github.com/gilmry/koprogo/issues/425), [#433](https://github.com/gilmry/koprogo/issues/433)

🤖 ADR drafted by `rust-expert` persona with `platform-engineer` cross-review — Tier 1 acceptance pending @gilmry sign-off.

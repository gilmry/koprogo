# ADR 0009: IoT and Energy Domains May Keep `f64` (Non-Monetary Measurements)

- **Status**: Accepted
- **Date**: 2026-04-30
- **Track**: Software / IoT / Energy
- **Authors**: `rust-expert` persona + @gilmry sign-off
- **Related**: [ADR 0007](0007-decimal-vs-f64-for-money.md) (forbids f64 in monetary code), [ADR 0008](0008-numeric-vs-double-precision-postgresql.md) (DB)

## Context

[ADR 0007](0007-decimal-vs-f64-for-money.md) forbids `f64` in monetary and accounting code due to PCMN exactness requirements. However, the audit of 2026-04-30 also found `f64` extensively in:

| Module | f64 count | Domain |
|---|---|---|
| `domain/entities/energy_campaign.rs` | 16 | Energy buying groups (kWh, prices) |
| `application/dto/energy_campaign_dto.rs` | 19 | idem |
| `application/use_cases/energy_campaign_use_cases.rs` | 9 | idem |
| `application/dto/iot_dto.rs` | 18 | IoT readings (temperature, water flow, occupancy sensors) |
| (future) IoT domain entities | TBD | physical measurements |

These domains have **fundamentally different characteristics** from accounting:

1. **Statistical aggregation** : data is averaged, smoothed, k-anonymized (k ≥ 5 for GDPR) before display. Centime-level exactness is **not meaningful** (a temperature of 20.001°C vs 20.000°C is below sensor precision).
2. **Sensor precision** : industrial IoT sensors typically have 0.1-1% relative error. IEEE 754 drift (~10⁻¹⁵) is **15+ orders of magnitude below sensor noise**.
3. **No legal exactness requirement** : energy consumption reports are informational, not invoices. Energy buying groups (campagnes ichoosr-style) compute estimated savings, not invoiced amounts.
4. **Performance** : IoT pipelines may process millions of readings per day. `f64` hardware-accelerated arithmetic is significantly faster than software-emulated `Decimal`.

## Decision

**`f64` is acceptable for IoT and energy measurement domains** in KoproGo. Specifically:

### Allowed `f64` paths

- `backend/src/domain/entities/energy_campaign.rs` and related entities (energy_buying_group, energy_consumption_estimate, etc.)
- `backend/src/domain/entities/iot_*.rs` (sensor readings, environmental data)
- `backend/src/application/dto/energy_*.rs` and `iot_*.rs`
- `backend/src/application/use_cases/energy_*.rs` and `iot_*.rs` (when computing aggregates, averages, statistics — NOT when computing invoiced amounts)
- `backend/src/infrastructure/database/repositories/iot_*.rs` and `energy_*.rs`
- PostgreSQL columns of type `DOUBLE PRECISION` for sensor readings and energy measurements

### Boundary cases — switch to `Decimal`

When an energy measurement **becomes monetary**, the conversion happens at the boundary:

```rust
// energy_campaign computes estimated savings in kWh (f64 OK)
let savings_kwh: f64 = current_consumption_kwh - new_consumption_kwh;

// Convert to monetary at the boundary — value goes to a copropriete bill
let savings_eur: Decimal = Decimal::from_f64_retain(savings_kwh)
    .ok_or(AppError::Validation("savings out of decimal range".into()))?
    * unit_price_per_kwh_eur; // unit_price_per_kwh_eur: Decimal
```

**Rule** : the type changes when the value enters accounting territory (invoice, charge, contribution).

### NOT allowed in IoT/energy modules

- Direct invoicing or charge computation in `f64` → use `Decimal`
- TVA application on energy bills → use `Decimal`
- Aggregating energy consumption across copropriétés for legal reporting (e.g., PEB / certificat performance énergétique) when the report has financial implications → use `Decimal`

## Consequences

### Positive

- **Performance** : IoT pipelines remain hardware-accelerated arithmetic
- **Library compatibility** : energy/IoT crates (numerical analysis, time-series) typically use `f64` — no friction
- **Pragmatic** : no migration burden on a domain where exactness is moot
- **Clear boundary** : monetary boundary explicit at conversion point, easy to review

### Negative

- **Boundary mistakes** : developers might accidentally use `f64` when a value crosses into monetary territory. Mitigated by code review (`rust-expert` and `code-reviewer` personas) and the static type system (`Decimal` is not `From<f64>` without explicit `from_f64_retain`)
- **Two coexisting numeric philosophies** : team must understand which domain uses which type. Mitigated by ADRs 0007/0008/0009 and persona memory files

### Neutral

- ADR 0007 grep enforcement allowlists `iot` and `energy` paths — non-monetary-by-default

## Alternatives Considered

### Use `Decimal` everywhere uniformly
Rejected. Forces software-emulated arithmetic on millions of IoT readings/day with no benefit for non-monetary domain. Estimated 5-10× slowdown on time-series aggregation. Misallocates engineering effort (migrating IoT code without business benefit).

### Use a `Measurement<T>` newtype that wraps `f64` to mark it explicitly non-monetary
Considered for future iteration. Would add type-level safety against accidental conversion to money. Not adopted in this ADR as it would require touching ~80 occurrences in energy/IoT modules — disproportionate for the safety gain. Could be revisited in a future ADR if boundary mistakes become recurrent.

### Keep all energy in `f64` but require all IoT in `Decimal`
Rejected. IoT and energy domains have similar characteristics (statistical, non-monetary, high volume). Splitting them creates an arbitrary boundary that doesn't match the underlying engineering reality.

## Enforcement

- **CI lint allowlist** : the grep filter `grep -rE '\bf64\b' backend/src/ | grep -v iot | grep -v energy` excludes IoT/energy paths from the f64-prohibition check
- **Code review** : when a PR introduces `f64` in a NEW module, `rust-expert` checks whether the module is monetary or measurement-domain and guides accordingly
- **Boundary annotations** : at every conversion point (energy → money), add a code comment `// Boundary: f64 measurement → Decimal monetary (cf. ADR 0009)` for reviewer clarity

## References

- ADR 0007 (forbids `f64` in money) : [`0007-decimal-vs-f64-for-money.md`](0007-decimal-vs-f64-for-money.md)
- ADR 0008 (DB) : [`0008-numeric-vs-double-precision-postgresql.md`](0008-numeric-vs-double-precision-postgresql.md)
- Audit : [`docs/audit/2026-04-30-f64-monetary-audit.md`](../audit/2026-04-30-f64-monetary-audit.md) §"Cluster 🟢 LOW"
- IEEE 754 sensor precision context : Garmin, Bosch sensor datasheets
- KoproGo : [#425](https://github.com/gilmry/koprogo/issues/425), [#433](https://github.com/gilmry/koprogo/issues/433)

🤖 ADR drafted by `rust-expert` persona — Tier 1 acceptance pending @gilmry sign-off.

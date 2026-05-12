# ADR 0007: Decimal (`rust_decimal`) for Monetary and Accounting Code

- **Status**: Accepted
- **Date**: 2026-04-30
- **Track**: Software / Finance
- **Authors**: `rust-expert` persona (cf. `.claude/agents/rust-expert.md`) + @gilmry sign-off
- **Related**: [ADR 0008](0008-numeric-vs-double-precision-postgresql.md) (DB), [ADR 0009](0009-iot-energy-keep-f64.md) (exception)

## Context

KoproGo is a Belgian copropriete management SaaS. The accounting subsystem must comply with **PCMN** (Plan Comptable Minimum Normalisé belge, Arrêté Royal du 12 juillet 2012). PCMN mandates **exactness at the centime level** for:

- Charges (factures fournisseurs, dépenses copropriété)
- Appels de fonds et contributions des propriétaires
- Écritures comptables (journal entries, double-entry: débit == crédit strict)
- État Daté (Art. 577 Code Civil belge — document légal pour vente immobilière)
- Rapports légaux annuels (Bilan, Compte de résultats)
- Taux de pénalité légal belge (8% — calcul en centimes)
- Devis (loi belge "3 quotes rule" pour travaux > 5000 €)

The audit of 2026-04-30 (cf. [`docs/audit/2026-04-30-f64-monetary-audit.md`](../audit/2026-04-30-f64-monetary-audit.md)) revealed **221 occurrences of `f64` in monetary/accounting code** spanning 19 files (domain entities, DTOs, use cases, repositories).

`f64` is IEEE 754 double-precision floating-point. It is **fundamentally unsuited for exact decimal arithmetic**:

- `0.1 + 0.2 != 0.3` — well-known IEEE 754 representation drift
- Cumulating thousands of operations (months × buildings × charges) introduces silent errors of cents per year
- Aggregate report (Bilan) imprecision risks legal liability vs commissaire aux comptes review
- `journal_entry` validation `débit == crédit` cannot be guaranteed with `f64` arithmetic

## Decision

**All monetary and accounting fields and computations in KoproGo MUST use `rust_decimal::Decimal`.**

Two acceptable representations:

1. **`rust_decimal::Decimal`** — preferred for new code. Native exact decimal type with bounded precision and no representation drift. Supports arithmetic, comparison, ordering, serialization.
2. **`i64` cents** — accepted for legacy code already using this pattern (e.g., `payment.rs::amount_cents`). Conversion to/from `Decimal` at API boundary via `Decimal::from(cents) / Decimal::from(100)`.

`f64` and `f32` are **forbidden** in the following paths:

- `backend/src/domain/entities/*.rs` for any field representing money, percentages affecting allocations (quotités, voting power), or rates (TVA, pénalité)
- `backend/src/application/dto/*.rs` for monetary fields (incl. budget, expense, contribution, call_for_funds, journal, quote, etat_date)
- `backend/src/application/use_cases/*.rs` for any module computing money, allocations, or financial reports (notably: `expense`, `budget`, `charge_distribution`, `journal_entry`, `financial_report`, `quote`, `etat_date`, `payment_reminder`)
- `backend/src/infrastructure/database/repositories/*.rs` for serialization of monetary columns

**Cargo.toml** — required features:

```toml
rust_decimal = { version = "1.36", features = ["serde-with-arbitrary-precision", "macros"] }
```

`serde-with-arbitrary-precision` ensures JSON serialization preserves exact decimal precision (uses string representation for values exceeding `f64`).

## Consequences

### Positive

- **PCMN compliance** : centime exactness guaranteed at type level
- **`débit == crédit` validation** in `journal_entry_use_cases` becomes provably correct
- **Audit trail integrity** : aggregate reports (Bilan, Compte de résultats) match individual transactions to the centime
- **Type safety** : compiler prevents accidental mixing of monetary and non-monetary numbers
- **`Decimal::from_str_exact("0.1") + Decimal::from_str_exact("0.2") == Decimal::from_str_exact("0.3")`** holds

### Negative

- **Migration effort** : 6-8 stories (M-L) on the existing 221 `f64` occurrences. Tracked in umbrella issue [#433](https://github.com/gilmry/koprogo/issues/433)
- **Slight perf overhead** vs `f64` arithmetic (decimal arithmetic is software-emulated, not hardware-accelerated). Estimated <5% on monetary hot paths — negligible vs DB latency
- **Library dependency** : `rust_decimal` adds compile time (already in `Cargo.toml`, no new dep)
- **JSON API impact** : numeric values ≥ 16 significant digits serialize as strings (per `arbitrary-precision`). Frontend (Astro/Svelte) must parse via `Decimal.js` or similar to avoid `Number()` rounding

### Neutral

- Database columns must use `NUMERIC(precision, scale)` instead of `DOUBLE PRECISION` — see [ADR 0008](0008-numeric-vs-double-precision-postgresql.md)
- Non-monetary domains (IoT, energy measurements) may keep `f64` — see [ADR 0009](0009-iot-energy-keep-f64.md)

## Alternatives Considered

### `f64` everywhere (status quo before this ADR)
Rejected. Fundamental incompatibility with PCMN exactness. Risk of legal liability.

### `i64` cents only (no `Decimal`)
Rejected. Doesn't handle percentages and rates cleanly (e.g., TVA 21.5%, taux pénalité 8.0%). Conversion arithmetic with cents is error-prone for ratios.

### `bigdecimal` crate (alternative to `rust_decimal`)
Rejected. `rust_decimal` is more idiomatic in the Rust ecosystem, has better `sqlx` integration (native `NUMERIC` mapping), and is faster for the typical monetary range (max 28 significant digits, sufficient for KoproGo).

### Custom `Money` newtype wrapping `i64` cents
Considered. Acceptable for legacy modules already using cents. New code prefers `Decimal` for flexibility (handles percentages without cent base assumptions).

## Implementation

Migration is tracked under umbrella issue [#433](https://github.com/gilmry/koprogo/issues/433), divided into 6-8 vertical stories (entity + DTO + use case + repo + SQL migration + tests, per story).

**Anti-pattern explicitly rejected** : migrating an entity to `Decimal` while leaving `as f64` / `f64::from(decimal)` conversions throughout dependent layers. This defeats the purpose (precision lost at every conversion). Each story migrates the **complete vertical** (entity → DTO → use case → handler → repo → migration → tests) atomically.

## Enforcement

- **Hook PostToolUse** : `posttool-warn-unwrap.sh` to be extended with `f64` detection in monetary paths (cf. issue [#425](https://github.com/gilmry/koprogo/issues/425) follow-up)
- **CI lint** : `grep -rE '\bf64\b' backend/src/domain/entities/ | grep -v iot | grep -v energy` must return **0 lines** post-migration
- **Code review** : `rust-expert` and `code-reviewer` personas reject any PR introducing `f64` in monetary contexts

## References

- Audit doc : [`docs/audit/2026-04-30-f64-monetary-audit.md`](../audit/2026-04-30-f64-monetary-audit.md)
- Memory rule : `project_no-f64-in-money.md` (system-level, non-versioned)
- Belgian PCMN : Arrêté Royal du 12 juillet 2012 portant exécution du Code des sociétés
- IEEE 754 representation issues : *"What Every Computer Scientist Should Know About Floating-Point Arithmetic"* — David Goldberg, ACM 1991
- `rust_decimal` documentation : https://docs.rs/rust_decimal/
- KoproGo issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#427](https://github.com/gilmry/koprogo/issues/427), [#430](https://github.com/gilmry/koprogo/issues/430), [#433](https://github.com/gilmry/koprogo/issues/433)

🤖 ADR drafted by `rust-expert` persona — Tier 1 acceptance pending @gilmry sign-off.

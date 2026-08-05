# ADR 0008: PostgreSQL `NUMERIC` for Monetary Columns (not `DOUBLE PRECISION`)

- **Status**: Accepted
- **Date**: 2026-04-30
- **Track**: Software / Database / Finance
- **Authors**: `rust-expert` persona + `platform-engineer` review + @gilmry sign-off
- **Related**: [ADR 0007](0007-decimal-vs-f64-for-money.md) (Rust), [ADR 0003](0003-postgresql-database.md) (DB choice)

## Context

The audit of 2026-04-30 found **9 SQL migrations** declaring monetary columns as `DOUBLE PRECISION`:

| Migration                                         | Tables impacted                                                         |
| ------------------------------------------------- | ----------------------------------------------------------------------- |
| `20240101000003_create_units.sql`                 | `units.area_m2`                                                         |
| `20240101000004_create_expenses.sql`              | `expenses.amount`, `amount_excl_vat`, `vat_amount`, etc. — **CRITICAL** |
| `20250127000000_refactor_owners_multitenancy.sql` | `unit_owners.percentage` (quotité copro)                                |
| `20251107120000_create_payment_reminders.sql`     | `payment_reminders.amount_due`, `penalty_amount`, etc.                  |
| `20251201000000_create_iot_readings.sql`          | `iot_readings.value` (acceptable — see ADR 0009)                        |
| `20251203000000_create_work_reports.sql`          | montants travaux                                                        |
| `20251203000001_create_technical_inspections.sql` | montants inspections                                                    |
| `20251204000000_create_energy_buying_groups.sql`  | consommations (acceptable — see ADR 0009)                               |
| `20260312000000_add_quorum_to_meetings.sql`       | quorum percentage                                                       |

PostgreSQL `DOUBLE PRECISION` is IEEE 754 binary64 — same drift issue as Rust `f64` (cf. [ADR 0007](0007-decimal-vs-f64-for-money.md)). Selecting a `DOUBLE PRECISION` column into a Rust `Decimal` via `sqlx` involves an **implicit `f64` round-trip** that loses precision regardless of the Rust type used.

PCMN compliance requires exactness end-to-end (DB column → repository → use case → DTO → API → frontend rendering).

## Decision

**Monetary columns in PostgreSQL MUST use `NUMERIC(precision, scale)`**, not `DOUBLE PRECISION`.

### Standard sizes

| Use case                                                       | Type             | Rationale                                                                                   |
| -------------------------------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------- |
| Montants en EUR (factures, paiements, budgets, contributions)  | `NUMERIC(15, 2)` | Up to 9,999,999,999,999.99 € — sufficient for any single transaction in copropriete context |
| Pourcentages (quotités, taux TVA, taux pénalité, voting power) | `NUMERIC(7, 4)`  | Up to 999.9999% — preserves 4 decimal digits (e.g., 21.5000%, 8.0000%)                      |
| Surfaces (m²)                                                  | `NUMERIC(10, 2)` | Up to 99,999,999.99 m² — sufficient for buildings                                           |
| Cents only (legacy `i64 cents` columns)                        | `BIGINT`         | Acceptable for `payment.amount_cents` already in this format                                |

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

---

## Amendment 2026-05-19 — f64 carve-outs & accounting-integrity policy (WP-A7)

- **Status of this amendment**: **Accepted** (validé @gilmry 2026-07-31,
  cf. audit `docs/agent-activity/2026-07-31-audit-433.md`).
- **Scope**: clarifies the **exhaustive, closed list** of places where
  `f64` (Rust) / `DOUBLE PRECISION` (SQL) remains acceptable _despite_ the
  Decimal rule, plus the `#526` expenses-positivity policy. Emerged from
  the #433 umbrella stories (WP-A1…A6).

### A. Decimal rule is the default; carve-outs are explicit and closed

Any monetary amount, quote-part, or value feeding a **legal threshold**
(quorum, majority, distribution) MUST be `Decimal` / `NUMERIC` end-to-end
(ADR-0007 + the decision above). The **only** authorised `f64` survivors:

| Site                                                  | Value                                                         | Why f64 is acceptable                                                                                                                                                                                         |
| ----------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `resolution.rs` `pour/contre/abstention_percentage()` | % d'affichage calculé depuis des **comptes entiers** de votes | **Présentation seule.** Jamais persisté, jamais comparé à un seuil légal. Le chemin légal quorum/majorité utilise les comptes entiers / `Decimal` directement — **aucun aller-retour `Decimal→f64→Decimal`**. |
| `vote.rs` proxy-validation ratio                      | ratio de quotités pour plafond de procurations                | Le **seuil** reste évalué en `Decimal` ; le `f64` éventuel est un indicateur dérivé non opposable. Invariant : la décision de rejet d'une procuration ne dépend que de comparaisons `Decimal`.                |
| `challenge.rs:401` progression %                      | `current_value as f64 / target_value as f64 * 100`            | **Score de jeu non-PCMN** (gamification). Jamais un montant, jamais légal. Analogue au carve-out IoT (ADR-0009).                                                                                              |
| `etat_date.unit_area`, `units.area_m2`                | surface m²                                                    | Mesure **physique**, pas un montant (ADR-0009 §physique).                                                                                                                                                     |
| IoT / énergie (`iot_readings.value`, conso)           | mesures capteur                                               | ADR-0009 (déjà acté).                                                                                                                                                                                         |

**Tout autre `f64`/`DOUBLE PRECISION` sur un montant ou une quotité est un
défaut** (cf. enforcement ci-dessous). Cette liste est **fermée** : ajouter
un carve-out exige un nouvel amendement signé.

### B. Ratio / %-display invariant (testable)

Pour chaque chemin légal (quorum Art. 3.87, majorités, répartition de
charges) : **assertion qu'aucune valeur de seuil ne transite par `f64`**.
Les `*_percentage(): f64` de `resolution.rs` sont tolérés _uniquement_
parce qu'ils ne sont consommés que par la sérialisation d'affichage —
vérifié par les tests de gouvernance (WP-A1, `bdd_governance`).

### C. #526 — `expenses.amount` CHECK (> 0) : politique

**Décision : conserver `CHECK (amount > 0)`** sur `expenses` (migration
`20260502000000`). Les annulations / avoirs / remboursements se modélisent
en **contre-écritures de journal** (PCMN — écriture inverse classe 6/7),
**pas** en relâchant la contrainte pour autoriser des montants ≤ 0.

Rationale : la positivité au niveau schéma est une défense en profondeur
PCMN ; une ligne de dépense ≤ 0 corromprait les agrégats (`SUM(amount)`,
dashboards, état daté). L'inversion comptable préserve l'auditabilité
(trace des deux écritures) là où un montant négatif l'effacerait. Confirmé
côté tests : `expenses_amount_check > 0` vert (#526 / WP-B3, commit
`086c953`).

### D. #339 — rotation de clé API

L'endpoint `POST /api-keys/{id}/rotate` est désormais **implémenté**
(WP-A7, branche `story/wbs-a7-adr8-rotate`) : désactivation immédiate de
l'ancienne clé + émission d'une remplaçante (métadonnées héritées),
isolation cross-org (404, pas de fuite d'existence), gate de rôle
SYNDIC/SUPERADMIN, 4-cat RED-first. **Aucun `501 Not Implemented` ne part
en bêta** (critère GO #429/#427).

### Enforcement (amendement)

- Le carve-out est **par site**, listé en §A. Un `code-reviewer` rejette
  tout nouveau `f64` monétaire/quotité hors de cette liste fermée.
- Toute extension de la liste = nouvel amendement ADR-0008 signé (Tier 1).

🤖 Amendment drafted by backend agent (Tier 2 **proposal**) — acceptance =
@gilmry merge. Traçé : `docs/agent-activity/2026-05-19-wbs-a7.md`.

## Amendment 2026-07-31 — extension du carve-out fermé (audit #433)

- **Status of this amendment**: **Accepted** (validé @gilmry 2026-07-31,
  cf. audit `docs/agent-activity/2026-07-31-audit-433.md`).
- **Scope**: l'audit de clôture de l'umbrella #433 a identifié deux sites
  `f64` résiduels non couverts par la liste fermée §A. Ni l'un ni l'autre
  n'est un montant ou une quotité opposable à un seuil légal — extension
  de la liste plutôt que migration Decimal.

| Site                                                                                           | Valeur                                              | Pourquoi `f64` est acceptable                                                                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `quote.rs` (`total_score`, `price_score`, `delay_score`, `warranty_score`, `reputation_score`) | Score de classement 0-100 pour comparaison de devis | **Heuristique de tri, jamais un montant ni une quotité.** N'alimente aucun calcul PCMN ni seuil légal — sert uniquement au tri d'affichage des devis entre eux.                                                                               |
| `age_request.rs::calculate_progress_percentage`                                                | % de progression d'une pétition AGE (demande 1/5)   | Calcul interne en `Decimal` (`Decimal::from(...)`, comparaisons exactes) ; conversion `f64` uniquement sur la **valeur de retour affichée**, jamais réutilisée pour une comparaison de seuil ultérieure. Même pattern que `resolution.rs` §A. |

Cette liste étendue reste **fermée** : toute nouvelle exception nécessite
un nouvel amendement signé, comme ci-dessus.

🤖 Amendment drafted by backend agent (Tier 2 **proposal**) — acceptance =
@gilmry en session. Traçé : `docs/agent-activity/2026-07-31-audit-433.md`.

---
persona: rust-expert
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `rust-expert`

## Anti-patterns recurrents trouvés (audit 2026-04-29)

### 🔴 1. f64 dans le code monétaire/comptable

- **Détection initiale** : findings rapportés par utilisateur le 2026-04-29 (`"on a trouvé aussi des f64 dans le code alors que pour la comptabilité il faut pas"`).
- **Mémoire associée** : `project_no-f64-in-money.md`.
- **Action** : audit complet `grep -rn 'f64\|f32' backend/src/{domain,application}/` à effectuer en S1.
- **Cible** : 0 occurrence f64/f32 dans contextes monétaires/comptables. `rust_decimal::Decimal` partout.

### 🔴 2. `Result<_, String>` partout (~80 % des ports)

- **Détection** : audit 2026-04-29 (cf. #425).
- **Crates disponibles** : `thiserror` 2.0 et `anyhow` 1.0 importés mais utilisés sur 3 ports seulement.
- **Cible** : enum `AppError` dans `backend/src/application/error.rs` (à créer), avec variants Validation/NotFound/Database/Unauthorized/etc.
- **Pattern** : `impl From<sqlx::Error> for AppError` + `impl ResponseError for AppError` (mapping HTTP).

### 🔴 3. 1967 `.unwrap()` / `.expect(` en code production

- **Détection** : audit 2026-04-29.
- **Hook actif** : PostToolUse warn-unwrap (cf. `.claude/hooks/posttool-warn-unwrap.sh`).
- **Cible** : 0 hors tests. Remplacer par `?` + AppError (variants typés).
- **Stratégie** : migration progressive module par module (auth d'abord, puis building, expense, etc.).

### 🟠 4. Entités à `pub` fields (audit a noté `expense.rs` 1245 LOC, 26 `pub`)

- **Problème** : invariants ignorables après construction.
- **Cible** : encapsulation via getters + invariants enforcés en `::new()` et `update_*()`.
- **Pattern Newtype** : `Email(String)`, `Money(Decimal)`, `Percentage(Decimal)` pour les types métier.

### 🟠 5. Fichiers > 1000 LOC

8 fichiers : `seed.rs` 4152, `mcp_sse_handlers.rs` 2010, `local_exchange_use_cases.rs` 1530, `gamification_use_cases.rs` 1427, `expense.rs` 1245, `payment_use_cases.rs` 1179, `shared_object_use_cases.rs` 1111, `convocation_use_cases.rs` 1098.

- **Cible** : split par contexte transactionnel ou par responsabilité.

## Conventions Rust acceptées

- **Edition** : 2021 (vérifier `Cargo.toml`).
- **Erreurs** : `Result<T, AppError>` typé via `thiserror`.
- **Numérique monétaire/comptable** : `rust_decimal::Decimal` exclusivement.
- **Async** : `tokio` 1.x, `async-trait` quand nécessaire pour traits.
- **Tests** : `#[cfg(test)]` modules + `#[tokio::test]` async + `cucumber` BDD.
- **Modularité** : séparation domain / application / infrastructure (hexagonal).

## Décisions en attente

- ADR : structure `AppError` enum exacte (variants à inclure).
- ADR : Newtype patterns pour Email, Money, Percentage, OwnerId.
- RFC : politique encapsulation entités domain (getters/setters vs `pub`).
- RFC : split de `seed.rs` (4152 LOC).

## Lessons learned

- L'agent IA optimise pour les chemins de moindre résistance (`f64` est le default Rust pour les literals décimaux, donc tentation forte). **La discipline doit être instrumentée** (hook + memory + ADR), pas juste demandée.
- **Tooling via docker compose** (cf. memory `feedback_use-docker-compose-for-tooling.md`) : avant toute review/PR backend, lancer `docker compose run --rm backend bash -c "SQLX_OFFLINE=true cargo check --lib"` puis `cargo clippy --lib -- -D warnings` puis `cargo test --lib`. Le shell hôte n'a pas cargo en standalone — toujours via compose.

## Liens

- [`.claude/agents/rust-expert.md`](rust-expert.md)
- Memory : `project_no-f64-in-money.md`
- Issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#427](https://github.com/gilmry/koprogo/issues/427)
- Lectures : *Rust for Rustaceans* (Jon Gjengset), *Programming Rust* (Blandy/Orendorff/Tindall), [Effective Rust](https://www.lurklurk.org/effective-rust/).

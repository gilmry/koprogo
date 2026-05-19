# Issue draft — f64 sur colonnes NUMERIC : panics latents + violation PCMN

**Status** : draft pour review humain avant `gh issue create` (Tier 1).

---

## Titre proposé

`bug(backend): f64 sur colonnes NUMERIC — panic /stats/syndic/urgent-tasks + 32 occurrences à migrer vers rust_decimal::Decimal`

## Labels

`bug`, `backend`, `security`, `tech-debt`, `priority:high`

## Body

### Constat

1. **Reproductible 100 %** : `GET /api/v1/stats/syndic/urgent-tasks` retourne 502 Bad Gateway dès qu'au moins une ligne `expenses` a `payment_status='overdue'` dans la DB seed dev. Le worker Actix crash, Traefik propage 502.

   Log :
   ```
   thread 'actix-rt|system:0|arbiter:0' panicked at 
   src/infrastructure/database/repositories/stats_repository_impl.rs:364:39:
   called `Result::unwrap()` on an `Err` value: ColumnDecode { 
     index: "amount", 
     source: "mismatched types; Rust type `f64` (as SQL type `FLOAT8`) 
              is not compatible with SQL type `NUMERIC`" }
   ```

2. **Trois panics latents identiques** (mêmes patterns `let amount: f64 = row.get("...")`) :
   - [stats_repository_impl.rs:364](backend/src/infrastructure/database/repositories/stats_repository_impl.rs#L364) — `expenses.amount`
   - [budget_repository_impl.rs:439](backend/src/infrastructure/database/repositories/budget_repository_impl.rs#L439) — `budget_lines.total_amount`
   - [payment_reminder_repository_impl.rs:492](backend/src/infrastructure/database/repositories/payment_reminder_repository_impl.rs#L492) — `payments.amount`

3. **30 autres occurrences `f64`** dans `backend/src/infrastructure/database/repositories/` réparties sur 11 fichiers. À auditer une par une — celles sur colonnes monétaires (`amount`, `total`, `balance`, `tantieme_share`, etc.) sont également non-conformes.

### Cause racine

Triple violation des règles projet :

1. **`f64` sur monnaie** — viole [mémoire `project_no-f64-in-money.md`](#) : « All monetary/accounting code MUST use rust_decimal::Decimal, never f64; IEEE 754 rounding incompatible with Belgian PCMN exactness ». Les colonnes DB sont déjà en `NUMERIC` (correct) mais le code Rust décode en `f64`.

2. **`Row::get()` panic-on-error** — équivalent `.unwrap()`, interdit par [CRITICAL.md §4](.claude/rules/CRITICAL.md). Doit être `Row::try_get::<Decimal, _>("amount")?` avec propagation `AppError`.

3. **Absence de tests `@negative`** — aucun test ne couvrait le cas « seed contient un overdue » → panic en silence jusqu'à l'observation manuelle. Viole [CRITICAL.md §3 / mémoire `feedback_tdd-bdd-four-categories.md`].

### Recette

Étapes Maury :

1. **Brief + audit complet** (cette issue) — inventaire exhaustif des 33 occurrences `f64`, classification :
   - (a) **monétaire** → migration Decimal obligatoire
   - (b) **ratio/coefficient non monétaire** (ex: `coverage_ratio`) → peut rester f64 si non-critique pour la compta
2. **Stories Maury splittées par repository** (sera détaillé après audit, ≈ 6-11 stories suivant grouping)
3. **Tests RED-first 4 catégories par endpoint touché** :
   - `@negative` : seed avec overdue → ancien code retourne 502 (rouge), nouveau retourne 200 avec montant exact
   - `@happy` : montant standard (123.45 €) round-trip Decimal sans perte
   - `@edge` : montant avec 4 décimales (PCMN), montant 0.00, montant max NUMERIC(15,4)
   - `@security` : aucun changement comportement RBAC sur l'endpoint
4. **Migration code** :
   ```rust
   // AVANT
   let amount: f64 = expense.get("amount");
   format!("Charge en retard - {:.2}€", amount)
   
   // APRÈS
   let amount: rust_decimal::Decimal = expense.try_get("amount")?;
   format!("Charge en retard - {}€", amount.round_dp(2))
   ```
   + Signatures repo : `Result<_, AppError>` (déjà OK pour ces 3, 0 occurrence `Result<_, String>` dans `repositories/`).

### Critères d'acceptation

- [ ] `grep -rn "let.*: f64 = .*\.get(" backend/src/` → 0 résultat sur colonnes monétaires
- [ ] Tous les 3 panics latents identifiés convertis (stats, budget, payment_reminder)
- [ ] Tests `@happy` + `@edge` + `@security` + `@negative` verts sur chaque endpoint touché (sortie `cargo test --tests`)
- [ ] Scénario BDD reproductible : avant fix → 502 ; après fix → 200 avec montant correct
- [ ] Inventaire des 30 autres `f64` classé monétaire vs non-monétaire et tracké en sous-issues / stories si nécessaire
- [ ] Mémoire `project_no-f64-in-money.md` re-référencée dans le pre-commit hook ou doc CONTRIBUTING (suivi à part)

### Hors scope (follow-ups)

- Pre-commit hook qui interdit `let.*: f64 = .*\.get\(.*amount` (à proposer dans #425 garde-fous)
- Audit côté DTO REST (les amounts sont-ils sérialisés en string ou float dans les réponses JSON ?) — risque de perte précision côté frontend
- Frontend Svelte : `parseFloat()` sur amounts ? À auditer parallèlement

### Refs

- [CRITICAL.md §4](.claude/rules/CRITICAL.md) — pas d'`unwrap()`, `Result<E>` typé
- Mémoire `project_no-f64-in-money.md`
- Mémoire `feedback_tdd-bdd-four-categories.md`
- Mémoire `feedback_audit-to-issue-first.md`
- Issue #427 (validation TDD/BDD + Cowork)

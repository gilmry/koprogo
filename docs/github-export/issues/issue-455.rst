==============================================================
Issue #455: story: EXP-003-complete — finir le Decimal cascade
==============================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:high rust,finance
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/455>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # EXP-003 Decimal cascade incomplet — colonnes monétaires FLOAT8 + casts SQL FLOAT8 résiduels
   
   ## Contexte
   
   EXP-003 (PR #437/#445) a migré les **entités Rust** vers `Decimal` pour toute la comptabilité (Expense, JournalEntry, CallForFunds, OwnerContribution, etc.). PR #438 a migré `unit_owners.ownership_percentage` DOUBLE PRECISION → NUMERIC(6,5).
   
   **Mais le cascade DB n'est pas complet** — révélé par BDD CI sur PR #450 (chore/ci-gitflow-alignment) qui fait enfin tourner `ci.yml` sur `chore/**`.
   
   ## Constat (BDD failures)
   
   Run [#25176804715](https://github.com/gilmry/koprogo/actions/runs/25176804715) — 30+ panics avec :
   
   ```
   called `Result::unwrap()` on an `Err` value:
     ColumnDecode { index: "\"amount\"",
       source: "mismatched types; Rust type `rust_decimal::decimal::Decimal` (as SQL type `NUMERIC`)
                is not compatible with SQL type `FLOAT8`" }
   ```
   
   Et idem sur `total_amount`. Causes identifiées :
   
   ### Cause 1 — Colonne FLOAT8 en DB, entité Decimal côté Rust
   
   **`expenses.amount`** est `DOUBLE PRECISION` ([migration 20240101000004_create_expenses.sql:10](https://github.com/gilmry/koprogo/blob/feature/dev/backend/migrations/20240101000004_create_expenses.sql#L10)) mais `Expense.amount: Decimal` ([entity:57](https://github.com/gilmry/koprogo/blob/feature/dev/backend/src/domain/entities/expense.rs#L57)).
   
   → `row.get("amount")` essaie de décoder FLOAT8 en Decimal → panic.
   
   ### Cause 2 — Cast SQL `::FLOAT8` résiduel sur colonne déjà NUMERIC
   
   **`call_for_funds.total_amount`** est déjà `DECIMAL(10,2)` ([migration 20251111015338:32](https://github.com/gilmry/koprogo/blob/feature/dev/backend/migrations/20251111015338_create_call_for_funds.sql#L32)) — bonne nouvelle.
   
   MAIS le repository fait `total_amount::FLOAT8 AS total_amount` dans 5 SELECT statements ([call_for_funds_repository_impl.rs:76,96,120,203,...](https://github.com/gilmry/koprogo/blob/feature/dev/backend/src/infrastructure/database/repositories/call_for_funds_repository_impl.rs)) — le cast force la colonne à FLOAT8 au moment du SELECT.
   
   → même panique côté Decimal entity.
   
   ### Cause 3 — DTO ExpenseFilters bind f64 sur colonne Decimal
   
   [`filters.rs:29-30`](https://github.com/gilmry/koprogo/blob/feature/dev/backend/src/application/dto/filters.rs) :
   ```rust
   pub min_amount: Option<f64>,
   pub max_amount: Option<f64>,
   ```
   
   Bind dans [expense_repository_impl.rs:319,322,376,379](https://github.com/gilmry/koprogo/blob/feature/dev/backend/src/infrastructure/database/repositories/expense_repository_impl.rs). Si on migre `expenses.amount` → NUMERIC, ces bindings paniqueront aussi.
   
   ## Recette proposée (ordre)
   
   ### Phase 1 — DB schema migration
   
   Migration unique `20260502000000_complete_expense_decimal_cascade.sql` :
   
   ```sql
   -- 1. expenses.amount: DOUBLE PRECISION → NUMERIC(12,2)
   --    Précision alignée sur journal_entry_lines.{debit,credit} (même chart of accounts)
   ALTER TABLE expenses ALTER COLUMN amount TYPE NUMERIC(12,2);
   
   -- 2. payment_reminders.{amount_owed, penalty_amount, total_amount}
   --    NOTE: l'entité PaymentReminder est encore f64 → garder FLOAT8 pour cohérence,
   --    OU migrer entité ET schéma. Décision : migrer en même temps (pénalités = monétaire).
   -- ALTER TABLE payment_reminders ALTER COLUMN amount_owed TYPE NUMERIC(12,2);
   -- ALTER TABLE payment_reminders ALTER COLUMN penalty_amount TYPE NUMERIC(12,2);
   -- ALTER TABLE payment_reminders ALTER COLUMN total_amount TYPE NUMERIC(12,2);
   ```
   
   Vérifier dépendances (`SELECT FROM information_schema.view_column_usage WHERE column_name='amount'`) avant ALTER — DROP/CREATE views/triggers qui réfèrent ces colonnes.
   
   ### Phase 2 — Code Rust
   
   1. `filters.rs`: `Option<f64>` → `Option<Decimal>` pour `min_amount`/`max_amount` (3 fields à modifier)
   2. `expense_repository_impl.rs`: aucun changement (déjà `row.get("amount")` direct, OK une fois colonne NUMERIC)
   3. `call_for_funds_repository_impl.rs`: **supprimer** les 5 occurrences de `total_amount::FLOAT8 AS total_amount` → simplement `total_amount`
   4. `payment_reminder.rs` entity: `f64` → `Decimal` pour amount_owed, penalty_amount, total_amount (décision éditoriale : pénalités IS monétaire, donc Decimal — cohérent ADR-0007)
   5. `payment_reminder_repository_impl.rs`: supprimer 6 occurrences de `*::FLOAT8 AS *` casts
   
   ### Phase 3 — Tests BDD
   
   `bdd_financial.rs` peut déjà accepter Decimal (cf. PR #437 mass-migration). Vérifier qu'aucun test ne hardcode encore un f64 binding sur les nouveaux Decimal fields.
   
   ### Phase 4 — Vérification
   
   ```bash
   docker compose exec -T backend bash -c "SQLX_OFFLINE=true cargo test --test bdd_financial 2>&1 | tail -30"
   # Attendu: 0 panic ColumnDecode
   ```
   
   ## Hors scope (pour suivi)
   
   - `work_report.cost: f64` ([work_report.rs:27](https://github.com/gilmry/koprogo/blob/feature/dev/backend/src/domain/entities/work_report.rs#L27)) — `cost` est monétaire mais l'entité est encore f64. À migrer en EXP-005 (post EXP-003-complete).
   - `technical_inspection.cost: Option<f64>` — idem.
   - `units.quota: f64` + `surface_area: f64` — quota est un % de copropriété ; per ADR-0009, peut rester f64 si pas utilisé en arithmétique comptable. Confirmer.
   - `units.amount` ne semble pas exister mais BDD log mentionne aussi `amount` génériquement — investiguer si autres tables touchées.
   
   ## Acceptance criteria
   
   - [ ] Migration `20260502000000_*` mergée
   - [ ] `cargo test --test bdd_financial` passe en CI (0 ColumnDecode panic)
   - [ ] `cargo test --test bdd_governance --test bdd_operations --test bdd_community` passent aussi
   - [ ] `cargo test --lib` toujours green (pas de régression sur les 1213 tests EXP-003)
   - [ ] PostgreSQL ALTER ne perd aucune donnée — vérifier sur dump test
   - [ ] Audit `git grep -nE "amount.*f64|f64.*amount"` ne retourne plus de fields entity monétaires
   
   ## Tier 1 / Tier 2
   
   - **Tier 2** (cette story) : rédaction migration SQL + édition Rust + tests
   - **Tier 1** : merge story branche → feature/dev (humain)
   
   ## Liens
   
   - Surfacé par : PR #450 (chore/ci-gitflow-alignment)
   - Dépend de : ADR-0007 (Decimal vs f64), ADR-0008 (NUMERIC vs DOUBLE), ADR-0009 (IoT keep f64)
   - Bloque : déploiement GitOps multi-topologie (PR-A/B/C/E) jusqu'à BDD verte

.. raw:: html

   </div>


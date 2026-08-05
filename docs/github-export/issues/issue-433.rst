==========================================================================================
Issue #433: Umbrella — Migration f64 → Decimal monétaire/comptable (PCMN belge compliance)
==========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:high rust,finance legal-compliance
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/433>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Suite à l'audit f64 monétaire livré dans EXP-002 (cf. [`docs/audit/2026-04-30-f64-monetary-audit.md`](https://github.com/gilmry/koprogo/blob/feature/dev/docs/audit/2026-04-30-f64-monetary-audit.md)) :
   
   - **19 fichiers** monétaires/comptables utilisent `f64` (incompatible PCMN belge AR 12/07/2012)
   - **~221 occurrences** à migrer vers `rust_decimal::Decimal`
   - **9 migrations SQL** `DOUBLE PRECISION` à passer en `NUMERIC`
   - **payment.rs** déjà clean (i64 cents) — pattern à suivre ailleurs
   
   Risques concrets identifiés :
   - 🔴 `journal_entry` : validation `débit == crédit` cassée silencieusement par IEEE 754
   - 🔴 `charge_distribution` : somme allocations peut ne pas atteindre 100% exactement
   - 🔴 `etat_date` : document légal vente immobilière (Art. 577 CC) non-exact
   - 🔴 `payment_reminder` : taux pénalité légal belge 8% imprécis
   - 🔴 `financial_report` (Bilan + Compte de résultats) : rapport légal annuel imprécis
   
   Cette **umbrella issue** trace les stories de migration découpées par cluster vertical (entité + DTO + use_case + repo + migration SQL + tests, par story). Chaque story est M-L (couvre l'ensemble vertical).
   
   ## Stories follow-up
   
   ### Backend domain entities monétaires
   
   - [ ] **EXP-003** — Migrate `expense.rs` (8 f64) + `invoice_line_item.rs` (12 f64) + DTO + use_cases + repo + migration SQL `20260430000000_alter_expense_amounts_to_numeric.sql` + tests 4-cat (#427)
   - [ ] **EXP-004** — Migrate `budget.rs` (8 f64) + `budget_dto.rs` (23 f64) + `budget_repository.rs` (15 f64) + use_cases + migration SQL + tests 4-cat sur calcul variance budget vs réel
   - [ ] **EXP-005** — Migrate `charge_distribution.rs` (10 f64) + use_cases (8 f64) + migration SQL + **`@security` test critique : somme allocations doit = 100% exactement**
   - [ ] **EXP-006 (PRIORITÉ MAX)** — Migrate `journal_entry.rs` (11 f64) + use_cases (8 f64) + repository_impl (19 f64) + migration SQL + **`@negative` test critique : validation débit==crédit ne tolère pas l'imprécision**
   - [ ] **EXP-007** — Migrate `quote.rs` (10 f64) + `etat_date.rs` (17 f64) + DTO etat_date (14 f64) + migration SQL + tests 4-cat
   - [ ] **EXP-008** — Migrate `owner_contribution.rs` (2 f64) + `call_for_funds.rs` (2 f64) + DTOs + migration SQL + tests 4-cat (`@security` : refus contribution > montant call)
   
   ### Use cases financiers
   
   - [ ] **FIN-001** — Migrate `financial_report_use_cases.rs` (23 f64) — calculs Bilan + Compte de résultats (rapport légal). Tests 4-cat exhaustifs.
   
   ### Pourcentages (priorité MEDIUM)
   
   - [ ] **PCT-001** — Migrate `resolution.rs` + `resolution_use_cases.rs` (32 f64) + `age_request.rs` (8) + `ag_session.rs` + DTO (15) + `payment_reminder.rs` (7 — taux pénalité belge légal). NB : `payment_reminder` taux 8% **est** un calcul de centimes, donc plus que medium.
   
   ### ADRs à créer (en début d'EXP-003 ou story dédiée ADR-001)
   
   - [ ] `docs/adr/0001-decimal-vs-f64-for-money.md` — décision formelle, citation PCMN AR 12/07/2012, exemples drift, configuration `Cargo.toml` (rust_decimal features `serde-with-arbitrary-precision`).
   - [ ] `docs/adr/0002-numeric-vs-double-precision-postgresql.md` — décision côté DB, type `NUMERIC(15,2)` pour montants, `NUMERIC(5,4)` pour pourcentages.
   - [ ] `docs/adr/0003-iot-energy-keep-f64.md` — justification que les domaines non-comptables (énergie, IoT, mesures physiques) peuvent garder `f64`.
   
   ### Hooks / CI à activer post-migration
   
   - [ ] Custom clippy lint : refuse `f64` dans `backend/src/domain/entities/*.rs` (excepté energy/IoT)
   - [ ] CI step : `grep -rE '\bf64\b' backend/src/domain/entities/ | grep -v iot | grep -v energy` doit retourner **0 ligne**
   - [ ] CI step similaire pour `application/use_cases/*finance*` `*charge*` `*budget*` `*expense*` `*journal*`
   
   ## Critères de succès global
   
   - [ ] 0 occurrence `f64`/`f32` dans `backend/src/domain/entities/*.rs` (sauf énergie/IoT justifié par ADR)
   - [ ] 0 occurrence dans `application/use_cases/*` financiers/comptables
   - [ ] 0 colonne `DOUBLE PRECISION` dans migrations pour montants/quotités
   - [ ] Tests 4-cat (#427) passent par module migré
   - [ ] ADR 0001/0002/0003 publiés et référencés dans le code
   - [ ] CI hook anti-f64 actif sur les paths sensibles
   
   ## Effort estimé global
   
   ~6-8 stories M-L (~2 semaines de travail backend + tests + migrations SQL + ADRs).
   
   ## Personas de référence (matrice #428)
   
   - 🤖 `rust-expert` (lead) — supervise la migration, review les PRs Rust
   - 🤖 `code-reviewer` — cohérence cross-cutting (DTO ↔ entité ↔ migration ↔ tests)
   - 🤖 `platform-engineer` — review migrations SQL `ALTER TABLE`
   - 🤖 `security-officer` — impact PCMN compliance + revue tests `@security`
   - 🤖 `csi-analyst` — alimente CSI report mensuel (tech debt visible → trend décroissant)
   - 👤 `@gilmry` (humain) — sign-off Tier 1 sur chaque PR
   
   ## Liens
   
   - Audit doc : [`docs/audit/2026-04-30-f64-monetary-audit.md`](https://github.com/gilmry/koprogo/blob/story/EXP-002/docs/audit/2026-04-30-f64-monetary-audit.md)
   - Story EXP-002 (audit-only) : sera mergée via PR à venir
   - Mémoire règle : `project_no-f64-in-money.md` (system-level memory, non-versionnée mais directive non-négociable)
   - Méthode : Maury v1.1 (#428) — discipline audit→propose→review→execute
   - Validation discipline : #427 (4-cat tests obligatoires sur chaque story de migration)
   - Pattern proof similaire : #431 (AUTH-001 a établi `AppError`, EXP-* établira `Decimal`)
   
   ---
   
   🤖 Issue umbrella créée par persona `rust-expert` (Claude) Tier 2 — logué.

.. raw:: html

   </div>


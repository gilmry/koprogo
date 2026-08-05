==============================================================================================
Issue #443: BDD-MIGRATION-001: Finalize Decimal cascade in BDD/E2E tests (~50 errors residual)
==============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:medium rust,finance
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-04-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/443>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Suite à #437 (EXP-003 cascade Decimal) et #442 (#438 SQL migration unit_owners NUMERIC), le périmètre `cargo check --lib` est clean (0 erreur, 0 warning) avec 1213 tests `--lib` verts. **Mais `cargo check --tests` montre encore ~50 erreurs résiduelles** dans :
   
   - `tests/bdd_financial.rs` — table-driven Cucumber steps avec literals f64 dans helpers + scenarios
   - `tests/e2e_call_for_funds.rs` — call_for_funds + create_contribution
   - `tests/e2e_resolutions.rs` — partiellement migré dans #442
   - `tests/e2e_owner_contributions.rs` — quota + amount
   - `tests/e2e_payments.rs` — amount
   
   ## Cause
   
   EXP-003 #437 a vérifié `cargo test --lib` (1213 verts) mais pas
   `cargo test --tests` (intégration + BDD). Le scope EXP-003 était déjà
   L+ (33 fichiers) et la décision honnête a été d'arrêter au lib.
   SQL-MIGRATION-001 #442 a migré une partie significative des tests
   (integration_unit_owner.rs + signatures bdd_financial + e2e_*) mais
   pas tous.
   
   Pré-existant — pas une régression #442 ni #437. Le `cargo test --lib`
   green publié dans EXP-003 est exact. Le manquement est sur la
   couverture `--tests`.
   
   ## Recette proposée
   
   ### Cluster 1 : `tests/bdd_financial.rs` (~7000 lignes)
   
   Migrer les literals f64 dans :
   - Cucumber tables (params `amount: f64`, `quota: f64`)
   - Helpers (`given_create_expense`, `given_call_of_amount_sent`, etc.)
   - Mock SQL inserts utilisant des literals `100.0` etc.
   - Assertions monétaires `(total - 1000.0).abs() < 0.01` →
     `total == dec!(1000)`
   
   Pattern : signatures déjà migrées dans #442 (12 helpers). Reste les
   **call sites** des helpers + literal SQL inserts dans `r#"INSERT..."#`
   strings (ces derniers OK car SQL parse les strings).
   
   ### Cluster 2 : `tests/e2e_*.rs` (8 fichiers)
   
   Sites de `add_owner_to_unit`, `create_contribution`, `create_call_for_funds`
   avec literals f64. Pattern simple : `1.0` → `dec!(1)`, `0.5` → `dec!(0.5)`,
   etc.
   
   ### Cluster 3 : `tests/integration_*.rs`
   
   `integration_unit_owner.rs` complet dans #442. Vérifier les autres
   integration tests (integration_expense, integration_payment, etc.)
   n'ont pas de drift.
   
   ## Critères de succès
   
   - [ ] `cargo check --tests` : 0 erreur
   - [ ] `cargo test --tests` : tous les tests verts ou explicitement skip
   - [ ] `cargo test --test bdd` (Cucumber) : 0 régression sur scenarios existants
   - [ ] BDD report HTML généré et review humaine confirme parité fonctionnelle
   - [ ] PR commentaire CSI : nombre de literals f64 migrés vs total trouvé
     (mesure pour CSI report mensuel)
   
   ## Effort estimé
   
   **M-L** (4-8h) — purement mécanique, pattern Decimal déjà en place,
   mais volume sans précision (tests BDD = 7000+ lignes).
   
   ## Anti-pattern à éviter
   
   Ne PAS migrer en force-push sur la branche test : risque de masquer un
   bug réel de scenario. Préférer commits incrémentaux par cluster + cargo
   test à chaque étape.
   
   ## Liens
   
   - Pré-existant à : #437 (EXP-003 cascade)
   - Découverte dans : #442 (#438 SQL-MIGRATION-001)
   - Pattern source : EXP-003 commit `1de6c2e` (lib tests migration)
   - ADR : `docs/adr/0007-decimal-vs-f64-for-money.md` (#435)
   - Umbrella : #433
   
   🤖 Auto-drafted by Claude Opus 4.7 sous Claude Desktop primary runtime.

.. raw:: html

   </div>


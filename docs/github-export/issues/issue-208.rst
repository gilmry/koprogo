======================================================================================
Issue #208: feat(tests): BDD step definitions for 24 new feature files (279 scenarios)
======================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: priority:high,release:v0.5.0 testing,bdd
:Assignees: Unassigned
:Created: 2026-02-26
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/208>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   24 fichiers `.feature` (279 scenarios Gherkin) ont été écrits mais n'ont **aucune step definition** dans les fichiers BDD. Cucumber les marque comme "skipped".
   
   ## Architecture
   
   4 nouveaux fichiers BDD groupés par domaine (approche DDD) :
   
   | Fichier | Bounded Contexts | Features | Scenarios |
   |---------|-----------------|----------|-----------|
   | `bdd_financial.rs` | BC2+BC10+BC12 | payments, payment_methods, journal_entries, call_for_funds, owner_contributions, charge_distribution, dashboard | 74 |
   | `bdd_governance.rs` | BC3+BC9+BC11+BC13 | resolutions, convocations, quotes, organizations, two_factor, public_syndic | 74 |
   | `bdd_operations.rs` | BC4+BC6+BC8 | tickets, notifications, work_reports, technical_inspections, iot, energy_campaigns | 83 |
   | `bdd_community.rs` | BC5 | notices, skills, shared_objects, resource_bookings, gamification | 72 |
   
   ## Prérequis
   
   - [ ] Ajouter 4 `[[test]]` entries dans `Cargo.toml` (harness = false)
   - [ ] Scaffolding World struct + setup_database() pour chaque fichier
   - [ ] Mettre à jour CI (`cargo test --test 'bdd*'`) et Makefile
   
   ## Tiers d'implémentation
   
   ### Tier 1 - Core (120 scenarios, ~40h) - P0
   - [ ] payments + payment_methods (28 sc.) → `bdd_financial.rs`
   - [ ] resolutions (14 sc.) → `bdd_governance.rs`
   - [ ] tickets (17 sc.) → `bdd_operations.rs`
   - [ ] notifications (14 sc.) → `bdd_operations.rs`
   - [ ] convocations (13 sc.) → `bdd_governance.rs`
   - [ ] quotes (13 sc.) → `bdd_governance.rs`
   - [ ] two_factor (12 sc.) → `bdd_governance.rs`
   - [ ] organizations (8 sc.) → `bdd_governance.rs`
   
   ### Tier 2 - Community (72 scenarios, ~23h) - P1
   - [ ] notices (15 sc.) → `bdd_community.rs`
   - [ ] skills (13 sc.) → `bdd_community.rs`
   - [ ] shared_objects (15 sc.) → `bdd_community.rs`
   - [ ] resource_bookings (16 sc.) → `bdd_community.rs`
   - [ ] gamification (13 sc.) → `bdd_community.rs`
   
   ### Tier 3 - Remaining (87 scenarios, ~27h) - P2
   - [ ] energy_campaigns (14 sc.) → `bdd_operations.rs`
   - [ ] iot (12 sc.) → `bdd_operations.rs`
   - [ ] work_reports (10 sc.) → `bdd_operations.rs`
   - [ ] technical_inspections (12 sc.) → `bdd_operations.rs`
   - [ ] journal_entries (8 sc.) → `bdd_financial.rs`
   - [ ] call_for_funds (10 sc.) → `bdd_financial.rs`
   - [ ] owner_contributions (8 sc.) → `bdd_financial.rs`
   - [ ] charge_distribution (6 sc.) → `bdd_financial.rs`
   - [ ] dashboard (4 sc.) → `bdd_financial.rs`
   - [ ] public_syndic (4 sc.) → `bdd_governance.rs`
   
   ## Pattern de référence
   
   Suivre le pattern de `bdd.rs` (lignes 339-383) : World struct → setup_database() → given/when/then macros → main() avec cucumber runner.
   
   Parent: #207

.. raw:: html

   </div>


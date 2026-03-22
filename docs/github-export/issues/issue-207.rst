=================================================================
Issue #207: Release 0.5.0 - Test Pyramid & Documentation Umbrella
=================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: documentation,priority:critical release:v0.5.0,testing
:Assignees: Unassigned
:Created: 2026-02-26
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/207>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Objectif
   
   Première release formelle de KoproGo (v0.5.0) avec couverture test pyramidale élevée, documentation complète et changelog structuré.
   
   ## Scope
   
   **Version** : 0.5.0
   **Effort estimé** : ~134h (10-12 semaines solo)
   **WBS** : `docs/WBS_RELEASE_0_5_0.rst` + `docs/WBS_PROJET_COMPLET.rst`
   
   ## Checklist
   
   ### Tests BDD (81h)
   - [ ] Infrastructure : 4 fichiers BDD groupés par domaine (`bdd_financial.rs`, `bdd_governance.rs`, `bdd_operations.rs`, `bdd_community.rs`)
   - [ ] Step definitions Tier 1 (120 scenarios) : payments, tickets, notifications, resolutions, convocations, quotes, 2FA, organizations
   - [ ] Step definitions Tier 2 (100 scenarios) : notices, skills, shared_objects, resource_bookings, gamification, energy_campaigns
   - [ ] Step definitions Tier 3 (59 scenarios) : journal_entries, call_for_funds, owner_contributions, charge_distribution, iot, work_reports, technical_inspections, dashboard, public_syndic
   
   ### Tests E2E Backend (11h)
   - [ ] Fix `common/mod.rs` avec tous les repos/use_cases
   - [ ] Fix 19 fichiers `e2e_*.rs` compilation
   - [ ] CI : `cargo test --test 'e2e*'`
   
   ### Playwright Frontend (15h)
   - [ ] `Login.spec.ts`
   - [ ] `Buildings.spec.ts`
   - [ ] `Expenses.spec.ts`
   - [ ] `Meetings.spec.ts`
   - [ ] `Tickets.spec.ts`
   - [ ] `Notifications.spec.ts`
   - [ ] `OwnerDashboard.spec.ts`
   
   ### Documentation (20h)
   - [ ] `docs/PAYMENT_INTEGRATION.rst`
   - [ ] `docs/TICKETS_MAINTENANCE.rst`
   - [ ] `docs/NOTIFICATIONS_SYSTEM.rst`
   - [ ] `docs/CONVOCATIONS_AG.rst`
   - [ ] `docs/CONTRACTOR_QUOTES.rst`
   - [ ] `docs/COMMUNITY_FEATURES.rst`
   - [ ] `backend/README.md`
   - [ ] `frontend/README.md`
   
   ### Release Mechanics (5h)
   - [ ] CHANGELOG.md restructuré [0.5.0]
   - [ ] Version bump Cargo.toml + package.json
   - [ ] CI pipeline mis à jour (bdd*, e2e*, playwright)
   - [ ] Tag v0.5.0 + GitHub Release
   
   ## Métriques cibles
   
   | Couche | Actuel | Cible |
   |--------|--------|-------|
   | Unit tests | 501 | 550+ |
   | BDD scenarios | 194 passants | 473 passants |
   | E2E backend | 30 passants | 215 passants |
   | Playwright | 27 | 64 |
   | Docs features | 8/30 | 16/30 |
   
   ## Références
   - WBS détaillé : `docs/WBS_RELEASE_0_5_0.rst`
   - WBS projet complet : `docs/WBS_PROJET_COMPLET.rst`
   - Plan approuvé : `.claude/plans/recursive-skipping-music.md`

.. raw:: html

   </div>


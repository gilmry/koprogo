# WBS Release 0.1.0 — KoproGo

## Contexte

Première release officielle. Branche `release/0.1.0`, main stable pour GitOps.
Avant d'écrire du code ou des tests, on fait un **audit sémantique** : chaque promesse métier des Jalons 0-3 est-elle spécifiée (BDD), implémentée (backend), câblée (frontend) et testée (E2E) ?

---

## Stratégie de Test — Pyramide KoproGo

- **Unit** : Logique domaine pure (entités, calculs, state machines)
- **BDD** : Spécification métier Given/When/Then — chaque workflow métier significatif
- **E2E Backend** : Contrat API HTTP (status codes, shapes, permissions, DB state)
- **E2E Frontend** : Mêmes workflows vus côté utilisateur (Playwright)
- **Contract Tests** : Cohérence DTO backend ↔ frontend via OpenAPI auto-généré (utoipa → openapi-typescript)

---

## MATRICE DE TRAÇABILITÉ — Jalons 0-3

Légende : ✅ = OK | ❌ = Manquant | ⚠️ = Partiel | n/a = Non applicable

### JALON 0 — Fondations Techniques ✅ (Milestone #5)

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #28 | Multi-rôles utilisateur | auth_use_cases | ✅ auth | ✅ e2e_auth | ✅ login | ✅ (api.ts) | ✅ Login |
| #30 | Seed comptes test | seed (bin) | ✅ building | n/a | ✅ admin/seed | n/a | ⚠️ Admin |
| #33 | Multi-owner docs + hooks | unit_owner_use_cases | ✅ building | ✅ e2e_unit_owner | ✅ units | ✅ (api.ts) | ❌ |
| — | Buildings CRUD | building_use_cases | ✅ building | ❌ | ✅ buildings | ✅ (api.ts) | ✅ Buildings |
| — | Units CRUD | unit_use_cases | ✅ building | ❌ | ✅ units | ✅ (api.ts) | ❌ |
| — | Owners CRUD | owner_use_cases | ⚠️ (dans building) | ❌ | ✅ owners | ✅ (api.ts) | ⚠️ OwnerDash |
| — | Meetings CRUD | meeting_use_cases | ✅ meetings | ✅ e2e_meetings | ✅ meetings | ✅ (api.ts) | ✅ Meetings |

### JALON 1 — Sécurité & GDPR 🔒 (Milestone #6)

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #39 | LUKS encryption at-rest | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #40 | Backups GPG + S3 | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #41 | Monitoring Prometheus/Grafana | n/a (infra) | n/a | n/a | ✅ admin/monitoring | n/a | n/a |
| #42 | GDPR export + erasure (Art 15,17) | gdpr_use_cases | ✅ gdpr | ✅ e2e_gdpr | ✅ settings/gdpr | ✅ (api.ts) | ✅ Gdpr |
| #43 | Security hardening (fail2ban, WAF, IDS) | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #78 | 2FA TOTP + rate limiting | two_factor_use_cases | ✅ two_factor | ❌ | ✅ settings | ✅ (api.ts) | ❌ |
| #90 | GDPR Art 16,18,21 (rectify, restrict, marketing) | gdpr_use_cases | ✅ gdpr | ✅ e2e_gdpr | ✅ settings/gdpr | ✅ (api.ts) | ⚠️ Gdpr (partiel) |
| #271 | Quorum 50%+ AG (Art 3.87§5) | meeting_use_cases | ⚠️ meetings | ✅ e2e_meetings | ✅ meetings | ✅ | ❌ |
| #272 | 2e convocation si quorum non atteint | convocation_use_cases | ✅ convocations | ✅ e2e_convocations | ✅ convocations | ✅ | ❌ |
| #273 | Réduction vote mandataire (Art 3.87§7) | resolution_use_cases | ✅ resolutions | ✅ e2e_resolutions | ✅ meetings | ✅ resolutions | ❌ |

### JALON 2 — Conformité Légale Belge 📋 (Milestone #7) — Tout fermé

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #79 | PCMN comptabilité belge | account_use_cases | ⚠️ expenses_pcn | ❌ | ✅ accountant | ✅ mcp | ❌ |
| #73 | Factures + workflow approbation | expense_use_cases | ✅ invoices | ❌ | ✅ invoice-workflow | ✅ (api.ts) | ✅ Expenses |
| #83 | Relances impayés 4 niveaux | payment_reminder_use_cases | ✅ payment_recovery | ✅ e2e_payment_recovery | ✅ payment-reminders | ✅ payment-reminders | ⚠️ Expenses |
| #77 | Rapports financiers (bilan, résultats) | financial_report_use_cases | ⚠️ expenses_pcn | ❌ | ✅ reports | ✅ mcp | ❌ |
| #80 | État daté (vente immobilière) | etat_date_use_cases | ✅ etat_date | ✅ e2e_etat_date | ✅ etats-dates | ✅ etats-dates | ❌ |
| #81 | Budget annuel + variance | budget_use_cases | ✅ budget | ✅ e2e_budget | ✅ budgets | ✅ budgets | ❌ |
| #82 | Conseil copropriété (obligatoire >20 lots) | board_member_use_cases + board_decision_use_cases | ✅ board_members + board_decisions | ✅ e2e_board | ✅ board-dashboard | ✅ (api.ts) | ✅ BoardOfDirectors |
| #29 | Validation quotes-parts 100% | unit_owner_use_cases | ✅ building | ✅ e2e_unit_owner | ✅ units | ✅ (api.ts) | ❌ |
| #76 | Gestion documentaire complète | document_use_cases | ✅ documents* | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #75 | AG assemblées générales complètes | meeting_use_cases | ✅ meetings* | ✅ e2e_meetings | ✅ meetings | ✅ (api.ts) | ✅ Meetings |
| #44 | Stratégie stockage documents | document_use_cases | ✅ documents | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #45 | Upload drag-and-drop | document_use_cases | ✅ documents | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #51 | Sondages conseil (polls) | poll_use_cases | ✅ polls | ❌ | ✅ polls | ✅ polls | ❌ |
| #200 | Journal entries (double-entry) | journal_entry_use_cases | ✅ journal_entries | ❌ | ✅ journal-entries | ✅ (api.ts) | ❌ |
| #201 | Appels de fonds | call_for_funds_use_cases | ✅ call_for_funds | ❌ | ✅ call-for-funds | ✅ (api.ts) | ❌ |
| #202 | Suivi versements propriétaires | owner_contribution_use_cases | ✅ owner_contributions | ❌ | ✅ owner-contributions | ✅ (api.ts) | ❌ |
| #205 | Répartition charges | charge_distribution_use_cases | ✅ charge_distribution | ❌ | ⚠️ (API only) | ✅ charge-dist | ❌ |

### JALON 3 — Features Différenciantes 🎯 (Milestone #8)

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #46 | Votes AG (3 majorités, tantièmes) | resolution_use_cases | ✅ resolutions | ✅ e2e_resolutions | ✅ meetings | ✅ resolutions | ❌ |
| #84 | Paiements Stripe + SEPA | payment_use_cases + payment_method_use_cases | ✅ payments + payment_methods | ✅ e2e_payments | ✅ owner/payments | ✅ payments | ❌ |
| #49 Ph1 | SEL (échange local temps) | local_exchange_use_cases | ✅ local_exchange | ✅ e2e_local_exchange | ✅ exchanges | ✅ local-exchanges | ❌ |
| #49 Ph2 | Notices communautaires | notice_use_cases | ✅ notices | ❌ | ✅ notices | ✅ notices | ❌ |
| #49 Ph3 | Annuaire compétences | skill_use_cases | ✅ skills | ❌ | ✅ skills | ✅ skills | ❌ |
| #49 Ph4 | Bibliothèque objets partagés | shared_object_use_cases | ✅ shared_objects | ❌ | ✅ sharing | ✅ sharing | ❌ |
| #49 Ph5 | Réservation ressources | resource_booking_use_cases | ✅ resource_bookings | ❌ | ✅ bookings | ✅ bookings | ❌ |
| #49 Ph6 | Gamification (achievements, challenges) | achievement + challenge + gamification_stats | ✅ gamification | ❌ | ✅ gamification | ✅ gamification | ❌ |
| #133 | IoT Linky smart meters | iot_use_cases + linky_use_cases | ✅ iot | ❌ | ❌ | ❌ | ❌ |
| #134 | Work reports | work_report_use_cases | ✅ work_reports | ❌ | ✅ work-reports | ✅ work-reports | ❌ |
| #134 | Technical inspections | technical_inspection_use_cases | ✅ technical_inspections | ❌ | ✅ inspections | ✅ inspections | ❌ |
| #52/#91 | Devis entrepreneurs | quote_use_cases | ✅ quotes | ✅ e2e_quotes | ✅ quotes | ✅ quotes | ❌ |
| #88 | Convocations AG auto | convocation_use_cases | ✅ convocations | ✅ e2e_convocations | ✅ convocations | ✅ convocations | ❌ |
| #86 | Notifications multi-canal | notification_use_cases | ✅ notifications | ✅ e2e_notifications | ✅ notifications | ✅ notifications | ✅ Notifications |
| #85 | Tickets maintenance | ticket_use_cases | ✅ tickets | ✅ e2e_tickets | ✅ tickets | ✅ tickets | ✅ Tickets |
| #92 | Syndic info publique | building_use_cases | ✅ public_syndic | ❌ | ✅ (public endpoint) | ✅ (api.ts) | ❌ |
| #96 | Energy campaigns (achat groupé) | energy_campaign + energy_bill_upload | ✅ energy_campaigns | ❌ | ✅ energy-campaigns | ✅ energy-campaigns | ❌ |
| #274 | BC15: AG Visioconférence | ag_session_use_cases | ✅ ag_sessions | ❌ | ❌ | ❌ | ❌ |
| #279 | BC17: AGE agile (demande 1/5) | age_request_use_cases | ✅ age_requests | ❌ | ❌ | ❌ | ❌ |
| #275 | BC16: Backoffice prestataires PWA | contractor_report_use_cases | ✅ contractor_reports | ❌ | ✅ contractor/[token] | ⚠️ (direct fetch) | ❌ |
| #276 | BC14: Marketplace + satisfaction | — | ❌ | ❌ | ❌ | ❌ | ❌ |
| #277 | Guide légal contextuel UI | — | ❌ | ❌ | ❌ | ❌ | ❌ |
| #280 | Orchestrateur énergie neutre | — | ❌ | ❌ | ❌ | ❌ | ❌ |

---

## SYNTHÈSE DES LACUNES

### Par couche — nombre de promesses métier avec trous :

| Couche | Jalons 0-3 total | ✅ OK | ❌ Manquant | Taux couverture |
|--------|-----------------|-------|-----------|-----------------|
| Use Case (backend impl) | 49 | 46 | 3 (BC14, #277, #280) | 94% |
| BDD feature | 46 impl | 44 | 2 (accounts dédié, expenses dédié) | 96% |
| E2E Backend | 46 impl | 19 | 27 | 41% |
| Frontend page | 46 impl | 43 | 3 (AG Sessions, AGE Requests, IoT) | 93% |
| API Client TS | 43 pages | 41 | 2 (AG Sessions, IoT) | 95% |
| Playwright | 43 pages | 11 | 32 | 26% |
| Contract DTO | — | 0 | tout | 0% |

### TOP 3 des lacunes critiques :

1. **Playwright** : 26% de couverture — 32 features sans test E2E frontend
2. **E2E Backend** : 41% — 27 use cases sans test HTTP dédié
3. **Contract Tests DTO** : 0% — aucun mécanisme de cohérence backend↔frontend

### Features Jalon 3 NON complètes (issues ouvertes) :
- #274 BC15 AG Visioconférence — backend ✅, BDD ✅ → manque E2E + frontend
- #279 BC17 AGE agile — backend ✅, BDD ✅ → manque E2E + frontend
- #275 BC16 Contractor PWA — backend ✅, BDD ✅, frontend ✅ (page) → manque E2E
- #276 BC14 Marketplace — non implémenté
- #277 Guide légal UI — non implémenté
- #280 Orchestrateur énergie — non implémenté

> **Hors scope 0.1.0** (repoussé) :
> - #252-265 MCP Tools AI Syndic (14 issues) → `release:0.2.0`, Jalon 4
> - #48 Auth itsme/eID → Jalon 4 (pas de release assignée)

---

## PLAN D'ACTION (par priorité)

### Phase 0 — Contract Tests DTO (fondation)
- [ ] 0.1 utoipa sur tous les DTOs backend → openapi.json
- [ ] 0.2 openapi-typescript → `frontend/src/types/api.d.ts`
- [ ] 0.3 CI : diff check types générés

### Phase 1 — Nettoyage
- [x] 1.1 Supprimer .bak et .disabled — commit `a9100b7`
- [x] 1.2 Commit fichiers non-trackés — commits `6c1e26a`, `6196593`, `73b2de6`
- [x] 1.3 Câbler features BDD orphelines (i18n, legal_compliance) — dans bdd.rs

### Phase 2 — BDD manquants (5 features)
- [x] 2.1 ag_sessions.feature → bdd_governance.rs — ✅ 0 failures, 0 skips
- [x] 2.2 age_requests.feature → bdd_governance.rs — ✅ 0 failures, 0 skips
- [x] 2.3 contractor_reports.feature → bdd_operations.rs — ✅ 0 failures, 0 skips
- [x] 2.4 accounts.feature (PCMN dédié) → bdd_financial.rs — ✅ 0 failures, 0 skips
- [x] 2.5 expenses.feature (dédié, consolider partiel) → bdd_financial.rs — ✅ 0 failures, 0 skips

### Phase 3 — E2E Backend manquants (les 27 trous) ✅ COMPLETE
Par priorité — features avec logique métier complexe d'abord :
- [x] 3.1 e2e_ag_session.rs — ✅ 8 tests, 0 failures
- [x] 3.2 e2e_age_request.rs — ✅ 6 tests, 0 failures
- [x] 3.3 e2e_contractor_report.rs — ✅ 7 tests, 0 failures
- [x] 3.4 e2e_polls.rs — ✅ 8 tests, 0 failures
- [x] 3.5 e2e_energy_campaigns.rs — ✅ 5 tests, 0 failures
- [x] 3.6 e2e_gamification.rs — ✅ 7 tests, 0 failures
- [x] 3.7 e2e_journal_entries.rs — ✅ 6 tests, 0 failures
- [x] 3.8 e2e_call_for_funds.rs — ✅ 7 tests, 0 failures
- [x] 3.9 e2e_owner_contributions.rs — ✅ 7 tests, 0 failures
- [x] 3.10 e2e_charge_distribution.rs — ✅ 4 tests, 0 failures
- [x] 3.11 e2e_work_reports.rs — ✅ 6 tests, 0 failures
- [x] 3.12 e2e_technical_inspections.rs — ✅ 6 tests, 0 failures
- [x] 3.13 e2e_iot.rs — ✅ 6 tests, 0 failures
- [x] 3.14 e2e_two_factor.rs — ✅ 6 tests, 0 failures
- [x] 3.15 e2e_skills.rs — ✅ 5 tests, 0 failures
- [x] 3.16 e2e_shared_objects.rs — ✅ 6 tests, 0 failures
- [x] 3.17 e2e_resource_bookings.rs — ✅ 5 tests, 0 failures
- [x] 3.18 e2e_notices.rs — ✅ 5 tests, 0 failures
- [x] 3.19 e2e_accounts.rs — ✅ 8 tests, 0 failures
- [x] 3.20 e2e_financial_reports.rs — ✅ 6 tests, 0 failures
- [x] 3.21 e2e_buildings.rs — ✅ 8 tests, 0 failures
- [x] 3.22 e2e_units.rs — ✅ 6 tests, 0 failures
- [x] 3.23 e2e_owners.rs — ✅ 6 tests, 0 failures
- [x] 3.24 e2e_dashboard.rs — ✅ 4 tests, 0 failures
- [x] 3.25 e2e_public_syndic.rs — ✅ 4 tests, 0 failures
- [x] 3.26 e2e_organizations.rs — ✅ 4 tests, 0 failures
- [x] 3.27 e2e_users.rs — ✅ 4 tests, 0 failures

### Phase 4 — Vérification (tout compile, tout passe) ✅ COMPLETE
- [x] 4.1 BDD — 5 fichiers, 454 scénarios, 0 failures, 0 skips — `c739116`
- [x] 4.2 E2E Backend — 48 fichiers, ~320 tests, 0 failures — Phase 3 complete

### Phase 5 — Playwright manquants (32 trous, mêmes workflows que E2E backend)
Tier 1 — Critiques :
- [ ] 5.1 Payments.spec.ts
- [ ] 5.2 Invoices.spec.ts
- [ ] 5.3 Convocations.spec.ts
- [ ] 5.4 Resolutions.spec.ts
- [ ] 5.5 Quotes.spec.ts
- [ ] 5.6 TwoFactor.spec.ts

Tier 2 — Admin/syndic :
- [ ] 5.7 Budgets.spec.ts
- [ ] 5.8 EtatsDates.spec.ts
- [ ] 5.9 EnergyCampaigns.spec.ts
- [ ] 5.10 LocalExchanges.spec.ts
- [ ] 5.11 Gamification.spec.ts
- [ ] 5.12 Polls.spec.ts
- [ ] 5.13 PaymentRecovery.spec.ts

Tier 3 — Communauté/support :
- [ ] 5.14 Documents.spec.ts
- [ ] 5.15 JournalEntries.spec.ts
- [ ] 5.16 CallForFunds.spec.ts
- [ ] 5.17 OwnerContributions.spec.ts
- [ ] 5.18 WorkReports.spec.ts
- [ ] 5.19 TechnicalInspections.spec.ts
- [ ] 5.20 Notices.spec.ts
- [ ] 5.21 Skills.spec.ts
- [ ] 5.22 Sharing.spec.ts
- [ ] 5.23 Bookings.spec.ts
- [ ] 5.24 UnitOwners.spec.ts
- [ ] 5.25 BoardManagement.spec.ts
- [ ] 5.26 ChargeDistribution.spec.ts
- [ ] 5.27 PublicSyndic.spec.ts
- [ ] 5.28 Accounts.spec.ts
- [ ] 5.29 FinancialReports.spec.ts
- [ ] 5.30 Organizations.spec.ts
- [ ] 5.31 Dashboard.spec.ts
- [ ] 5.32 I18n.spec.ts

### Phase 6 — Développement features manquantes Jalon 3 (TDD/BDD Red-Green-Commit)

**Workflow obligatoire pour chaque feature :**
1. RED : Écrire le test BDD (.feature) qui échoue
2. GREEN : Implémenter le code minimal pour que le test passe
3. COMMIT : Commit atomique avec git hooks (format + lint + tests)
4. REFACTOR : Nettoyer si nécessaire, re-commit

**Git hooks actifs** (`make install-hooks`) :
- `pre-commit` → `make format` + `make lint`
- `pre-push` → `make lint` + `make test` (unit + BDD + build frontend)

**CI** : Chaque push sur `release/0.1.0` déclenche GitHub Actions — **tous les jobs doivent être verts** avant de continuer.

#### 6.1 P0 Legal (~4h)
- [ ] #271 Quorum 50%+ validation AG — vérifier wiring migration + tests
- [ ] #272 2e convocation si quorum non atteint — vérifier wiring + tests

#### 6.2 P0 BC (~26h)
- [ ] #274 BC15: AG Visioconférence — BDD → E2E backend → frontend (pages + API client + composants) → Playwright
- [ ] #279 BC17: AGE agile — BDD → E2E backend → frontend → Playwright

#### 6.3 P1 BC (~36h)
- [ ] #275 BC16: Backoffice prestataires — BDD → E2E backend (frontend page existe déjà)
- [ ] #276 BC14: Marketplace corps de métier — domain → use cases → repo → handlers → BDD → E2E → frontend → Playwright

#### 6.4 P1 Tools (~10h)
- [ ] #277 Guide légal contextuel UI (LegalHelper.svelte, AG Wizard)

#### 6.5 P1 Energy (~16h)
- [ ] #280 Orchestrateur énergie neutre (CER, maisons individuelles, CREG)

#### 6.6 P2 Content (~22h)
- [ ] #278 Blog 18 articles RST (5 séries thématiques)

### Phase 7 — Documentation complète du logiciel

#### 7.1 Documentation technique
- [ ] Architecture hexagonale (schéma ports & adapters, flux données)
- [ ] Guide déploiement (Docker, K3s, variables env, migrations)
- [ ] Guide développeur (setup local, conventions, workflow TDD/BDD)
- [ ] API Reference (auto-générée depuis OpenAPI/utoipa)
- [ ] Modèle de données (ERD auto-généré ou documenté)

#### 7.2 Documentation utilisateur
- [ ] Guide syndic (parcours AG, convocations, votes, budget)
- [ ] Guide copropriétaire (paiements, tickets, communauté, SEL)
- [ ] Guide comptable (PCMN, journal entries, rapports financiers, état daté)
- [ ] Guide administrateur (organisations, users, monitoring, GDPR)

#### 7.3 Vidéos Playwright comme preuves des parcours utilisateurs
Les tests Playwright enregistrent des vidéos (déjà configuré : 1280x720). Ces vidéos sont :
- [ ] Collectées après chaque run Playwright réussi
- [ ] Organisées par feature dans `docs/videos/` ou publiées sur GitHub Pages
- [ ] Intégrées dans la documentation utilisateur comme démonstrations visuelles
- [ ] Référencées dans les release notes comme preuve de fonctionnement

**Parcours vidéo obligatoires (1 vidéo = 1 workflow complet) :**
- [ ] Login → Dashboard → Navigation
- [ ] Créer bâtiment → Ajouter lots → Ajouter copropriétaires
- [ ] Convoquer AG → Voter → Clôturer → PV
- [ ] Encoder facture → Approuver → Payer → Relance
- [ ] Créer devis → Comparer → Valider → Rapport prestataire
- [ ] SEL : Offrir service → Demander → Compléter → Rating
- [ ] GDPR : Export données → Anonymisation
- [ ] Copropriétaire : Demande AGE → Signatures → Soumission syndic

### Phase 8 — Revue humaine UI/UX

**La release est BLOQUÉE tant que cette revue n'est pas validée.**

#### 8.1 Revue de cohérence UI
- [ ] Navigation : tous les liens fonctionnent, pas de pages orphelines
- [ ] Formulaires : tous les champs obligatoires sont marqués, messages d'erreur clairs
- [ ] Responsive : chaque page testée mobile (375px) + desktop (1440px)
- [ ] i18n : vérifier FR/NL au minimum sur les parcours critiques
- [ ] Accessibilité : vérifier WCAG 2.1 AA (Playwright Accessibility.spec.ts)

#### 8.2 Revue de valeur métier
- [ ] Chaque promesse Jalon 0-3 est accessible depuis l'UI en ≤3 clics
- [ ] Les workflows correspondent à la réalité d'un syndic belge
- [ ] Les termes juridiques sont corrects (Art. CC, PCMN, TVA, état daté)
- [ ] Les calculs sont vérifiés (quorum, majorités, quotes-parts, pénalités)
- [ ] La valeur livrée est conforme à la vision du projet

#### 8.3 Validation finale
- [ ] Checklist de revue signée (document dans `docs/release/`)
- [ ] Bugs bloquants identifiés → corrigés → re-testés
- [ ] Décision GO/NO-GO pour la release

### Phase 9 — WBS documenté + traçabilité GitHub

#### 9.1 WBS sur le dépôt
- [ ] Documenter le WBS final dans `docs/WBS_RELEASE_0_1_0.rst` (ou .md)
- [ ] Inclure la matrice de traçabilité (Issue → BDD → E2E → Frontend → Playwright)
- [ ] Inclure les métriques finales (LOC, tests, couverture, endpoints)

#### 9.2 Traçabilité GitHub
- [ ] Chaque issue Jalon 0-3 est fermée avec lien vers le commit/PR
- [ ] Milestones 5-8 à 100% (toutes issues fermées)
- [ ] Labels cohérents sur toutes les issues
- [ ] GitHub Project board reflète le WBS (colonnes par phase)
- [ ] Chaque PR de la release référence l'issue correspondante

#### 9.3 CI verte finale
- [ ] Tous les GitHub Actions jobs verts sur `release/0.1.0`
- [ ] Coverage report généré et archivé
- [ ] Security audit (`make audit`) sans vulnérabilité critique
- [ ] Docker images buildées et testées

### Phase 10 — Tag & Release

- [ ] 10.1 Merge `release/0.1.0` → `main` (PR avec review)
- [ ] 10.2 Tag `v0.1.0` sur main
- [ ] 10.3 GitHub Release avec :
  - Release notes (features, breaking changes, known issues)
  - Lien vers documentation
  - Lien vers vidéos Playwright
  - Matrice de couverture des tests
- [ ] 10.4 Docker build/push déclenché automatiquement sur tag
- [ ] 10.5 Documentation déployée sur GitHub Pages

---

## Scope 0.1.0 : Jalons 0-3

**Décision** : La 0.1.0 couvre les Jalons 0-3. Les MCP Tools (#252-265) et l'auth itsme (#48) sont repoussés en Jalon 4 / release 0.2.0.

### Issues encore ouvertes (à compléter)

#### Jalon 1 — Bugs legal (3 issues)

| Issue | Titre | Effort estimé | État |
|-------|-------|---------------|------|
| #271 | Quorum 50%+ validation AG (Art 3.87§5) | ~2h | migration existe, vérifier wiring |
| #272 | 2e convocation si quorum non atteint (Art 3.87§5) | ~2h | migration existe, vérifier wiring |
| #273 | Réduction vote mandataire (Art 3.87§7) | ~2h | ✅ done (à fermer) |

#### Jalon 3 — Features (7 issues)

| Issue | Titre | Effort estimé | État |
|-------|-------|---------------|------|
| #274 | BC15: AG Visioconférence (AgSession, quorum combiné) | ~8h | backend ✅, BDD ✅, manque E2E + frontend |
| #275 | BC16: Backoffice prestataires PWA (ContractorReport) | ~6h | backend ✅, BDD ✅, frontend ✅, manque E2E |
| #276 | BC14: Marketplace corps de métier + satisfaction | ~20h | non implémenté |
| #277 | Guide légal contextuel UI (LegalHelper, AG Wizard) | ~10h | non implémenté |
| #278 | Blog 18 articles RST | ~22h | docs only |
| #279 | BC17: AGE agile (demande 1/5, concertation) | ~8h | backend ✅, BDD ✅, manque E2E + frontend |
| #280 | Orchestrateur énergie neutre (CER, CREG) | ~16h | non implémenté |

**Effort total restant estimé : ~96h** (features ~90h + bugs legal ~6h)

### Ordre de priorité

1. **P0 Legal** (#271, #272, #273) — bugs conformité, ~6h
2. **P0 BC** (#274, #279) — AG visio + AGE agile, ~16h (backend + BDD faits, manque E2E + frontend)
3. **P1 BC** (#275, #276) — prestataires + marketplace, ~26h
4. **P1 Tools** (#277) — guide légal UI, ~10h
5. **P1 Energy** (#280) — orchestrateur, ~16h
6. **P2 Content** (#278) — blog, ~22h

Après l'implémentation de chaque feature, elle passe dans la matrice de traçabilité (BDD → E2E backend → frontend → Playwright → contract test DTO).

> **Mise à jour** : 15 mars 2026 — MCP (#252-265) et itsme (#48) repoussés hors 0.1.0. Effort réduit de ~145h à ~96h.

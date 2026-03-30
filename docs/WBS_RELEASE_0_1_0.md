# WBS Release 0.1.0 — KoproGo

## Contexte

Première release officielle. Branche `release/0.1.0`, main stable pour GitOps.
Avant d'écrire du code ou des tests, on fait un **audit sémantique** : chaque promesse métier des Jalons 0-3 est-elle spécifiée (BDD), implémentée (backend), câblée (frontend) et testée (E2E) ?

> **Mise à jour 2026-03-29 (v3)** : Audit et harmonisation des 6 documents Maury.
> Tous les documents (product-brief, PRD, architecture, epics-and-stories, validation-report, estimation)
> sont maintenant alignés sur le code : 4 majorités Art. 3.88 CC (Absolute/TwoThirds/FourFifths/Unanimity),
> voting power 0-10000 dix-millièmes, 21 personas, Résidence du Parc Royal 182 lots.
> INC-04 (nomenclature majorités) résolu. 921 BDD scenarios. 560 endpoints.
> CI: 7/8 green (Playwright à investiguer).
>
> **Mise à jour 2026-03-29 (v2)** : Analyse BMAD vs codebase réelle — infra = 52% des commits.
> Issues #354 (Tests IaC) et #355 (Restructuration IaC) créées dans Jalon 1.
> L'infra (920 commits, 18.7k LOC, 14 rôles Ansible, 4 modules Terraform) n'a **0 tests automatisés**.
> Méthode Maury v2 corrigée : IaC + CI/CD + DTOs comme couches full-stack.
> YAGNI + DRY ajoutés aux invariants de qualité.
>
> **Mise à jour 2026-03-29** : Chaîne Test-Driven Emergence complète (#346-#350).
> Specs multi-rôles (21 personas, 3 immeubles, 5014 lignes). Seeds faker+teardown.
> 146 BDD workflow scenarios. 12 E2E avec seed+teardown. MajorityType Art. 3.88 (4 types).
> CI: 1160 unit tests OK, clippy clean, npm audit 0 vulns. Issue #353 crowdlending R&D.
>
> **2026-03-28** : 12 scénarios Documentation Vivante écrits.
> Architecture hexagonale frontend (#343). Diagnostic multi-rôles (#345).
> Stratégie Test-Driven Emergence (#346-#350). RFC RACE (#344).

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
| — | Buildings CRUD | building_use_cases | ✅ building | ✅ e2e_buildings | ✅ buildings | ✅ (api.ts) | ✅ Buildings |
| — | Units CRUD | unit_use_cases | ✅ building | ✅ e2e_units | ✅ units | ✅ (api.ts) | ❌ |
| — | Owners CRUD | owner_use_cases | ⚠️ (dans building) | ✅ e2e_owners | ✅ owners | ✅ (api.ts) | ⚠️ OwnerDash |
| — | Meetings CRUD | meeting_use_cases | ✅ meetings | ✅ e2e_meetings | ✅ meetings | ✅ (api.ts) | ✅ Meetings |

### JALON 1 — Sécurité & GDPR 🔒 (Milestone #6)

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #39 | LUKS encryption at-rest | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #40 | Backups GPG + S3 | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #41 | Monitoring Prometheus/Grafana | n/a (infra) | n/a | n/a | ✅ admin/monitoring | n/a | n/a |
| #42 | GDPR export + erasure (Art 15,17) | gdpr_use_cases | ✅ gdpr | ✅ e2e_gdpr | ✅ settings/gdpr | ✅ (api.ts) | ✅ Gdpr |
| #43 | Security hardening (fail2ban, WAF, IDS) | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #78 | 2FA TOTP + rate limiting | two_factor_use_cases | ✅ two_factor | ✅ e2e_two_factor | ✅ settings | ✅ (api.ts) | ❌ |
| #90 | GDPR Art 16,18,21 (rectify, restrict, marketing) | gdpr_use_cases | ✅ gdpr | ✅ e2e_gdpr | ✅ settings/gdpr | ✅ (api.ts) | ⚠️ Gdpr (partiel) |
| #271 ✅ | Quorum 50%+ AG (Art 3.87§5) | meeting_use_cases | ✅ meetings | ✅ e2e_meetings | ✅ meetings | ✅ | ❌ |
| #272 ✅ | 2e convocation si quorum non atteint | convocation_use_cases | ✅ convocations | ✅ e2e_convocations | ✅ convocations | ✅ | ⚠️ Scen |
| #273 ✅ | Réduction vote mandataire (Art 3.87§7) | resolution_use_cases | ✅ resolutions | ✅ e2e_resolutions | ✅ meetings | ✅ resolutions | ❌ |
| #326 ✅ | GDPR Consent (Art. 7) | consent_use_cases | ✅ consent | ✅ e2e_consent | ✅ ConsentModal | ✅ Consent | ❌ |
| #327 ✅ | Security Incidents (Art. 33) | security_incident_use_cases | ✅ | ✅ e2e_security_incidents | ⚠️ | ✅ SecurityIncidents | ❌ |
| #328 ✅ | API Key Management | api_key_use_cases | ✅ | ✅ e2e_api_keys | ⚠️ | ✅ ApiKeys | ❌ |
| #329 ✅ | GDPR Art. 30 Register | gdpr_art30_use_cases | ✅ | ✅ e2e_gdpr_art30 | ⚠️ | ⚠️ | ❌ |
| #343 ✅ | Hexa frontend + testids + i18n | n/a | n/a | n/a | ✅ 105 composants | ✅ 22 clients | ⚠️ 6/12 |
| #354 | **Tests IaC** (terraform validate, ansible-lint, molecule, conftest ISO 27001) | n/a (infra) | n/a | n/a | n/a | n/a | n/a |
| #355 | **Restructuration IaC** (repo séparé, tests, policy-as-code, CI/CD infra) | n/a (infra) | n/a | n/a | n/a | n/a | n/a |

### JALON 2 — Conformité Légale Belge 📋 (Milestone #7) — Tout fermé

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #79 | PCMN comptabilité belge | account_use_cases | ⚠️ expenses_pcn | ✅ e2e_accounts | ✅ accountant | ✅ mcp | ❌ |
| #73 | Factures + workflow approbation | expense_use_cases | ✅ invoices | ✅ e2e_invoices | ✅ invoice-workflow | ✅ (api.ts) | ✅ Expenses |
| #83 | Relances impayés 4 niveaux | payment_reminder_use_cases | ✅ payment_recovery | ✅ e2e_payment_recovery | ✅ payment-reminders | ✅ payment-reminders | ⚠️ Expenses |
| #77 | Rapports financiers (bilan, résultats) | financial_report_use_cases | ⚠️ expenses_pcn | ✅ e2e_financial_reports | ✅ reports | ✅ mcp | ❌ |
| #80 | État daté (vente immobilière) | etat_date_use_cases | ✅ etat_date | ✅ e2e_etat_date | ✅ etats-dates | ✅ etats-dates | ❌ |
| #81 | Budget annuel + variance | budget_use_cases | ✅ budget | ✅ e2e_budget | ✅ budgets | ✅ budgets | ❌ |
| #82 | Conseil copropriété (obligatoire >20 lots) | board_member_use_cases + board_decision_use_cases | ✅ board_members + board_decisions | ✅ e2e_board | ✅ board-dashboard | ✅ (api.ts) | ✅ BoardOfDirectors |
| #29 | Validation quotes-parts 100% | unit_owner_use_cases | ✅ building | ✅ e2e_unit_owner | ✅ units | ✅ (api.ts) | ❌ |
| #76 | Gestion documentaire complète | document_use_cases | ✅ documents* | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #75 | AG assemblées générales complètes | meeting_use_cases | ✅ meetings* | ✅ e2e_meetings | ✅ meetings | ✅ (api.ts) | ✅ Meetings |
| #44 | Stratégie stockage documents | document_use_cases | ✅ documents | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #45 | Upload drag-and-drop | document_use_cases | ✅ documents | ✅ e2e_documents | ✅ documents | ✅ (api.ts) | ❌ |
| #51 | Sondages conseil (polls) | poll_use_cases | ✅ polls | ✅ e2e_polls | ✅ polls | ✅ polls | ❌ |
| #200 | Journal entries (double-entry) | journal_entry_use_cases | ✅ journal_entries | ✅ e2e_journal_entries | ✅ journal-entries | ✅ (api.ts) | ❌ |
| #201 | Appels de fonds | call_for_funds_use_cases | ✅ call_for_funds | ✅ e2e_call_for_funds | ✅ call-for-funds | ✅ (api.ts) | ❌ |
| #202 | Suivi versements propriétaires | owner_contribution_use_cases | ✅ owner_contributions | ✅ e2e_owner_contributions | ✅ owner-contributions | ✅ (api.ts) | ❌ |
| #205 | Répartition charges | charge_distribution_use_cases | ✅ charge_distribution | ✅ e2e_charge_distribution | ⚠️ (API only) | ✅ charge-dist | ❌ |
| #345 ✅ | Diagnostic multi-rôles | n/a | n/a | n/a | n/a | n/a | n/a |
| #346 ✅ | Specs multi-rôles (8 workflows, 5014 lignes) | ✅ specs | ✅ 8 specs | n/a | n/a | n/a | n/a |
| #347 ✅ | Seeds faker + teardown (21 personas, 3 immeubles) | ✅ seed.rs | n/a | ✅ POST/DELETE /seed | n/a | n/a | n/a |
| #348 ✅ | BDD alignés multi-rôles (146 scenarios) | n/a | ✅ 8 features | n/a | n/a | n/a | n/a |
| #349 ✅ | E2E alignés multi-rôles (12 scenarios) | n/a | n/a | n/a | n/a | n/a | ✅ 12 scenarios |
| #350 ✅ | Gaps légaux : MajorityType 4 types Art. 3.88, dix-millièmes | ✅ resolution.rs | ✅ vote_ag | ✅ e2e_resolutions | n/a | n/a | n/a |

### JALON 3 — Features Différenciantes 🎯 (Milestone #8)

| Issue | Promesse métier | Use Case | BDD | E2E Backend | Frontend Page | API Client | Playwright |
|-------|----------------|----------|-----|-------------|---------------|------------|------------|
| #46 | Votes AG (4 majorités Art. 3.88, dix-millièmes) | resolution_use_cases | ✅ resolutions | ✅ e2e_resolutions | ✅ meetings | ✅ resolutions | ❌ |
| #84 | Paiements Stripe + SEPA | payment_use_cases + payment_method_use_cases | ✅ payments + payment_methods | ✅ e2e_payments | ✅ owner/payments | ✅ payments | ❌ |
| #49 Ph1 | SEL (échange local temps) | local_exchange_use_cases | ✅ local_exchange | ✅ e2e_local_exchange | ✅ exchanges | ✅ local-exchanges | ❌ |
| #49 Ph2 | Notices communautaires | notice_use_cases | ✅ notices | ✅ e2e_notices | ✅ notices | ✅ notices | ❌ |
| #49 Ph3 | Annuaire compétences | skill_use_cases | ✅ skills | ✅ e2e_skills | ✅ skills | ✅ skills | ❌ |
| #49 Ph4 | Bibliothèque objets partagés | shared_object_use_cases | ✅ shared_objects | ✅ e2e_shared_objects | ✅ sharing | ✅ sharing | ❌ |
| #49 Ph5 | Réservation ressources | resource_booking_use_cases | ✅ resource_bookings | ✅ e2e_resource_bookings | ✅ bookings | ✅ bookings | ❌ |
| #49 Ph6 | Gamification (achievements, challenges) | achievement + challenge + gamification_stats | ✅ gamification | ✅ e2e_gamification | ✅ gamification | ✅ gamification | ❌ |
| #133 | IoT Linky smart meters | iot_use_cases + linky_use_cases | ✅ iot | ✅ e2e_iot | ❌ | ❌ | ❌ |
| #300 | IoT MQTT + BOINC Grid | boinc_use_cases + mqtt_adapter | ✅ iot_mqtt_boinc | ❌ | ❌ | ❌ | ❌ |
| #134 | Work reports | work_report_use_cases | ✅ work_reports | ✅ e2e_work_reports | ✅ work-reports | ✅ work-reports | ❌ |
| #134 | Technical inspections | technical_inspection_use_cases | ✅ technical_inspections | ✅ e2e_technical_inspections | ✅ inspections | ✅ inspections | ❌ |
| #52/#91 | Devis entrepreneurs | quote_use_cases | ✅ quotes | ✅ e2e_quotes | ✅ quotes | ✅ quotes | ❌ |
| #88 | Convocations AG auto | convocation_use_cases | ✅ convocations | ✅ e2e_convocations | ✅ convocations | ✅ convocations | ❌ |
| #86 | Notifications multi-canal | notification_use_cases | ✅ notifications | ✅ e2e_notifications | ✅ notifications | ✅ notifications | ✅ Notifications |
| #85 | Tickets maintenance | ticket_use_cases | ✅ tickets | ✅ e2e_tickets | ✅ tickets | ✅ tickets | ✅ Tickets |
| #92 | Syndic info publique | building_use_cases | ✅ public_syndic | ✅ e2e_public_syndic | ✅ (public endpoint) | ✅ (api.ts) | ❌ |
| #96 | Energy campaigns (achat groupé) | energy_campaign + energy_bill_upload | ✅ energy_campaigns | ✅ e2e_energy_campaigns | ✅ energy-campaigns | ✅ energy-campaigns | ❌ |
| #274 ✅ | BC15: AG Visioconférence | ag_session_use_cases | ✅ ag_sessions | ✅ e2e_ag_sessions | ✅ AgVideoSession | ⚠️ | ❌ |
| #279 ✅ | BC17: AGE agile (demande 1/5) | age_request_use_cases | ✅ age_requests | ✅ e2e_age_requests | ✅ AgePetitionProgress | ⚠️ | ❌ |
| #275 ✅ | BC16: Backoffice prestataires PWA | contractor_report_use_cases | ✅ contractor_reports | ✅ e2e_contractor_reports | ✅ contractor/ | ✅ ContractorReport | ❌ |
| #276 ✅ | BC14: Marketplace + satisfaction | marketplace_use_cases | ✅ marketplace | ✅ e2e_marketplace | ✅ marketplace | ✅ Marketplace | ❌ |
| #277 ✅ | Guide légal contextuel UI | legal_use_cases | ✅ legal | ✅ e2e_legal | ✅ LegalHelper | ✅ LegalHelper | ❌ |
| #280 ✅ | Orchestrateur énergie neutre | energy_campaign_use_cases | ✅ energy_campaigns | ✅ e2e_energy_campaigns | ✅ energy-campaigns | ✅ EnergyCampaigns | ❌ |
| #326 ✅ | GDPR Consent (Art. 7) | consent_use_cases | ✅ consent | ✅ e2e_consent | ✅ ConsentModal | ✅ Consent | ❌ |
| #327 ✅ | Security Incidents (Art. 33) | security_incident_use_cases | ✅ security_incidents | ✅ e2e_security_incidents | ⚠️ | ✅ SecurityIncidents | ❌ |
| #328 ✅ | API Key Management | api_key_use_cases | ✅ api_keys | ✅ e2e_api_keys | ⚠️ | ✅ ApiKeys | ❌ |
| #329 ✅ | GDPR Art. 30 Register | gdpr_art30_use_cases | ✅ gdpr_art30 | ✅ e2e_gdpr_art30 | ⚠️ | ⚠️ | ❌ |

---

## SYNTHÈSE DES LACUNES

### Par couche — nombre de promesses métier avec trous :

> **Mise à jour 2026-03-24** : Merge branche `integration` → `main` (48 commits, 65 conflits résolus).
> 55 issues fermées (#220-237, #252-265, #271-280, #300-317, #326-330, #332-334).
> 8 issues créées rétroactivement (#326-334) pour traçabilité du travail non planifié.
> Toutes les features Jalon 3 sont maintenant implémentées (code complet).
> Branches nettoyées : seules main, dev, integration, staging, production restent (identiques).

| Couche | Jalons 0-3 total | ✅ OK | ❌ Manquant | Taux couverture |
|--------|-----------------|-------|-----------|-----------------|
| Use Case (backend impl) | 57 | 57 | 0 | **100%** |
| BDD feature | 57 impl | 57 | 0 | **100%** |
| E2E Backend | 57 impl | 57 | 0 | **100%** |
| Frontend page | 57 impl | 53 | 4 (pages admin pour AG Sessions, AGE, SecurityIncidents, ApiKeys) | 93% |
| API Client TS | 53 pages | 49 | 4 | 92% |
| Playwright | 48 spec files | 217+ pass / à vérifier | Gdpr+Resolutions fixés, à re-vérifier | **~95%** |
| Contract DTO | — | 0 | tout | 0% |
| **Infrastructure IaC** | **236 fichiers** | **0 testés** | **236** | **0%** |
| **Terraform validate** | 39 .tf files | ❌ | 39 | **0%** |
| **Ansible lint** | 47 YAML + 21 J2 | ❌ | 68 | **0%** |
| **Helm lint** | 23 files | ❌ | 23 | **0%** |
| **Shell check** | 36 scripts | ❌ | 36 | **0%** |
| **Policy ISO 27001** | 9 contrôles mappés | ❌ | 9 | **0%** |

### TOP 5 des lacunes restantes :

1. **Infrastructure IaC : 0% de tests** — 236 fichiers, 18.7k LOC, 0 tests automatisés. #354 + #355.
   L'infra = 52% des commits du projet (1 033 / 1 977). Terraform validate, ansible-lint, molecule, conftest ISO 27001 tous manquants. **Bloquant pour la confiance production.**
2. **Contract Tests DTO** : 0% — aucun mécanisme de cohérence backend↔frontend
3. **Playwright** : 21 tests en échec (ApiKeys + SecurityIncidents — null constraint building_id) — Issue #331
4. **Frontend pages admin** : 4 features sans page dédiée complète (AG Sessions, AGE, SecurityIncidents, ApiKeys)
5. **Policy-as-Code ISO 27001** : 9 contrôles mappés (SECURITY.md) mais 0 vérifié automatiquement

### Toutes les features Jalon 3 sont maintenant COMPLÈTES (issues fermées)

- ✅ #274 BC15 AG Visioconférence — **FERMÉE** (backend + frontend + BDD + E2E)
- ✅ #275 BC16 Contractor PWA — **FERMÉE** (backend + frontend PWA + BDD + E2E)
- ✅ #276 BC14 Marketplace — **FERMÉE** (ServiceProvider + ContractEvaluation + handlers)
- ✅ #277 Guide légal UI — **FERMÉE** (LegalHelper.svelte + legal_handlers.rs)
- ✅ #278 Blog 25+ articles — **FERMÉE**
- ✅ #279 BC17 AGE agile — **FERMÉE** (AgeRequest + petition + handlers)
- ✅ #280 Orchestrateur énergie — **FERMÉE** (extensions campagnes)
- ✅ #300 IoT MQTT — **FERMÉE** (mqtt_devices + BOINC)
- ✅ #309 Chaîne Ticket→Rapport→Dépense — **FERMÉE**

### Corrections GitHub effectuées (2026-03-24)

- ✅ 55 issues fermées (voir détail ci-dessus)
- ✅ 8 issues créées rétroactivement (#326-334) pour traçabilité
- ✅ PR #325 : CI fixes (formatting, RUSTSEC-2026-0066, astro check, SSG)
- ✅ 4 PRs dependabot mergées (#318-321 : svelte, tailwindcss, @astrojs/check)
- ✅ Branches nettoyées : main = dev = integration = staging = production
- ✅ #85, #86, #88, #89, #91, #92 déplacées J4 → J3
- ✅ #90 déplacée J4 → J1
- ✅ #300 créée pour IoT MQTT/BOINC (code existait sans issue)

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

- [x] 5.1 Payments.spec.ts — 8 tests (create+retrieve, list, processing transition, payment-methods, stats, auth)
- [x] 5.2 Invoices.spec.ts — 6 tests (Draft→PendingApproval→Approved, detail page, building list)
- [x] 5.3 Convocations.spec.ts — 6 tests (create+list, detail, deadline légale 15j, meeting lookup, auth)
- [x] 5.4 Resolutions.spec.ts — 7 tests (create+retrieve, list, detail page, vote, list votes, close, auth)
- [x] 5.5 Quotes.spec.ts — 7 tests (create+retrieve, list, count, submit Received, comparison page, auth)
- [x] 5.6 TwoFactor.spec.ts — 6 tests (status disabled, setup QR+backup, idempotent, invalid TOTP, auth)

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

#### Jalon 1 — Sécurité & GDPR (7 issues)

| Issue | Titre | Sévérité | Effort estimé | État |
|-------|-------|----------|---------------|------|
| #271 | Quorum 50%+ validation AG (Art 3.87§5) | conformité | ~2h | migration existe, vérifier wiring |
| #272 | 2e convocation si quorum non atteint (Art 3.87§5) | conformité | ~2h | migration existe, vérifier wiring |
| ~~#273~~ | ~~Réduction vote mandataire (Art 3.87§7)~~ | — | — | ✅ FERMÉE 2026-03-21 |
| **#301** | **Permissions rôles : boutons admin visibles syndic** | MAJEUR | ~1h | NEW 22/03 |
| **#302** | **CRITIQUE : Isolation multi-tenant — données non filtrées** | **CRITIQUE** | ~8h | NEW 22/03 |
| **#315** | **[RGPD] Art. 13-14 : Politique de confidentialité** | RGPD | ~4h | NEW 22/03 |
| **#316** | **[RGPD] Art. 28 : DPA sous-traitants** | RGPD | ~2h | NEW 22/03 |
| **#317** | **[RGPD] Art. 33 : Notification violation 72h** | RGPD | ~4h | NEW 22/03 |

#### Jalon 2 — Conformité Légale Belge (7 issues)

| Issue | Titre | Sévérité | Effort estimé | État |
|-------|-------|----------|---------------|------|
| **#303** | **Calcul tantièmes ≠ 1000 millièmes** | MAJEUR | ~4h | NEW 22/03 |
| **#306** | **CRITIQUE : Validation tantièmes >100%** | **CRITIQUE** | ~4h | NEW 22/03 |
| **#310** | **AG : Lien agenda-résolutions — bloquer votes hors ODJ** | conformité | ~4h | NEW 22/03 |
| **#311** | **AG : Quorum 50%+50% et 2ème convocation auto** | conformité | ~4h | NEW 22/03 |
| **#312** | **AG : Procurations — max 3 mandats + exception 10%** | conformité | ~3h | NEW 22/03 |
| **#313** | **AG : Distribution PV 30 jours + génération auto** | conformité | ~6h | NEW 22/03 |
| **#314** | **Syndic : Mandat max 3 ans avec validation** | conformité | ~2h | NEW 22/03 |

#### Bugs UX (4 issues, sans milestone)

| Issue | Titre | Sévérité | Effort estimé | État |
|-------|-------|----------|---------------|------|
| **#304** | **Pages en anglais : Tickets, Announcements, Bookings** | cosmétique | ~2h | NEW 22/03 |
| **#305** | **Bouton créer ticket silencieux si building_id manquant** | MAJEUR | ~2h | NEW 22/03 |
| **#307** | **Sondages/Annonces/Réservations : immeubles non chargés** | MAJEUR | ~2h | NEW 22/03 |
| **#308** | **Label sondages 'Building' au lieu de 'Immeuble'** | cosmétique | ~0.5h | NEW 22/03 |

#### Jalon 3 — Features (9 issues)

| Issue | Titre | Effort estimé | État |
|-------|-------|---------------|------|
| #274 | BC15: AG Visioconférence (AgSession, quorum combiné) | ~4h | backend ✅, BDD ✅, E2E ✅ → manque frontend |
| #275 | BC16: Backoffice prestataires PWA (ContractorReport) | ~2h | backend ✅, BDD ✅, E2E ✅, frontend ✅ → **à fermer ?** |
| **#309** | **Connecter chaîne approbation dépenses (Ticket→Rapport→Dépense)** | ~12h | NEW 22/03 — GAP architectural |
| #276 | BC14: Marketplace corps de métier + satisfaction | ~20h | non implémenté |
| #277 | Guide légal contextuel UI (LegalHelper, AG Wizard) | ~10h | non implémenté |
| #278 | Blog 18 articles RST | ~22h | docs only |
| #279 | BC17: AGE agile (demande 1/5, concertation) | ~4h | backend ✅, BDD ✅, E2E ✅ → manque frontend |
| #280 | Orchestrateur énergie neutre (CER, CREG) | ~16h | non implémenté |
| #300 | IoT MQTT + BOINC Grid Computing | ~4h | backend ✅, BDD ✅ → manque E2E + frontend |

**Effort total restant estimé : ~148h** (features ~80h + bugs legal ~4h + 17 nouvelles issues E2E ~64h)

### Ordre de priorité

1. **P0 Legal** (#271, #272) — bugs conformité, ~4h
2. **P0 BC** (#274, #279) — AG visio + AGE agile, ~8h (manque uniquement frontend)
3. **P0 BC** (#275) — prestataires, ~2h (tout fait, vérifier et fermer)
4. **P1 BC** (#276) — marketplace, ~20h (non implémenté)
5. **P1 Tools** (#277) — guide légal UI, ~10h
6. **P1 Energy** (#280) — orchestrateur, ~16h
7. **P1 IoT** (#300) — MQTT/BOINC E2E + frontend, ~4h
8. **P2 Content** (#278) — blog, ~22h

Après l'implémentation de chaque feature, elle passe dans la matrice de traçabilité (BDD → E2E backend → frontend → Playwright → contract test DTO).

> **Mise à jour** : 15 mars 2026 — MCP (#252-265) et itsme (#48) repoussés hors 0.1.0. Effort réduit de ~145h à ~96h.
> **Mise à jour** : 21 mars 2026 — Audit cohérence WBS ↔ Issues ↔ Code. E2E Backend 41%→96%. #273 fermée. 7 issues re-milestoned. #300 créée. Effort réduit de ~96h à ~84h.
> **Mise à jour** : 22 mars 2026 — Tests E2E manuels (rapport-tests-e2e-koprogo.docx). 17 issues créées :
>   - 8 bugs (#301-#308) : 2 CRITIQUES (multi-tenant #302, tantièmes #306), 4 MAJEURS, 2 cosmétiques
>   - 1 GAP architectural (#309) : chaîne approbation dépenses non connectée
>   - 5 conformité légale (#310-#314) : AG agenda-résolutions, quorum 2e convocation, procurations max 3, PV 30j, mandat syndic 3 ans
>   - 3 RGPD (#315-#317) : politique confidentialité Art.13-14, DPA Art.28, notification violation Art.33
>   - Matrice conformité : 67% (25/37 conforme) → objectif 90% pour v0.1.0
>   - RGPD : 60% (6/10 articles) → objectif 80% pour v0.1.0

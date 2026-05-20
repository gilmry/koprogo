---
feature: refonte-ux-multi-role-acp
phase: prd
phase_togaf: B-C (Business + SI)
agent_bmad: John (Product Manager)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 1.0
status: Draft awaiting human sign-off
brief_source: brief.md (Mary, v1.0 signé 2026-05-20)
total_frs: 45
modules_covered: 8 (Identity+ACP, Property, Governance, Accounting, Community, Maintenance, Portfolio, Cross-cutting)
changelog:
  - "1.0 (2026-05-20) — PRD initial transformant brief v1.0 signé en 45 FRs numérotées avec matrice 4-cat BDD condensée"
---

# PRD — Refonte UX multi-rôle + modèle ACP

## Methode Maury — Phase TOGAF B-C (Business + SI)

> 🟢 **Phase 2 Draft** — Brief Phase 1 signé par @gilmry le 2026-05-20 (v1.0). Ce PRD transforme les 22 capacités + 27 invariants + 19 critères succès du brief en **45 FRs numérotées avec matrice 4-cat BDD Gherkin condensée**.
>
> **GATE Phase 2 = sign-off humain** avant ouverture Phase 3 (Architecture, Winston).

---

## 1. Résumé exécutif

Cette refonte UX/domaine adresse 5 problèmes structurels révélés par la session live testing 2026-05-20 :

1. **Pas de sélecteur d'immeuble global** → erreurs de portée syndic multi-immeubles
2. **Modèle juridique faux** — `Building.organization_id` saute le niveau ACP (Art. 3.84 CC)
3. **RBAC Communauté incohérent** — conflit d'intérêt syndic participant
4. **Ticketing immature** — plaintes/réponses/évaluations Contractor non tracées
5. **Monolithe vs boîte à outils** — pas de modularité par ACP

Solution : **45 FRs** organisées en 8 modules, livrées en 5 slices avec stratégie test-driven 3-niveaux (caractérisation + RED-GREEN-BLUE Vitest + Playwright multi-rôle). Cluster #433 Decimal et epic #555 Result<_, String> coordonnés (mêmes use_cases touchés).

**Verdict Phase 2** : PRD prêt pour signature, GO conditionnel sur acceptation Phase 3 Architecture par Winston.

---

## 2. Objectifs produit (mesurables, traçables vers brief §10)

| # | Objectif | Métrique cible | Échéance | Brief SC |
|---|----------|----------------|----------|----------|
| O1 | Zéro fuite cross-ACP | Test `@security` Playwright VERT | Phase 4 stories | SC1 |
| O2 | Zéro participation syndic en Communauté | Tests `@security` VERTS sur SEL/Poll/Notice/SharedObject | Phase 4 stories | SC2 |
| O3 | Sélecteur immeuble + bannière sur 100% pages contextuelles | Audit Playwright snapshot | Phase 4 stories | SC3-SC4 |
| O4 | Comptable et sous-rôles 403 hors périmètre | Tests `@security` VERTS | Phase 4 stories | SC5-SC9 |
| O5 | Lighthouse a11y ≥ 90 sur pages nouvelles | CI gate axe-core | Phase 4 stories | SC6 |
| O6 | Suite caractérisation reste VERTE sur toutes les slices | CI sur chaque PR | Continu | SC7 |
| O7 | Délégation temporaire + mandats externes bornés `valid_until` | Tests `@happy`+`@negative` | Phase 4 stories | SC8, SC13 |
| O8 | PWA Contractor magic link bout-en-bout | Test E2E mobile Playwright `--device "Pixel 7"` | Phase 4 stories | SC11 |
| O9 | AG hybride quorum agrégé + auth forte + 2 signatures PV | Tests BDD INV-18/19/20 VERTS | Phase 4 stories | SC12 |
| O10 | Ticketing : plaintes documentées + SLA + audit immuable | Tests `@happy`+`@negative`+`@security` | Phase 4 stories | SC14 |
| O11 | Évaluation Contractor refusée sans cahier des charges | Test `@negative` 422 INV-21 | Phase 4 stories | SC15 |
| O12 | Point AGO « évaluation intervenants » auto-généré non retirable | Tests `@happy`+`@security` INV-22 | Phase 4 stories | SC16 |
| O13 | Onboarding modulaire ≤ 5 min ACP « 1 module seul » | Mesure sur 5 utilisateurs test naïfs | Phase 5 validation | SC17 |
| O14 | KPI continu adoption modulaire ≥ 20% ACPs sur 1 seul module | Analytics produit à 3 mois | Post-release | SC18 |
| O15 | Zéro fuite cross-module désactivé | Tests `@security` 403 sur 6 modules | Phase 4 stories | SC19 |
| O16 | Coordination cluster #433 Decimal et epic #555 sans double churn git | Audit PRs (1 use_case touché = 1 PR pour les 2 migrations simultanées) | Phase 6 exécution | — |

---

## 3. Périmètre

### 3.1 In scope (cette refonte)

Tous les modules listés dans le brief §5 (Bounded Contexts) :

- Identity & Access + nouvelle entité ACP + sous-rôles + MagicLink + Mandate
- Property Management (refacto FK + état daté notaire + conformité)
- Governance (CdC + Commissaire + AG hybride + point AGO obligatoire)
- Accounting (split Encodeur/Émetteur + validate-before-compute)
- Community (Moderator rôle syndic)
- Maintenance & Operations (Ticket kind=complaint + SyndicResponse + TechnicalSpec + ContractorEvaluation)
- Portfolio (entité backend + favoris + équipe)
- External Mandates (avocat/notaire/AMO bornés)
- **Modularité** transverse + onboarding + activation/désactivation

### 3.2 Out of scope (cf. brief §9)

- Refonte visuelle/branding
- API publique complète (#111 — Phase ecosystem)
- Tauri Desktop/Mobile (#295-298 — Phase 2)
- Multi-immeubles par ACP en UX riche (cartes/plans) — modèle OK, UX minimale
- Crowdlending #353 — R&D
- Visio intégrée native (redirige vers Jitsi/Whereby externe)

⚠️ **#48 Strong auth voting itsme/eID** : **promu in-scope** par C15 (vote distant AG hybride exige auth forte).

---

## 4. Exigences fonctionnelles (45 FRs)

Convention BDD condensée par FR : `H/E/S/N` = `@happy / @edge / @security / @negative`. Le détail des `Gherkin` complets sera produit en Phase 4 (stories.md) — ici on fixe juste les cas + les bornes.

### 4.1 Module Identity & Access + ACP (FR1-FR8)

| FR | Description | Acteur | Pré-condition | Post-condition | INV brief |
|---|---|---|---|---|---|
| **FR1** | Modélisation `ACP` comme racine d'agrégat distincte (table + entité Rust) | Admin | — | ACP créée avec `organization_id` nullable | INV-1, INV-2 |
| **FR2** | Migration data `Building.organization_id` → `Building.acp_id` + intermédiaire `ACP.organization_id?` | Système | Migration SQL appliquée | Aucun building orphelin, audit migration | INV-1 |
| **FR3** | Endpoints CRUD `/acps` (create/list/get/update/delete) | Admin | RBAC admin | ACP créée/modifiée + audit | INV-1, INV-2 |
| **FR4** | Filtrage role-based des queries `list_buildings`/`list_acps` | Tous rôles | Auth + role identifié | Admin voit tout, syndic ACPs de son cabinet, owner ses ACPs, contractor ses missions | INV-3, INV-7 |
| **FR5** | Sous-rôles métier : `accountant.encodeur` vs `accountant.emetteur` + `community.moderator` | Admin | Role base existant | Permissions différenciées | INV-4, INV-10 |
| **FR6** | Entité `MagicLink` + endpoint `POST /magic-links` + page `GET /c/<token>` | Syndic→Contractor | Ticket/devis créé | Token signé, expirable, single-use ou borné | INV-13, INV-17 |
| **FR7** | Entité `Mandate` (avocat/notaire/AMO) + workflow émission + audit | ACP/Syndic | Mandat décidé | Mandate persistée avec `valid_until` + scope | INV-14 |
| **FR8** | Délégation temporaire syndic→owner mandaté (`UserRoleAssignment.valid_until`) | AG/Syndic | Motif + durée | Rôle attribué temporairement, audit | INV-8 |

**BDD 4-cat condensé (FR1-FR8)** :
- **H** : Admin crée ACP / refacto migration successful / endpoints répondent 200 / magic link consommé / mandate dans validité → OK
- **E** : ACP avec 0 building / ACP avec organization_id=null (auto-gérée) / mandate à exactement `valid_until` → OK
- **S** : Syndic cabinet B tente accès ACP cabinet A → 403 / Contractor tente accès via `/c/<other-token>` → 403 / mandate expiré → 403
- **N** : Création ACP avec organization_id inexistante → 422 / migration sans backup → refuse / magic link expiré → 403 typé

### 4.2 Module Property Management (FR9-FR12)

| FR | Description | INV brief |
|---|---|---|
| **FR9** | `Building.acp_id` FK NOT NULL (post-backfill) + check INV-1 | INV-1 |
| **FR10** | Endpoint `GET /acps/:id/units/:unit_id/etat-date` (Art. 577 CC) avec auth Mandate notaire | INV-15, C12 |
| **FR11** | Fiche immeuble admin/syndic affiche **count réel** units + **somme réelle** quotas + badge conformité + delta | C8 brief, #553 Bugs 1/3/4 |
| **FR12** | `Building.is_conformant() -> bool` (count==total_units && SUM(quota)==1000) | INV-1, mémoire admin-publishes-conform-buildings |

**BDD 4-cat (FR9-FR12)** :
- **H** : Building avec acp_id + 100% conformité → visible, badge vert
- **E** : Building 999/1000 millièmes → non-conformant, pas de tolérance arrondi
- **S** : Notaire demande état daté unit X → OK ; demande unit Y → 403 INV-15
- **N** : Création Building sans acp_id → 422 ; total tantièmes affiche NaN → erreur typée + log + affichage `—`

### 4.3 Module Governance (FR13-FR20)

| FR | Description | INV brief |
|---|---|---|
| **FR13** | `Meeting.mode` enum `in_person/remote/hybrid` | C15 |
| **FR14** | Quorum agrégé `attendees_in_person + remote + proxy` | INV-19 |
| **FR15** | Vote distant authentifié forte (#48 itsme/eID) — refusé sans auth forte | INV-18 |
| **FR16** | 2 signatures électroniques qualifiées eIDAS (président + secrétaire) PV | INV-20 |
| **FR17** | `Meeting.assert_can_complete()` — refuse Completed sans convocations + quorum + résolutions + PV | C-brief, #554 |
| **FR18** | Use-case `generate_ago_resolutions(meeting_id)` ajoute auto Resolution `EvaluationContractors` (non retirable) | INV-22 |
| **FR19** | `CdC` membre élu par AG + action `create_alert` visible AG | INV-12, C13 |
| **FR20** | `CommissaireAuxComptes` lecture full PCMN + signature `commissaire.sign_certificate` | INV-11, C10 |

**BDD 4-cat (FR13-FR20)** :
- **H** : AG hybride avec présentiel + distant + proxy → quorum agrégé correct ; PV signé 2× → clôture OK
- **E** : Quorum à exactement 50.0% → seuil respecté (vs 49.99%→refus)
- **S** : Owner tente vote distant sans itsme/eID → 403 ; syndic tente retirer Resolution `EvaluationContractors` → 403 ; CdC après `valid_until` perd droits → vérif 403
- **N** : Meeting.complete() sans convocation → 422 typé avec liste pieces manquantes ; PV sans 2 signatures → 422

### 4.4 Module Accounting (FR21-FR25)

| FR | Description | INV brief |
|---|---|---|
| **FR21** | Permissions `invoice.create` (Encodeur) vs `expense.issue + call_for_funds.create` (Émetteur) | INV-10 |
| **FR22** | Tout use-case calcul (charges/quorum/répartition/appels de fonds/PV) commence par `building.assert_conformant()?` ; 422 sinon | C-brief, validate-before-compute |
| **FR23** | Total tantièmes calculé via `SUM(units.quota)` côté backend (Decimal-as-string) ; jamais NaN ; fallback `—` typé en FE | #553 Bug 3 |
| **FR24** | Montants reçus du backend = strings Decimal ; aucun `parseFloat`/`Number()` FE ; somme vérifiée Decimal-equivalent | #553 Bug 6, cluster #433 |
| **FR25** | `Commissaire.sign_certificate` → entité `VerificationCertificate` signée numériquement + workflow signature avant clôture comptes annuels | INV-11, C10 |

**BDD 4-cat (FR21-FR25)** :
- **H** : Émetteur crée appel de fonds → OK ; Commissaire signe certificat → persisté + audit
- **E** : Encodeur saisit facture exacte à 0,01€ → arrondi Decimal correct (zéro dérive)
- **S** : Encodeur tente `expense.issue` → 403 INV-10 ; Commissaire tente édit écriture → 403 INV-11
- **N** : Calcul charges sur building non-conform → 422 + détail deltas ; Total tantièmes calculé sans units → renvoie 0 (pas NaN)

### 4.5 Module Community (FR26-FR30)

| FR | Description | INV brief |
|---|---|---|
| **FR26** | Syndic = rôle `Moderator` sur SEL/Poll/Notice/SharedObject (CRUD admin mais pas create/vote/comment perso) | INV-4 |
| **FR27** | Exception : Syndic peut créer `Reservation` si `on_behalf_of_acp = true` (AG, prestataires) | INV-5 |
| **FR28** | Comptable (émetteur ET encodeur) accès `/community/*` → 403 | INV-6 |
| **FR29** | Owner = participant complet Communauté | — |
| **FR30** | CdC = participant copropriétaire normal (pas Moderator) | — |

**BDD 4-cat (FR26-FR30)** :
- **H** : Owner crée annonce/vote sondage/échange SEL → OK ; Syndic modère SEL litigieux → édite/supprime sans participer
- **E** : Réservation `on_behalf_of_acp=true` syndic → autorisée + log spécifique
- **S** : Syndic tente vote sondage personnel → 403 INV-4 ; Comptable accède `/community/sel` → 403 INV-6 ; CdC tente édit SEL voisin → 403
- **N** : Réservation `on_behalf_of_acp=true` sans justification motif → 422

### 4.6 Module Maintenance & Operations (FR31-FR35)

| FR | Description | INV brief |
|---|---|---|
| **FR31** | `Ticket.kind = complaint` avec `evidence_attachments[]` + `witnesses[]` + `incident_date` + `severity` | C17 |
| **FR32** | Entité `SyndicResponse` (responder_user_id, response_text, action_proposed, response_date) + SLA configurable par sévérité + escalade CdC si dépassé | INV-23, C17 |
| **FR33** | Entité `TechnicalSpec` (scope, deliverables, deadlines, criteria, attachments) versionnable, signée ACP/syndic/AMO | C16 |
| **FR34** | Entité `ContractorEvaluation` (contractor_id, technical_spec_id, scores, plaintes_linked[]) ; refuse 422 sans TechnicalSpec préalable | INV-21, C18 |
| **FR35** | Audit immuable sur plaintes/réponses/cahier/évaluations (aucune édition/suppression) | INV-24 |

**BDD 4-cat (FR31-FR35)** :
- **H** : Marie poste plainte avec 3 photos → Sylvie répond <SLA → escalade évitée ; AMO signe TechnicalSpec → Contractor évalué référencé → OK
- **E** : Plainte sévérité critical → SLA 24h appliqué ; ContractorEvaluation pile à expiration TechnicalSpec → autorisée
- **S** : Marie tente édit plainte 5min après création → 403 INV-24 ; ContractorEvaluation sans TechnicalSpec → 422 INV-21
- **N** : SLA dépassé sans response → audit + notification CdC + escalade visible AGO ; Évaluation Contractor inexistant → 404

### 4.7 Module Portfolio (FR36-FR38)

| FR | Description | INV brief |
|---|---|---|
| **FR36** | Entité `Portfolio` backend (table + relations user/organization/buildings) + favoris star + partage équipe | C-brief, mémoire-#553 |
| **FR37** | Sélecteur immeuble UI (dropdown + autocomplete + favoris star + portefeuilles équipe) en haut à gauche, conditionné par rôle | C1 |
| **FR38** | Bannière contextuelle 3-niveaux `Immeuble · ACP · Cabinet syndic` quand building sélectionné | C1 |

**BDD 4-cat (FR36-FR38)** :
- **H** : Sylvie sélectionne immeuble → menus se contextualisent → bannière 3-niveaux exacte ; ajoute en favori star ; crée portfolio équipe partagé
- **E** : Cabinet avec 100 ACPs → autocomplete fluide (perf < 200ms) ; portfolio vide → message "ajoutez vos immeubles préférés"
- **S** : Gestionnaire cabinet B tente accès Portfolio cabinet A → 403 INV-9
- **N** : Sélection building inexistant → 404 + reset sélecteur

### 4.8 Cross-cutting Modularité (FR39-FR42)

| FR | Description | INV brief |
|---|---|---|
| **FR39** | Table `acp_enabled_modules` + UI conditionnelle (menus cachés si désactivé) + API 403/404 `ModuleDisabledError` | C20, INV-25 |
| **FR40** | Onboarding modulaire ≤ 5 min (assistant guidé : profil ACP → recommandations → activation + démo) | C22, SC17 |
| **FR41** | Activation/désactivation modules auditée : admin SaaS pour Community/Ticketing/Maintenance/Portfolio ; vote AG pour Accounting/Governance | INV-26, C20 |
| **FR42** | Désactivation = archivage (flag `archived_at`), jamais suppression ; délai légal respecté (5 ans Compta, 10 ans AG) ; réactivation = restauration | INV-27 |

**BDD 4-cat (FR39-FR42)** :
- **H** : Admin active module Communauté pour ACP X → menu Communauté apparaît côté syndic ; onboarding 5 modules en 4min23 → KPI OK
- **E** : Module activé puis désactivé puis réactivé → données restaurées intactes ; activation Accounting via vote AG ≥ 50% → autorisée
- **S** : Syndic ACP avec Compta désactivée tente accès `/expenses` → 403 ModuleDisabledError ; admin tente désactiver Accounting sans vote AG → 403 INV-26
- **N** : Désactivation module avec dépendances actives (ex: AG planifiée si Governance désactivé) → 422 + message clair

### 4.9 Tests & Quality (FR43-FR45)

| FR | Description | Mémoire |
|---|---|---|
| **FR43** | Suite `frontend/tests/e2e/characterization/` créée AVANT toute slice de refonte (flows existants à figer) | fe-refactor-test-driven |
| **FR44** | Tests Playwright multi-rôle utilisent uniquement helpers shared (`loginAsSyndic[WithBuilding]`) — zéro helper local UI-login | multirole-narrative-scenarios, #550 |
| **FR45** | Tous composants nouveaux/refactorés respectent WCAG 2.1 AA + `data-testid="<entity>-<action>"` systématique ; CI gate axe-core | a11y-wcag-aa-baseline, data-testid-systematic |

**BDD 4-cat (FR43-FR45)** :
- **H** : Suite caractérisation 100% verte sur HEAD pré-refonte ; Playwright login admin → action → assertion en multi-rôle OK
- **E** : Composant border focus visible au keyboard tab seul (a11y) ; axe-core score ≥ 90
- **S** : Test caractérisation casse à mi-slice → arrêt immédiat refonte + investigation
- **N** : Bouton sans data-testid détecté en PR → CI fail (gate dur) ; sélecteur Playwright `nth-child` détecté → warning

---

## 5. Exigences non-fonctionnelles

| NFR | Cible | Vérifiable |
|---|---|---|
| **NFR1 — Perf API** | P99 < 500ms (cf. CLAUDE.md cible v0.1.0) | Tests charge + monitoring |
| **NFR2 — A11y** | WCAG 2.1 AA + Lighthouse a11y ≥ 90 sur pages nouvelles | CI gate `axe-core` + `eslint-plugin-jsx-a11y` |
| **NFR3 — Sécu** | Aucune fuite cross-ACP / cross-module / cross-mandate ; audit immuable | Tests `@security` BDD 4-cat |
| **NFR4 — i18n** | FR/NL/EN/DE tous nouveaux strings traduits | CI check `i18n missing keys` |
| **NFR5 — Compat browsers** | Firefox/Chrome/Safari (2 dernières versions) + mobile (Pixel 7 / iPhone 14) | Playwright multi-browser matrix |
| **NFR6 — Decimal** | Tout montant = `rust_decimal::Decimal` BE + string JSON FE | CI grep `f64\\|Number(amount)` → fail |
| **NFR7 — Result typed** | Aucun `Result<_, String>` ajouté dans use-cases touchés | CI grep + hook PreToolUse |

---

## 6. Stratégie tests caractérisation (Niveau 1 fe-refactor-test-driven)

Suite `frontend/tests/e2e/characterization/` à créer en sous-slice initiale, avant toute refonte applicative. Cible : capturer le **comportement existant qui doit rester** :

| Flow caractérisé | Spec à créer |
|---|---|
| Login admin/syndic/owner + dashboard initial | `00-login-and-dashboards.spec.ts` |
| Création immeuble admin → assignation Organization → visible syndic | `01-building-creation-flow.spec.ts` |
| Création AG syndic → convocations → vote → clôture | `02-ag-full-cycle.spec.ts` |
| Création expense + appel de fonds + paiement | `03-expense-and-payment.spec.ts` |
| Vue copropriétaire de ses lots + ses votes | `04-owner-view.spec.ts` |
| Notification bell + sync | `05-notifications-sync.spec.ts` |

**Critère gate** : 100% caractérisation VERT en `feature/dev` HEAD avant tout commit de la slice 1 de refonte. Toute régression = arrêt immédiat.

---

## 7. Dépendances cross-stories

| Dépendance | Coordination |
|---|---|
| **Cluster #433 Decimal (umbrella)** | Toute FR touchant un use_case du cluster fait simultanément la migration Decimal (1 PR par fichier = 2 migrations). FR22-25, FR12, FR14 directement concernées. |
| **Epic #555 Result<_, String>** | Idem : toute FR touchant un use_case migre vers `AppError`. FR17 (Meeting.complete) déjà tracée par #554 hors-scope, à reprendre ici. |
| **#553 Building admin** | FR9, FR11, FR12 résolvent les bugs identifiés (#553 Bugs 1-6) |
| **#554 AG state machine + world-model seed** | FR17, FR18 résolvent les bugs #554 |
| **#550 Playwright stratification** | FR44 prolonge la solution (zéro helper local) |
| **#552 work-reports 400** | Pas dans le scope direct, mais Contract Types Check CI doit rester VERT |
| **#48 itsme/eID** | FR15 le promeut in-scope (vote distant AG hybride) |

---

## 8. Plan de release (slices déployables)

**5 slices** ordonnées par dépendances + criticité légale :

| # | Slice | Modules touchés | FRs | Effort | Critère go |
|---|---|---|---|---|---|
| **0** | Caractérisation FE | Tests only | FR43, FR44 | M | Suite 100% verte pré-refonte |
| **1** | Refacto domaine ACP + migration data + endpoints | Identity+Property | FR1-FR4, FR9 | L | Aucune fuite cross-ACP (test `@security`), 0 building orphelin migration |
| **2** | Sélecteur global + bannière + menus contextuels + Portfolio | Portfolio + transverse UI | FR36-FR38, FR4 (UI) | M | Sélecteur sur 100% pages contextuelles, bannière 3-niveaux exacte |
| **3** | Sous-rôles métier + Magic Link Contractor + PWA + Mandates | Identity+Access + Maintenance | FR5-FR8, FR31-FR35 | L | Encodeur/Émetteur split fonctionnel, magic link Contractor bout-en-bout, mandats bornés |
| **4** | Governance hybride + Commissaire + CdC + point AGO + signatures eIDAS | Governance + Accounting | FR13-FR20, FR22, FR25 | L | AG hybride quorum agrégé OK, point AGO auto-généré, signatures électroniques fonctionnelles |
| **5** | Modularité + onboarding + RBAC Communauté Moderator | Community + Cross-cutting | FR26-FR30, FR39-FR42, FR45 | M | Modules activables/désactivables, syndic Moderator sans participation, onboarding ≤5min |

Chaque slice = 1 ou plusieurs PRs avec gate CI VERT + tests caractérisation VERTS + nouveaux tests 4-cat.

---

## 9. Risques + mitigation

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Migration data ACP casse buildings existants | Moyenne | Critique | Backup obligatoire + dry-run + script de rollback + tests `@negative` sur migration |
| Coordination #433/#555 ratée → double churn git | Haute | Moyen | Convention « 1 PR par use_case = 2 migrations simultanées » + meta-comment #433 |
| Signature électronique eIDAS coûteuse | Moyenne | Moyen | ADR Phase 3 sur prestataire (eID belge gratuit pour citoyens vs Universign/DocuSign EU payant) |
| Performance sélecteur immeuble avec 100+ ACPs | Faible | Moyen | Pagination autocomplete + index DB + cache LRU côté FE |
| PWA Contractor pas adoptée (réticence artisans) | Moyenne | Faible | UX ultra-simplifiée (3 écrans max) + onboarding par SMS + feedback usage |
| Cluster Decimal #433 traîne → FR22-25 bloquées | Moyenne | Élevé | Démarrer Slice 0+1 en parallèle de #433, escalade humain si #433 retard >2 semaines |
| Adoption modulaire faible → SC18 raté | Faible | Faible | Analytics dès release + plan d'ajustement messaging si KPI < 20% à 3 mois |

---

## 10. Gate de validation Phase 2 — sign-off humain

Sign-off @gilmry requis avant ouverture Phase 3 (Architecture, Winston) :

- [ ] Objectifs O1-O16 mesurables et traçables vers brief
- [ ] Périmètre In/Out scope validé (incl. #48 promu in-scope)
- [ ] 45 FRs validées (numérotation + matrice 4-cat condensée)
- [ ] NFRs validées (perf P99 < 500ms, WCAG AA, i18n, Decimal/Result typed)
- [ ] Stratégie tests caractérisation validée (6 specs prioritaires)
- [ ] Dépendances cross-stories (#433/#555/#553/#554/#550/#48) acceptées
- [ ] Plan de release 5 slices validé (ordre + critères go par slice)
- [ ] Risques 7 + mitigations acceptés
- [ ] Coordination Phase 3 (Architecture) → ADRs à produire identifiés :
  - ADR ACP comme racine d'agrégat
  - ADR Portefeuille entité backend
  - ADR Convention data-testid
  - ADR Caractérisation FE arborescence
  - ADR Signature électronique eIDAS prestataire
  - ADR Modularité par ACP (table + module registry)

**Date signature** : _à compléter_
**Signature** : _@gilmry_

---

## 11. Phase suivante

Phase 3 (Architecture — Winston) **bloquée tant que Phase 2 non signée**. Au sign-off : Winston produit `architecture.md` avec diagrammes d'agrégat, ports/adapters, ADRs ci-dessus, migrations SQL, impact frontend détaillé.

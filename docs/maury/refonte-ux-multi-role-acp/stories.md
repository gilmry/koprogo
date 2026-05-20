---
feature: refonte-ux-multi-role-acp
phase: stories
phase_togaf: E (Opportunities & Solutions)
agent_bmad: Bob (Scrum Master)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 1.0
status: Signed by @gilmry 2026-05-20
signed_at: 2026-05-20
signed_by: "@gilmry"
brief_source: brief.md (Mary, v1.0 signé 2026-05-20)
prd_source: prd.md (John, v1.0 signé 2026-05-20)
architecture_source: architecture.md (Winston, v1.0 signé 2026-05-20)
total_stories: 31
total_slices: 6
changelog:
  - "1.0 (2026-05-20) — SIGNÉES par @gilmry. Phase 4 verrouillée, Phase 5 (Validation PO) débloquée. 31 stories en 6 slices (incl. slice 0 caractérisation + slice transversal), 1 story = 1 PR cible, AC 4-cat Gherkin condensé, data-testid listés, files à toucher, dépendances inter-stories, coordination #433/#555 par story."
---

# Stories — Refonte UX multi-rôle + modèle ACP

## Méthode Maury — Phase TOGAF E (Opportunities & Solutions)

> ✅ **Stories SIGNÉES par @gilmry le 2026-05-20** — Phase 4 verrouillée, Phase 5 (Validation PO) débloquée. 31 stories en 6 slices. Voir [`validation.md`](validation.md) pour le sign-off PO et la priorisation finale.

---

## 1. Vue d'ensemble

### 1.1 Découpage par slice

| Slice | Nom | Stories | FRs PRD | Effort cumulé |
|---|---|---|---|---|
| **0** | Caractérisation FE (régression safety net) | 1 | FR43, FR44 | M |
| **1** | Refacto domaine ACP + migration data + conformité | 4 | FR1-FR4, FR9-FR12 | L |
| **2** | Sélecteur global + bannière + Portfolio + #553 fix | 5 | FR4 (UI), FR11, FR36-FR38 | M |
| **3** | Sous-rôles + Magic Link + PWA + Mandates + Ticketing | 9 | FR5-FR8, FR31-FR35 | L |
| **4** | Governance hybride + Commissaire + CdC + signatures eIDAS | 9 | FR13-FR20, FR22-FR25 | L |
| **5** | Modularité + onboarding + RBAC Communauté Moderator | 8 | FR26-FR30, FR39-FR42, FR45 | M |
| **Tx** | Transversal continu (FR43-FR45 enforcement) | 3 | FR45 + observabilité | M |

**Total** : **31 stories** en **6 slices** (5 fonctionnelles + 1 transversal continu + slice 0 caractérisation).

### 1.2 Convention de naming

`<slice>.<n>-<entity>-<action>` — ex `1.1-acp-domain-entity`, `4.5-meeting-assert-can-complete`. Branch git : `story/<slice>.<n>-<entity>-<action>`.

### 1.3 Règle 1 story = 1 PR (sauf exception annotée)

Toute story produit **1 PR atomique** mergeable seule. Si une story coordonne 2 migrations simultanées (cf. cluster #433 Decimal + epic #555 Result), elle reste **1 PR** mais marquée `[cluster-coord]` dans le titre.

### 1.4 Template story

Chaque story expose :
- **Goal** — phrase d'intention (verbe d'action)
- **FR/INV** — PRD FRs couvertes + brief INVs vérifiées
- **Effort** — S (≤1j) / M (1-3j) / L (3-5j)
- **Deps** — stories prérequises (avant-merge)
- **AC 4-cat** — `@happy` / `@edge` / `@security` / `@negative` Gherkin condensé
- **data-testid** — liste éléments interactifs à exposer
- **Files** — fichiers backend + frontend à toucher
- **ADR refs** — ADR-001N inline architecture
- **Cluster coord** — #433 Decimal + #555 Result + autres

### 1.5 Gate par slice

Pas de slice N+1 ouverte sans :
- Toutes stories slice N mergées sur `feature/dev`
- Caractérisation FE 100% VERT (slice 0 immuable)
- Critère go slice N (cf. PRD §8) respecté
- Sign-off humain @gilmry sur slice complétée

---

## 2. Slice 0 — Caractérisation FE (régression safety net)

### Story 0.1 — Suite caractérisation 6 specs (gel comportement existant)

- **Goal** : Créer 6 specs Playwright dans `frontend/tests/e2e/characterization/` qui figent les flows existants HEAD `feature/dev` avant toute refonte applicative. Ces specs DOIVENT rester VERTES sur toutes les slices ultérieures.
- **FR/INV** : FR43, FR44 ; pas d'INV (régression safety net)
- **Effort** : M
- **Deps** : aucune (story d'entrée Maury Phase 4)
- **AC 4-cat** :
  - `@happy` : login admin/syndic/owner + dashboard initial chacun → screenshots stables, durée < 30s
  - `@edge` : flow building creation admin → assignation organization → visible syndic → 100% VERT
  - `@security` : aucune (pas de nouveau comportement testé, juste fige)
  - `@negative` : si une spec caractérisation tourne ROUGE sur HEAD pré-refonte → STOP. Le test est bugué ou bien le HEAD est déjà cassé. Investigation avant slice 1.
- **data-testid** : aucun ajouté ici (specs utilisent sélecteurs existants `getByText`/`role=` — accepté pour caractérisation, interdit dans `refonte-ux/`)
- **Files** :
  - `frontend/tests/e2e/characterization/00-login-and-dashboards.spec.ts`
  - `frontend/tests/e2e/characterization/01-building-creation-flow.spec.ts`
  - `frontend/tests/e2e/characterization/02-ag-full-cycle.spec.ts`
  - `frontend/tests/e2e/characterization/03-expense-and-payment.spec.ts`
  - `frontend/tests/e2e/characterization/04-owner-view.spec.ts`
  - `frontend/tests/e2e/characterization/05-notifications-sync.spec.ts`
  - `frontend/tests/e2e/helpers/auth.ts` (loginAsSyndic[WithBuilding], loginAsAdmin, loginAsOwner — réutilisation #550)
  - `playwright.config.ts` (project `characterization` runner séparé)
  - `package.json` (script `test:characterization`)
- **ADR refs** : ADR-0013 (arborescence)
- **Cluster coord** : —
- **Mémoires** : [[fe-refactor-test-driven]] niveau 1, [[multirole-narrative-scenarios]], [[bdd-seed-dates-relative]]

---

## 3. Slice 1 — Refacto domaine ACP + migration data + conformité

### Story 1.1 — Entité ACP backend + CRUD `/acps`

- **Goal** : Créer l'entité `Acp` (domain) + port `AcpRepository` + use-cases CRUD + adapter PostgreSQL + handlers Actix + migration SQL `create_acps`.
- **FR/INV** : FR1, FR3 ; INV-1, INV-2
- **Effort** : L
- **Deps** : 0.1 (caractérisation verte)
- **AC 4-cat** :
  - `@happy` : Admin POST `/acps {name, address, organization_id?}` → 201 + Acp persistée + audit ; GET `/acps` filtré rôle → admin voit toutes, syndic voit celles de son cabinet
  - `@edge` : ACP avec `organization_id = null` (ACP auto-gérée) → autorisée ; ACP avec 0 building lié → autorisée
  - `@security` : Syndic cabinet B tente accès ACP cabinet A → 403 `AcpNotInScope` ; user non-admin tente POST `/acps` → 403
  - `@negative` : POST avec `organization_id` inexistante → 422 ; PUT sur ACP inexistante → 404 typé
- **data-testid** : `acp-create-submit`, `acp-list-row-{{id}}`, `acp-edit-submit`
- **Files** :
  - `backend/src/domain/entities/acp.rs`
  - `backend/src/application/ports/acp_repository.rs`
  - `backend/src/application/use_cases/acp_use_cases.rs`
  - `backend/src/application/dto/acp_dto.rs`
  - `backend/src/infrastructure/database/repositories/acp_repository_impl.rs`
  - `backend/src/infrastructure/web/handlers/acp_handlers.rs`
  - `backend/migrations/20260601_010000_create_acps.sql` + DOWN
  - `backend/tests/integration/acp_test.rs`
  - `backend/tests/features/acp.feature` (BDD 4-cat)
- **ADR refs** : ADR-0010 (ACP racine d'agrégat)
- **Cluster coord** : NEW use-case → AppError natif, pas de #555 dette ; pas de #433 ici (pas de Decimal monétaire dans ACP)

### Story 1.2 — Migration data `buildings.organization_id → acp_id` (3 étapes)

- **Goal** : Migration data en 3 étapes (NULLABLE → backfill → NOT NULL) avec script de rollback. Pour chaque organization existante : créer ACP miroir, backfill `buildings.acp_id`, supprimer `organization_id`.
- **FR/INV** : FR2, FR9 ; INV-1
- **Effort** : M
- **Deps** : 1.1 (entité Acp existe)
- **AC 4-cat** :
  - `@happy` : Migration appliquée sur DB de dev (≥1 organization avec ≥1 building) → 0 building orphelin + audit_event créé par ACP miroir
  - `@edge` : Organization sans building → ACP miroir créée mais reste sans building (pas d'orphelin)
  - `@security` : Migration nécessite backup explicite (variable `BACKUP_CONFIRMED=true`) ; sinon refuse
  - `@negative` : Migration en 3 étapes interrompue après étape 2 → rollback automatique restaure schema initial sans perte data
- **data-testid** : — (migration backend pure)
- **Files** :
  - `backend/migrations/20260601_020000_add_buildings_acp_id.sql` (NULLABLE)
  - `backend/migrations/20260601_030000_backfill_buildings_acp_id.sql` (data)
  - `backend/migrations/20260601_040000_buildings_acp_id_not_null.sql` (ALTER + DROP organization_id)
  - `backend/migrations/20260601_020000_DOWN.sql`, `20260601_030000_DOWN.sql`, `20260601_040000_DOWN.sql`
  - `backend/tests/integration/migration_acp_backfill_test.rs` (testcontainers, valide aller-retour)
- **ADR refs** : ADR-0010
- **Cluster coord** : —

### Story 1.3 — Filtrage role-based `list_buildings` + `list_acps`

- **Goal** : Adapter use-cases liste pour filtrer par scope (admin tout, syndic cabinet, owner ses ACPs, contractor via MagicLink). Introduit `ListScope` enum.
- **FR/INV** : FR4 ; INV-3, INV-7
- **Effort** : M
- **Deps** : 1.1, 1.2
- **AC 4-cat** :
  - `@happy` : Admin GET `/buildings` → tout ; Syndic GET → ACPs de son organization seulement ; Owner GET → ACPs où user a UserRoleAssignment owner
  - `@edge` : User multi-rôle (admin ET syndic A) → admin domine (voit tout)
  - `@security` : Syndic cabinet B forge query param `acp_id=cabinet_A` → 403 `AcpNotInScope`
  - `@negative` : User non-auth tente GET `/buildings` → 401 ; query sans scope_id → 400
- **data-testid** : — (filtre transparent côté backend)
- **Files** :
  - `backend/src/application/use_cases/list_buildings_use_case.rs` (refacto)
  - `backend/src/application/use_cases/list_acps_use_case.rs` (NEW)
  - `backend/src/infrastructure/web/middleware/scope_guard.rs` (NEW)
  - `backend/tests/features/list_buildings_role_based.feature`
- **ADR refs** : ADR-0010
- **Cluster coord** : si use-case touche Decimal/Result legacy → migrer dans la même PR (audit pré-PR)

### Story 1.4 — `Building.is_conformant()` + FR11 fiche immeuble correcte

- **Goal** : Méthode domain `Building.is_conformant() -> bool` (count_units==total_units && SUM(quotas)==Decimal(1000)). Fiche immeuble admin/syndic affiche count réel + somme réelle + badge conformité + delta. Résout #553 Bugs 1/3/4.
- **FR/INV** : FR9, FR11, FR12 ; INV-1, mémoire [[admin-publishes-conform-buildings]]
- **Effort** : M
- **Deps** : 1.1, 1.2
- **AC 4-cat** :
  - `@happy` : Building 50/50 units + SUM quotas == 1000 → `is_conformant() == true` + badge vert UI
  - `@edge` : Building 999/1000 millièmes (1 millième manquant) → non-conformant ; pas de tolérance arrondi (cluster #433 Decimal strict)
  - `@security` : Syndic ne peut publier (rendre visible) un building non-conformant ; admin garde la conformité (mémoire admin-publishes-conform-buildings)
  - `@negative` : count_units==0 → fiche affiche "—" et non NaN (#553 Bug 3) ; pas de panic ni unwrap()
- **data-testid** : `building-conformity-badge`, `building-units-count`, `building-quota-sum`, `building-quota-delta`, `building-edit-submit`
- **Files** :
  - `backend/src/domain/entities/building.rs` (ajout `is_conformant`, `units_count`, `quota_sum`)
  - `backend/src/application/dto/building_dto.rs` (sérialise count réel + sum Decimal-as-string)
  - `frontend/src/lib/components/buildings/ConformityBadge.svelte` (NEW)
  - `frontend/src/lib/components/buildings/BuildingDetail.svelte` (refacto)
  - `frontend/src/lib/components/buildings/__tests__/ConformityBadge.test.ts` (Vitest RED-GREEN-BLUE)
  - `frontend/tests/e2e/refonte-ux/slice-1-acp-refacto/building-conformity.spec.ts`
- **ADR refs** : ADR-0010, ADR-0012 (data-testid)
- **Cluster coord** : **#433 simultané** (quotas Decimal, SUM Decimal, plus de `Number()` ni `parseFloat` côté FE)

---

## 4. Slice 2 — Sélecteur global + bannière + Portfolio + #553 fix

### Story 2.1 — Entité `Portfolio` backend + CRUD `/portfolios`

- **Goal** : Tables `portfolios` + `portfolio_buildings` + `portfolio_shares` + entité domain + use-cases + handlers + migration SQL.
- **FR/INV** : FR36 ; mémoire [[koprogo-modular-toolbox]] (favoris star)
- **Effort** : M
- **Deps** : 1.1 (acp_id existe)
- **AC 4-cat** :
  - `@happy` : Syndic crée portfolio "Mes immeubles favoris" → ajoute 3 buildings (1 star, 2 normaux) → list buildings dans portfolio retourne 3, star d'abord
  - `@edge` : Portfolio vide (0 building) → autorisé, retourne `[]`
  - `@security` : Gestionnaire cabinet B tente accès portfolio cabinet A → 403 `AcpNotInScope` ; user non partagé tente GET portfolio partagé d'autre user → 403
  - `@negative` : POST portfolio sans `name` → 422 ; ajout building inexistant → 404 typé
- **data-testid** : `portfolio-create-submit`, `portfolio-add-building`, `portfolio-share-submit`, `portfolio-toggle-favorite-{{id}}`
- **Files** :
  - `backend/src/domain/entities/portfolio.rs`
  - `backend/src/application/ports/portfolio_repository.rs`
  - `backend/src/application/use_cases/portfolio_use_cases.rs`
  - `backend/src/infrastructure/database/repositories/portfolio_repository_impl.rs`
  - `backend/src/infrastructure/web/handlers/portfolio_handlers.rs`
  - `backend/migrations/20260601_050000_create_portfolios.sql` + DOWN
  - `backend/tests/features/portfolio.feature`
- **ADR refs** : ADR-0011 (Portefeuille entité backend)
- **Cluster coord** : NEW → AppError natif

### Story 2.2 — Composant `BuildingSelector.svelte` + store scope

- **Goal** : Composant global (top-left layout) avec dropdown + autocomplete + favoris star + portefeuilles équipe. Conditionné par rôle (visible si admin/syndic/accountant.*). Store `scope` réactif (selectedBuildingId/AcpId/PortfolioId).
- **FR/INV** : FR37 ; brief C1
- **Effort** : M
- **Deps** : 2.1
- **AC 4-cat** :
  - `@happy` : Syndic ouvre selector → typing "immeu" → autocomplete 3 résultats < 200ms → click building → store mis à jour → menus contextualisés
  - `@edge` : Cabinet avec 100 ACPs et 500 buildings → autocomplete reste < 200ms (debounce 150ms + pagination 20)
  - `@security` : Owner ne voit pas le selector (RBAC role-based render) ; building cliqué hors scope → 403 + reset selector
  - `@negative` : Aucun building → message "Aucun immeuble dans votre périmètre" + lien vers admin si syndic
- **data-testid** : `building-selector-input`, `building-selector-result-{{id}}`, `building-selector-favorite-{{id}}`, `building-selector-clear`
- **Files** :
  - `frontend/src/lib/components/global/BuildingSelector.svelte` (NEW)
  - `frontend/src/stores/scope.svelte.ts` (NEW, Svelte 5 runes)
  - `frontend/src/lib/api/buildings.ts` (extension search endpoint)
  - `frontend/src/lib/components/global/__tests__/BuildingSelector.test.ts` (Vitest)
  - `frontend/src/layouts/AppLayout.astro` (intégration top-left)
- **ADR refs** : ADR-0011, ADR-0012
- **Cluster coord** : —

### Story 2.3 — Composant `ContextBanner.svelte` (bannière 3-niveaux)

- **Goal** : Bannière contextuelle `Cabinet · ACP · Immeuble` quand building sélectionné. Couleur conformité (vert/orange/rouge selon `is_conformant`).
- **FR/INV** : FR38 ; brief C1, INV-1
- **Effort** : S
- **Deps** : 1.4 (is_conformant), 2.2 (store scope)
- **AC 4-cat** :
  - `@happy` : Building sélectionné conformant → bannière verte avec 3 niveaux `Cabinet Maury · ACP Résidence X · Immeuble A`
  - `@edge` : ACP auto-gérée (organization_id=null) → bannière 2 niveaux `ACP · Immeuble` (cabinet absent)
  - `@security` : Bannière respecte filtrage rôle (un syndic ne voit pas le cabinet d'un autre)
  - `@negative` : Aucun building sélectionné → bannière masquée (pas placeholder vide)
- **data-testid** : `context-banner`, `context-banner-cabinet`, `context-banner-acp`, `context-banner-building`, `context-banner-conformity-icon`
- **Files** :
  - `frontend/src/lib/components/global/ContextBanner.svelte` (NEW)
  - `frontend/src/lib/components/global/__tests__/ContextBanner.test.ts`
  - `frontend/src/layouts/AppLayout.astro` (intégration sous header)
- **ADR refs** : ADR-0012
- **Cluster coord** : —

### Story 2.4 — Refacto `Navigation.svelte` (menus conditionnels rôle + sélection)

- **Goal** : Navigation latérale conditionnée par rôle ET sélection ACP/Building. 5 menus principaux : Gestion, Compta, Gouvernance, Communauté, Ticketing. Sous-menus collapsibles.
- **FR/INV** : FR4 (UI) ; brief C1, C2
- **Effort** : M
- **Deps** : 2.2, 2.3
- **AC 4-cat** :
  - `@happy` : Syndic + building sélectionné → 5 menus visibles ; Owner → Communauté + Mes lots visibles seulement
  - `@edge` : Admin sans building sélectionné → menus génériques `/admin/*` ; sélection efface menus admin (mode "in-context")
  - `@security` : Accountant.encodeur n'a pas de menu Communauté ; clic forcé URL `/community` → 403 + redirect
  - `@negative` : User authentifié sans aucun UserRoleAssignment → écran "Aucun rôle attribué — contactez votre administrateur"
- **data-testid** : `navigation-menu-gestion`, `navigation-menu-compta`, `navigation-menu-gouvernance`, `navigation-menu-communaute`, `navigation-menu-ticketing`, `navigation-submenu-{{key}}`
- **Files** :
  - `frontend/src/lib/components/navigation/Navigation.svelte` (refacto major)
  - `frontend/src/lib/components/navigation/RoleSubmenu.svelte` (NEW)
  - `frontend/src/lib/auth/permissions.ts` (helpers `canSee(role, menu, scope)`)
  - `frontend/src/lib/components/navigation/__tests__/Navigation.test.ts`
- **ADR refs** : ADR-0012
- **Cluster coord** : —

### Story 2.5 — E2E refonte-ux slice 2 (multi-rôle narratif)

- **Goal** : Spec Playwright slice 2 multi-rôle : admin crée ACP+building → syndic se logue → sélectionne building → bannière 3 niveaux exacte + menus contextualisés → owner se logue → menus restreints + pas de selector.
- **FR/INV** : FR4, FR11, FR36-FR38 + FR44 (helpers shared)
- **Effort** : S
- **Deps** : 1.4, 2.2, 2.3, 2.4
- **AC 4-cat** :
  - `@happy` : Admin login → POST /acps → POST /buildings → logout. Syndic login → selector OK → banner OK → menus 5 OK
  - `@edge` : Bascule selector building A → building B → menus restent stables (pas de reflow > 100ms)
  - `@security` : Syndic cabinet B login → tente accès URL building cabinet A → 403 + redirect
  - `@negative` : Building non-conformant invisible côté syndic mais visible côté admin
- **data-testid** : utilise ceux des stories 2.1-2.4
- **Files** :
  - `frontend/tests/e2e/refonte-ux/slice-2-selector-banner/admin-creates-syndic-selects.spec.ts`
- **ADR refs** : ADR-0013 (arborescence refonte-ux)
- **Cluster coord** : —

---

## 5. Slice 3 — Sous-rôles + Magic Link + PWA + Mandates + Ticketing

### Story 3.1 — Sous-rôles métier (accountant.{encodeur,emetteur} + community.moderator + autres)

- **Goal** : Extension `UserRoleAssignment.role` enum : ajouter `accountant.encodeur`, `accountant.emetteur`, `community.moderator`, `lawyer`, `notary`, `amo`, `architect`, `bet`, `warden`. Refacto permission checks pour distinguer encodeur vs émetteur (FR21).
- **FR/INV** : FR5, FR21 ; INV-4, INV-10
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : Accountant.encodeur peut créer Invoice ; Accountant.emetteur peut créer Expense + CallForFunds
  - `@edge` : User cumule encodeur ET émetteur → tous droits réunis (union)
  - `@security` : Accountant.encodeur tente POST `/expenses` → 403 INV-10 ; encodeur tente POST `/call-for-funds` → 403
  - `@negative` : Assignment avec role inconnu → 422 ; clean-up role string trim+lowercase
- **data-testid** : — (RBAC backend)
- **Files** :
  - `backend/migrations/20260615_010000_split_accountant_roles.sql` (note : pas de schema change si role VARCHAR, juste seed + enum Rust)
  - `backend/src/domain/value_objects/role.rs` (extension enum)
  - `backend/src/application/use_cases/invoice_use_cases.rs` (NEW — Encodeur)
  - `backend/src/application/use_cases/expense_use_cases.rs` (refacto permission)
  - `backend/src/application/use_cases/call_for_funds_use_cases.rs` (refacto permission)
  - `backend/tests/features/accountant_subroles.feature`
- **ADR refs** : —
- **Cluster coord** : **#433 simultané** sur expense/call_for_funds (Decimal monétaire) ; **#555 simultané** si use-cases touchés ont des `Result<_, String>` legacy

### Story 3.2 — Entité `MagicLink` + endpoint + page publique `/c/<token>`

- **Goal** : Table `magic_links` + entité + use-cases issue/validate_and_consume + endpoint `POST /magic-links` (syndic) + page Astro publique `/c/[token]` qui résout le scope (ticket/quote/invoice/evaluation).
- **FR/INV** : FR6 ; INV-13, INV-17
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : Syndic POST `/magic-links {subject_user_id, scope_kind=ticket, scope_id, expires_in=7d}` → token signé → contractor ouvre `/c/<token>` → voit le ticket
  - `@edge` : Token à exactement `expires_at` (1 seconde près) → autorisé ; consommation single_use seconde fois → 403
  - `@security` : Token forgé/altéré (HMAC invalide) → 403 `MagicLinkInvalid` ; tentative `/c/<token-other-scope>` → 403
  - `@negative` : Token expiré → 403 typé avec message "lien expiré, demandez-en un nouveau au syndic"
- **data-testid** : `magic-link-issue-submit`, `magic-link-target-input`, `c-page-ticket-content`, `c-page-respond-submit`
- **Files** :
  - `backend/src/domain/entities/magic_link.rs`
  - `backend/src/application/ports/magic_link_repository.rs`
  - `backend/src/application/use_cases/magic_link_use_cases.rs`
  - `backend/src/infrastructure/database/repositories/magic_link_repository_impl.rs`
  - `backend/src/infrastructure/web/handlers/magic_link_handlers.rs`
  - `backend/migrations/20260605_010000_create_magic_links.sql` + DOWN
  - `frontend/src/pages/c/[token].astro` (NEW)
  - `backend/tests/features/magic_link.feature`
- **ADR refs** : —
- **Cluster coord** : NEW → AppError natif

### Story 3.3 — PWA Contractor (manifest + Service Worker + UX 3 écrans)

- **Goal** : PWA install-able sur mobile contractor. 3 écrans max : (1) résumé scope, (2) action (réponse/devis/évaluation), (3) confirmation. Offline-safe (IndexedDB draft).
- **FR/INV** : FR6 (suite) ; brief C13, mémoire [[fe-refactor-test-driven]]
- **Effort** : M
- **Deps** : 3.2
- **AC 4-cat** :
  - `@happy` : Contractor sur Android Chrome → ouvre `/c/<token>` → install prompt → installe → ouvre PWA → flow 3 écrans → soumet réponse → confirmation
  - `@edge` : Contractor offline → écrit réponse → reconnecté → sync auto IndexedDB → 200 + audit
  - `@security` : PWA installée avec token expiré → écran "Lien expiré" + bouton "Demander un nouveau lien" (mailto syndic)
  - `@negative` : SW cache stale après release → query param `?v=<version>` force re-register + re-fetch (apprentissage #549)
- **data-testid** : `pwa-screen-1-summary`, `pwa-screen-2-action`, `pwa-screen-3-confirm`, `pwa-install-prompt`
- **Files** :
  - `frontend/public/manifest.webmanifest` (extension scope `/c/*`)
  - `frontend/public/sw.js` (cache strategy network-first /magic-links + cache-first /c/*)
  - `frontend/src/lib/components/pwa/MagicLinkContractorPage.svelte` (NEW)
  - `frontend/tests/e2e/refonte-ux/slice-3-magic-link-pwa/pwa-contractor.spec.ts` (Playwright `--device "Pixel 7"`)
- **ADR refs** : —
- **Cluster coord** : —

### Story 3.4 — Entité `Mandate` (avocat/notaire/AMO/architect/BET) + workflow émission

- **Goal** : Table `mandates` + entité + use-cases issue + workflow émission (syndic OU décision AG selon kind) + audit immuable. Refus 403 si `valid_until < now()`.
- **FR/INV** : FR7 ; INV-14
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : Syndic émet Mandate notaire pour Building X (étatdaté) avec valid_until=2026-12-31 → Mandate persistée + audit_event
  - `@edge` : Mandate juste à `valid_until` (1 seconde avant) → autorisée pour action ; après → 403 expired
  - `@security` : Notaire mandaté sur Unit Y tente accès Unit Z → 403 `AcpNotInScope` ; expiré → 403 `MandateExpired`
  - `@negative` : POST `/mandates` sans `valid_until` → 422 (champ obligatoire)
- **data-testid** : `mandate-issue-submit`, `mandate-kind-select`, `mandate-valid-until-input`
- **Files** :
  - `backend/src/domain/entities/mandate.rs`
  - `backend/src/application/ports/mandate_repository.rs`
  - `backend/src/application/use_cases/mandate_use_cases.rs`
  - `backend/src/infrastructure/database/repositories/mandate_repository_impl.rs`
  - `backend/migrations/20260605_020000_create_mandates.sql` + DOWN
  - `backend/tests/features/mandate.feature`
- **ADR refs** : —
- **Cluster coord** : NEW → AppError natif

### Story 3.5 — Délégation temporaire `UserRoleAssignment.valid_until`

- **Goal** : Extension table `user_role_assignments` avec `valid_until` (NULLABLE = permanent) + `delegated_from_user_id` (NULLABLE). Use-case `delegate_role` + audit.
- **FR/INV** : FR8 ; INV-8
- **Effort** : S
- **Deps** : 3.1
- **AC 4-cat** :
  - `@happy` : Syndic délègue role syndic à Owner Pierre pour 7 jours → Pierre voit menus syndic dans cette fenêtre → rôle expiré auto
  - `@edge` : Délégation juste à `valid_until` → action OK ; +1ms → 403
  - `@security` : Owner Pierre ne peut pas re-déléguer à un tiers (la délégation est non-transitive)
  - `@negative` : Délégation avec `valid_until < now()` → 422
- **data-testid** : `role-delegate-submit`, `role-delegate-target-input`, `role-delegate-until-input`
- **Files** :
  - `backend/migrations/20260605_030000_extend_user_role_assignments.sql` + DOWN
  - `backend/src/domain/entities/user_role_assignment.rs` (refacto)
  - `backend/src/application/use_cases/role_delegation_use_cases.rs` (NEW)
  - `backend/tests/features/role_delegation.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 3.6 — `Ticket.kind=complaint` + severity + evidence + witnesses

- **Goal** : Extension entité `Ticket` avec `kind` enum (Request|Complaint), `severity` (Low|Normal|High|Critical), `incident_date`, `evidence_attachments[]`, `witnesses[]`.
- **FR/INV** : FR31 ; brief C17
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : Owner crée plainte severity=critical avec 3 photos + 2 témoins owners → Ticket persisté + notifications syndic + CdC
  - `@edge` : Plainte sans evidence (text-only) → autorisée mais badge "preuves manquantes"
  - `@security` : Audit immuable INV-24 : 5 min après création, tentative edit → 403 `TicketImmutable`
  - `@negative` : kind=complaint sans severity → 422 ; evidence_attachments > 10 fichiers → 422 ; fichier > 10MB → 422
- **data-testid** : `ticket-create-kind-select`, `ticket-severity-select`, `ticket-evidence-upload`, `ticket-witness-add`, `ticket-submit`
- **Files** :
  - `backend/migrations/20260605_040000_extend_tickets_complaint.sql` + DOWN
  - `backend/src/domain/entities/ticket.rs` (refacto)
  - `backend/src/application/dto/ticket_dto.rs`
  - `backend/src/application/use_cases/ticket_use_cases.rs` (refacto)
  - `frontend/src/lib/components/tickets/TicketCreate.svelte` (refacto)
  - `backend/tests/features/ticket_complaint.feature`
- **ADR refs** : ADR-0012
- **Cluster coord** : si use-case ticket touche legacy `Result<_, String>` → migrer #555 simultané

### Story 3.7 — `SyndicResponse` + SLA + escalade CdC

- **Goal** : Entité `SyndicResponse` (append-only) + champ `Ticket.sla_due_at` calculé par severity policy + escalade CdC si dépassé.
- **FR/INV** : FR32 ; INV-23, brief C17
- **Effort** : M
- **Deps** : 3.6
- **AC 4-cat** :
  - `@happy` : Syndic répond < SLA (24h pour critical, 5j pour low) → escalade évitée + notification owner
  - `@edge` : SLA juste à expiration (1 seconde avant) → autorisée ; juste après → escalade créée
  - `@security` : SyndicResponse non éditable (audit immuable) ; CdC reçoit notification escalade mais ne peut pas répondre à la place du syndic
  - `@negative` : Tentative édit response 1h après → 403 `ResponseImmutable`
- **data-testid** : `syndic-response-submit`, `syndic-response-action-proposed`, `ticket-sla-badge`
- **Files** :
  - `backend/migrations/20260605_050000_create_syndic_responses.sql` + DOWN
  - `backend/src/domain/entities/syndic_response.rs`
  - `backend/src/application/use_cases/syndic_response_use_cases.rs`
  - `backend/src/infrastructure/jobs/sla_escalation_job.rs` (NEW, cron)
  - `backend/tests/features/syndic_response_sla.feature`
- **ADR refs** : —
- **Cluster coord** : NEW → AppError natif

### Story 3.8 — `TechnicalSpec` versionnable (cahier des charges signé)

- **Goal** : Entité `TechnicalSpec` avec versionning semver + signatures multi-parties (ACP/Syndic/AMO) + attachements documents.
- **FR/INV** : FR33 ; brief C16
- **Effort** : M
- **Deps** : 3.4 (Mandate AMO)
- **AC 4-cat** :
  - `@happy` : Syndic crée TechnicalSpec v1.0.0 → AMO signe → status Approved → versionnable v1.1.0 si modif
  - `@edge` : Spec v2.0.0 majeure → signatures précédentes invalidées + re-signature requise
  - `@security` : Owner non-mandaté ne peut pas signer ; signature par tiers → 403
  - `@negative` : Tentative création spec avec scope mal défini (deliverables vides) → 422
- **data-testid** : `tech-spec-create-submit`, `tech-spec-version-input`, `tech-spec-sign-submit`, `tech-spec-attach-upload`
- **Files** :
  - `backend/migrations/20260605_060000_create_technical_specs.sql` + DOWN
  - `backend/src/domain/entities/technical_spec.rs`
  - `backend/src/application/use_cases/technical_spec_use_cases.rs`
  - `backend/tests/features/technical_spec.feature`
- **ADR refs** : ADR-0014 (signatures)
- **Cluster coord** : —

### Story 3.9 — `ContractorEvaluation` (refuse 422 sans TechnicalSpec)

- **Goal** : Entité `ContractorEvaluation` qui nécessite `TechnicalSpec` préalable (refus 422 `TechnicalSpecRequired` sinon). Lien `tickets_linked[]` vers plaintes ayant motivé l'éval.
- **FR/INV** : FR34, FR35 ; INV-21, INV-24, brief C18
- **Effort** : M
- **Deps** : 3.8
- **AC 4-cat** :
  - `@happy` : Évaluation Contractor X référençant TechnicalSpec v1.0.0 + 2 tickets liés → scores 1-5 enregistrés + audit
  - `@edge` : Évaluation pile à expiration de TechnicalSpec (1 sec avant) → autorisée
  - `@security` : ContractorEvaluation append-only (INV-24) ; tentative édit → 403
  - `@negative` : Évaluation sans TechnicalSpec → 422 `TechnicalSpecRequired` ; Contractor inexistant → 404
- **data-testid** : `contractor-eval-submit`, `contractor-eval-spec-select`, `contractor-eval-tickets-link`, `contractor-eval-scores-{{criterion}}`
- **Files** :
  - `backend/migrations/20260605_070000_create_contractor_evaluations.sql` + DOWN
  - `backend/src/domain/entities/contractor_evaluation.rs`
  - `backend/src/application/use_cases/contractor_evaluation_use_cases.rs`
  - `backend/tests/features/contractor_evaluation.feature`
- **ADR refs** : ADR-0014 (signature évaluation optionnelle)
- **Cluster coord** : —

---

## 6. Slice 4 — Governance hybride + Commissaire + CdC + signatures eIDAS

### Story 4.1 — `Meeting.mode` enum (in_person/remote/hybrid) + quorum agrégé

- **Goal** : Extension `meetings.mode` + use-case `compute_quorum` qui agrège attendees_in_person + remote + proxy en Decimal.
- **FR/INV** : FR13, FR14 ; INV-19, brief C15
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : AG hybride mode=hybrid → 10 présentiels + 5 distants + 3 procurations → quorum agrégé OK selon Decimal somme
  - `@edge` : Quorum à exactement 50.0% (seuil Art. 3.87 §3) → respecté ; 49.99% → refusé
  - `@security` : Mode=remote impose `auth_method` strong (cf. 4.2)
  - `@negative` : Meeting mode=hybrid sans configuration distance (videoconf_url manquant) → 422
- **data-testid** : `meeting-mode-select`, `meeting-quorum-current`, `meeting-quorum-required`
- **Files** :
  - `backend/migrations/20260610_010000_extend_meetings_hybrid.sql` + DOWN
  - `backend/src/domain/entities/meeting.rs` (refacto)
  - `backend/src/application/use_cases/compute_quorum_use_case.rs` (refacto)
  - `backend/tests/features/meeting_hybrid_quorum.feature`
- **ADR refs** : —
- **Cluster coord** : **#433 simultané** (quorum Decimal, pas f64) ; **#555 simultané**

### Story 4.2 — Vote distant `auth_method` strong (#48 itsme/eID)

- **Goal** : Extension `votes.auth_method` enum (presence|proxy|itsme|eid) + refus 403 si mode meeting=remote/hybrid sans auth strong.
- **FR/INV** : FR15 ; INV-18, #48
- **Effort** : M
- **Deps** : 4.1
- **AC 4-cat** :
  - `@happy` : Owner vote distant avec itsme → vote enregistré avec `auth_method=itsme`
  - `@edge` : Owner tente vote distant avec proxy (procuration distance) → autorisé sous conditions Art. 3.87 §4
  - `@security` : Owner tente vote distant avec auth_method=presence → 403 `VoteAuthInsufficient`
  - `@negative` : Vote sans auth_method → 422
- **data-testid** : `vote-auth-method-select`, `vote-itsme-button`, `vote-eid-button`, `vote-cast-submit`
- **Files** :
  - `backend/migrations/20260610_020000_extend_votes_auth_method.sql` + DOWN
  - `backend/src/domain/entities/vote.rs`
  - `backend/src/application/use_cases/vote_use_cases.rs`
  - `frontend/src/lib/components/governance/VoteCast.svelte` (refacto)
  - `backend/tests/features/vote_remote_auth.feature`
- **ADR refs** : ADR-0014
- **Cluster coord** : —

### Story 4.3 — `Minutes` (PV) + 2 signatures eIDAS qualifiées

- **Goal** : Aggregate `Minutes` + 2 signatures (président + secrétaire) eIDAS qualifié. Refus `Meeting.complete()` sans 2 signatures (cf. 4.5).
- **FR/INV** : FR16 ; INV-20
- **Effort** : M
- **Deps** : 4.4 (adapter signature)
- **AC 4-cat** :
  - `@happy` : Président signe PV via eID belge → secrétaire signe via itsme → Meeting.complete() OK
  - `@edge` : 1 seul signataire → Meeting reste InProgress, attente 2ème
  - `@security` : Tentative signature par owner non-président/secrétaire → 403
  - `@negative` : Signature invalide (eIDAS rejet) → 422 + détail erreur ; PV vide → 422
- **data-testid** : `minutes-pdf-preview`, `minutes-sign-president`, `minutes-sign-secretary`, `minutes-status-badge`
- **Files** :
  - `backend/migrations/20260610_030000_create_minutes.sql` + DOWN
  - `backend/src/domain/entities/minutes.rs`
  - `backend/src/application/use_cases/minutes_use_cases.rs`
  - `frontend/src/lib/components/governance/MinutesSigning.svelte`
  - `backend/tests/features/minutes_signatures.feature`
- **ADR refs** : ADR-0014
- **Cluster coord** : —

### Story 4.4 — Adapter `ElectronicSignatureProvider` (port + 3 adapters)

- **Goal** : Port hexagonal `ElectronicSignatureProvider` + 3 adapters : eID belge (FAS), itsme, Universign. Sélection par ACP (préférence cabinet) avec fallback Universign pour non-BE.
- **FR/INV** : FR16, FR20, FR25 ; ADR-0014
- **Effort** : L
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : eID belge demande signature → reçoit `QualifiedSignature` + persiste audit ; itsme idem ; Universign idem (mock en dev)
  - `@edge` : Préférence cabinet = itsme → fallback Universign si user non-BE
  - `@security` : Hash document calculé avant envoi prestataire (HMAC SHA-256) ; vérification à réception
  - `@negative` : Prestataire timeout → retry exponential backoff 3× → erreur typée + audit
- **data-testid** : (backend pur, UI dans 4.3)
- **Files** :
  - `backend/src/application/ports/electronic_signature_provider.rs`
  - `backend/src/infrastructure/external/signature_provider_eid.rs`
  - `backend/src/infrastructure/external/signature_provider_itsme.rs`
  - `backend/src/infrastructure/external/signature_provider_universign.rs`
  - `backend/tests/integration/signature_providers_test.rs` (mocks)
- **ADR refs** : ADR-0014
- **Cluster coord** : NEW → AppError natif

### Story 4.5 — `Meeting.assert_can_complete()` (reprise #554)

- **Goal** : Méthode domain `assert_can_complete()` qui vérifie : convocations envoyées + quorum atteint + résolutions clôturées + PV signé 2×. Refus 422 `MeetingNotCompletable{missing:[...]}` sinon.
- **FR/INV** : FR17 ; #554, brief C-brief
- **Effort** : M
- **Deps** : 4.1, 4.2, 4.3
- **AC 4-cat** :
  - `@happy` : Meeting avec toutes pré-conditions → complete() → status Completed + audit
  - `@edge` : Meeting avec PV signé 2× mais 1 résolution encore ouverte → 422 avec missing=["resolution_R-42_not_closed"]
  - `@security` : Tentative complete() par owner non-syndic → 403
  - `@negative` : Tentative complete() sur Meeting déjà Completed → 422 `MeetingAlreadyCompleted`
- **data-testid** : `meeting-complete-submit`, `meeting-missing-checklist-{{key}}`
- **Files** :
  - `backend/src/domain/entities/meeting.rs` (ajout `assert_can_complete`)
  - `backend/src/application/use_cases/meeting_use_cases.rs` (refacto)
  - `backend/tests/features/meeting_complete.feature`
- **ADR refs** : —
- **Cluster coord** : **#555 simultané** (Result<_, String> legacy à migrer dans meeting_use_cases)

### Story 4.6 — Résolution `EvaluationContractors` AGO auto non retirable

- **Goal** : Use-case `generate_ago_resolutions(meeting_id)` ajoute auto une Resolution `kind=EvaluationContractors_AUTO` + `is_auto_generated=true`. Refus 403 `ResolutionAutoNotRemovable` si tentative delete.
- **FR/INV** : FR18 ; INV-22
- **Effort** : S
- **Deps** : 4.1
- **AC 4-cat** :
  - `@happy` : AGO créée → use-case lance auto → Resolution `EvaluationContractors_AUTO` présente non-éditable
  - `@edge` : AGE (extraordinaire) → pas de génération auto
  - `@security` : Syndic tente DELETE resolution `EvaluationContractors_AUTO` → 403
  - `@negative` : Modification text resolution AUTO → 403 (immutable)
- **data-testid** : `resolution-auto-badge`, `resolution-delete-{{id}}` (caché si AUTO)
- **Files** :
  - `backend/src/application/use_cases/generate_ago_resolutions_use_case.rs` (NEW)
  - `backend/src/domain/entities/resolution.rs` (refacto avec is_auto_generated)
  - `backend/tests/features/resolution_auto_evaluation.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 4.7 — `CdC` membre élu + action `create_alert`

- **Goal** : Entité `CdC` (conseil de copropriété — Art. 3.87 §1 CC) avec élection en AG + action `create_alert(text, severity, target=AG_next)`.
- **FR/INV** : FR19 ; INV-12, brief C13
- **Effort** : S
- **Deps** : 4.5
- **AC 4-cat** :
  - `@happy` : Élection CdC en AG → 3 membres élus → CdC peut create_alert visible AG suivante
  - `@edge` : CdC membre démissionne → mandate_until = now() → perd droits
  - `@security` : Owner non-élu CdC tente create_alert → 403 ; après mandate_until → 403
  - `@negative` : Élection sans quorum AG → 422
- **data-testid** : `cdc-elect-submit`, `cdc-member-list`, `cdc-alert-create-submit`
- **Files** :
  - `backend/src/domain/entities/cdc.rs` (extension ou NEW selon existant)
  - `backend/src/application/use_cases/cdc_use_cases.rs`
  - `backend/tests/features/cdc_alert.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 4.8 — `CommissaireAuxComptes` + `VerificationCertificate`

- **Goal** : Entité `Commissaire` (Art. 3.88 CC) + use-case `sign_certificate(financial_period)` → `VerificationCertificate` signée eIDAS. Workflow PRE-clôture comptes annuels.
- **FR/INV** : FR20, FR25 ; INV-11, brief C10
- **Effort** : M
- **Deps** : 4.4
- **AC 4-cat** :
  - `@happy` : Commissaire signe certificat période 2026 → VerificationCertificate persisté + comptes annuels passent en status Verified
  - `@edge` : Commissaire mandate_until expiré → 403
  - `@security` : Syndic ne peut pas signer à la place du Commissaire ; édit écriture après signature → 403
  - `@negative` : Tentative clôture comptes annuels sans VerificationCertificate → 422
- **data-testid** : `commissaire-sign-cert-submit`, `verification-cert-status`, `annual-accounts-close-submit`
- **Files** :
  - `backend/src/domain/entities/commissaire.rs` (NEW ou extension)
  - `backend/src/domain/entities/verification_certificate.rs` (NEW)
  - `backend/migrations/20260610_040000_create_verification_certificates.sql` + DOWN
  - `backend/src/application/use_cases/verification_certificate_use_cases.rs`
  - `backend/tests/features/commissaire_certificate.feature`
- **ADR refs** : ADR-0014
- **Cluster coord** : **#433 simultané** (montants PCMN clôture annuelle Decimal) ; **#555 simultané** sur accounting

### Story 4.9 — `validate-before-compute` sur calculs accounting [cluster-coord]

- **Goal** : Tout use-case calcul (charges/répartition/quorum/appels de fonds/PV) commence par `building.assert_conformant()?`. Refus 422 `BuildingNotConformant{reason}` sinon. Pattern aligné mémoire [[validate-before-compute]].
- **FR/INV** : FR22 ; mémoire [[validate-before-compute]], brief C-brief
- **Effort** : L
- **Deps** : 1.4 (is_conformant)
- **AC 4-cat** :
  - `@happy` : Building conforme → calculs charges/répartition OK + résultats Decimal exacts
  - `@edge` : Building avec 999/1000 millièmes (delta 0.0001) → 422 sans tolérance (mémoire #433)
  - `@security` : Audit immuable INV-24 : tentative calcul sur building non-conforme → log + alerte
  - `@negative` : Calcul SUM tantièmes sur 0 unit → renvoie Decimal(0), pas NaN ; FE affiche `—`
- **data-testid** : `charge-distribution-error-banner`, `call-for-funds-error-banner`
- **Files** :
  - `backend/src/application/use_cases/expense_use_cases.rs` (refacto pre-check + Decimal + AppError)
  - `backend/src/application/use_cases/call_for_funds_use_cases.rs` (idem)
  - `backend/src/application/use_cases/charge_distribution_use_case.rs` (idem)
  - `backend/src/application/use_cases/etat_date_use_cases.rs` (idem)
  - `backend/tests/features/validate_before_compute.feature`
  - `frontend/src/lib/components/expenses/ExpenseList.svelte` (banner)
- **ADR refs** : ADR-0010
- **Cluster coord** : **#433 + #555 simultané** sur 4 use-cases (PR `[cluster-coord]` regroupé : 1 PR = 4 use-cases × 2 migrations = atomicité validate-before-compute)

---

## 7. Slice 5 — Modularité + onboarding + RBAC Communauté Moderator

### Story 5.1 — Table `acp_enabled_modules` + `ModuleGuard` middleware + `ModuleDisabledError`

- **Goal** : Table + entité + use-cases enable/disable + middleware Actix `ModuleGuard` + extension `AppError::ModuleDisabled`.
- **FR/INV** : FR39 ; INV-25, ADR-0015
- **Effort** : M
- **Deps** : 1.1
- **AC 4-cat** :
  - `@happy` : Admin enable module=community ACP X → menu Communauté visible côté syndic ACP X ; route `/community/*` répond 200
  - `@edge` : Module activé puis désactivé puis réactivé → `archived_at` cycling → données intactes (cf. INV-27)
  - `@security` : Syndic ACP avec Compta désactivée tente `/expenses` → 403 `ModuleDisabled{module:accounting}`
  - `@negative` : Module name invalide ("foobar") → 422 ; module=identity tentative disable → 403 (toujours actif)
- **data-testid** : `module-enable-submit`, `module-disable-submit`, `module-list-row-{{name}}`
- **Files** :
  - `backend/migrations/20260620_010000_create_acp_enabled_modules.sql` + DOWN
  - `backend/src/domain/entities/acp_enabled_module.rs`
  - `backend/src/application/ports/module_registry.rs`
  - `backend/src/application/use_cases/module_registry_use_cases.rs`
  - `backend/src/infrastructure/web/middleware/module_guard.rs`
  - `backend/src/domain/errors/app_error.rs` (extension ModuleDisabled)
  - `backend/tests/features/module_registry.feature`
- **ADR refs** : ADR-0015
- **Cluster coord** : NEW → AppError natif

### Story 5.2 — UI `ModuleGate.svelte` + store `enabled_modules`

- **Goal** : Composant `<ModuleGate module="community">…</ModuleGate>` masque enfants si module désactivé. Store `enabled_modules` synced sur sélection ACP.
- **FR/INV** : FR39 (UI) ; ADR-0015
- **Effort** : S
- **Deps** : 5.1
- **AC 4-cat** :
  - `@happy` : ACP avec community activé → contenu rendu ; ACP sans → fragment vide (pas placeholder)
  - `@edge` : Bascule ACP en cours de session → store re-fetch + UI re-render
  - `@security` : ModuleGate côté UI ne remplace pas middleware backend (defense-in-depth)
  - `@negative` : Module name inconnu dans gate → erreur typée console + fragment vide
- **data-testid** : `module-gate-{{module}}` (présent ssi rendu)
- **Files** :
  - `frontend/src/lib/components/global/ModuleGate.svelte`
  - `frontend/src/stores/enabled_modules.svelte.ts`
  - `frontend/src/lib/api/modules.ts`
  - `frontend/src/lib/components/global/__tests__/ModuleGate.test.ts`
- **ADR refs** : ADR-0015
- **Cluster coord** : —

### Story 5.3 — Syndic = `community.moderator` (RBAC Community SEL/Poll/Notice/SharedObject)

- **Goal** : Adapter use-cases Community pour refuser participation perso syndic (create/vote/comment/échange SEL) MAIS autoriser modération (edit/supprime). Pattern `Moderator` rôle.
- **FR/INV** : FR26 ; INV-4
- **Effort** : M
- **Deps** : 3.1, 5.1
- **AC 4-cat** :
  - `@happy` : Syndic moderator édite SEL litigieux → OK ; Owner participe → OK
  - `@edge` : Syndic cumule role owner (a un lot dans l'ACP) → peut participer ès qualités owner
  - `@security` : Syndic pur (sans role owner) tente create_sel_offer → 403 INV-4 ; vote poll → 403 ; comment notice → 403
  - `@negative` : Modération sans motif texte → 422 (audit requis)
- **data-testid** : `sel-create-submit` (caché syndic non-owner), `sel-moderate-edit`, `sel-moderate-delete`
- **Files** :
  - `backend/src/application/use_cases/sel_use_cases.rs` (refacto permission)
  - `backend/src/application/use_cases/poll_use_cases.rs` (idem)
  - `backend/src/application/use_cases/notice_use_cases.rs` (idem)
  - `backend/src/application/use_cases/shared_object_use_cases.rs` (idem)
  - `backend/tests/features/community_syndic_moderator.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 5.4 — `Reservation.on_behalf_of_acp` (exception syndic)

- **Goal** : Champ `Reservation.on_behalf_of_acp: bool` + motif obligatoire si true. Syndic autorisé à réserver pour AG/prestataires sous cette flag.
- **FR/INV** : FR27 ; INV-5
- **Effort** : S
- **Deps** : 5.3
- **AC 4-cat** :
  - `@happy` : Syndic réserve salle commune `on_behalf_of_acp=true motif="AG annuelle"` → OK + log spécifique
  - `@edge` : Syndic réserve `on_behalf_of_acp=false` → 403 (participation perso interdite)
  - `@security` : Owner ne peut pas mettre `on_behalf_of_acp=true` (réservé syndic)
  - `@negative` : `on_behalf_of_acp=true` sans motif → 422
- **data-testid** : `reservation-on-behalf-toggle`, `reservation-motif-input`, `reservation-submit`
- **Files** :
  - `backend/src/domain/entities/reservation.rs` (refacto)
  - `backend/src/application/use_cases/reservation_use_cases.rs` (refacto)
  - `frontend/src/lib/components/community/ReservationCreate.svelte` (refacto)
  - `backend/tests/features/reservation_on_behalf.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 5.5 — Comptable (encodeur ET émetteur) 403 sur `/community/*`

- **Goal** : Vérification middleware `community.read_access` exclut explicitement accountant.encodeur ET accountant.emetteur. CdC participe en tant qu'owner (cf. FR30).
- **FR/INV** : FR28, FR30 ; INV-6
- **Effort** : S
- **Deps** : 5.3
- **AC 4-cat** :
  - `@happy` : Owner Marie accède `/community/sel` → OK ; CdC Catherine idem (rôle owner sous-jacent)
  - `@edge` : Comptable Paul (encodeur) cumule role owner → accède via role owner OK
  - `@security` : Comptable Pierre (émetteur) pur sans role owner → 403 INV-6
  - `@negative` : Accès direct URL bypass UI → 403 middleware
- **data-testid** : `community-no-access-message` (visible si 403)
- **Files** :
  - `backend/src/infrastructure/web/middleware/community_access_guard.rs` (NEW)
  - `backend/tests/features/community_accountant_forbidden.feature`
- **ADR refs** : —
- **Cluster coord** : —

### Story 5.6 — Activation/désactivation modules audité + archivage data

- **Goal** : Use-cases enable/disable avec audit immuable + archivage data (`archived_at` set, jamais DELETE). Re-activation = restauration. Vote AG requis pour Accounting/Governance.
- **FR/INV** : FR41, FR42 ; INV-26, INV-27
- **Effort** : M
- **Deps** : 5.1
- **AC 4-cat** :
  - `@happy` : Admin disable community ACP X → archived_at set → re-enable → data intacte + audit cycle
  - `@edge` : Désactivation module avec dépendance active (AG planifiée si Governance désactivé) → 422 + message clair
  - `@security` : Admin tente disable accounting sans vote AG ≥ 50% → 403 INV-26 ; tentative DELETE row → impossible (use-cases ne l'exposent pas)
  - `@negative` : Re-enable d'un module jamais activé → 404 `ModuleNeverEnabled`
- **data-testid** : `module-audit-log-{{event_id}}`, `module-archived-banner`
- **Files** :
  - `backend/src/application/use_cases/module_registry_use_cases.rs` (extension : audit + dependency check)
  - `backend/tests/features/module_lifecycle.feature`
- **ADR refs** : ADR-0015
- **Cluster coord** : —

### Story 5.7 — Onboarding modulaire wizard ≤ 5 min

- **Goal** : Composant `OnboardingWizard.svelte` 5 étapes : profil ACP → recommandation modules → activation → démo → confirmation. KPI < 5min mesuré client-side.
- **FR/INV** : FR40 ; SC17, C22
- **Effort** : M
- **Deps** : 5.1, 5.2
- **AC 4-cat** :
  - `@happy` : User test naïf complète wizard en 4min23 → analytics enregistre → modules sélectionnés activés
  - `@edge` : User skip recommandation → activation modules par défaut (community + identity)
  - `@security` : Wizard accessible que pour admin SaaS lors création nouvelle ACP, jamais bypass
  - `@negative` : Wizard interrompu mi-parcours → reprise possible (state IndexedDB)
- **data-testid** : `onboarding-step-{{n}}`, `onboarding-module-toggle-{{module}}`, `onboarding-finish-submit`
- **Files** :
  - `frontend/src/lib/components/onboarding/OnboardingWizard.svelte` (NEW)
  - `frontend/src/lib/components/onboarding/__tests__/OnboardingWizard.test.ts`
  - `frontend/tests/e2e/refonte-ux/slice-5-modularity/onboarding-wizard.spec.ts`
- **ADR refs** : ADR-0015
- **Cluster coord** : —

### Story 5.8 — Gate CI a11y axe-core + data-testid + Lighthouse

- **Goal** : CI gate : axe-core ≥ 90 sur pages refonte, ESLint plugin `koprogo-testid-required`, Lighthouse a11y ≥ 90. Bloque PR si violations.
- **FR/INV** : FR45 ; NFR2, mémoires [[a11y-wcag-aa-baseline]], [[data-testid-systematic]]
- **Effort** : M
- **Deps** : toutes stories slice 2-5 (avoir le code à auditer)
- **AC 4-cat** :
  - `@happy` : PR slice 5 mergeable → axe-core PASS + Lighthouse a11y ≥ 90 + zéro testid manquant
  - `@edge` : Composant avec aria-label correct → axe-core PASS même sans testid (lint check séparé)
  - `@security` : Tentative push sans testid sur button interactif → CI fail bloquant (gate dur à partir slice 5)
  - `@negative` : Lighthouse score 89 → fail (≥90 minimum)
- **data-testid** : (cible : tous les composants nouveaux/refactorés des slices 1-5)
- **Files** :
  - `.github/workflows/ci.yml` (ajout jobs axe-core + lighthouse + lint-testid)
  - `frontend/eslint-plugins/koprogo-testid-required.js` (NEW)
  - `frontend/playwright.config.ts` (project a11y-audit)
  - `docs/ci/A11Y_GATE.md` (doc gate)
- **ADR refs** : ADR-0012, ADR-0013
- **Cluster coord** : —

---

## 8. Slice Transversal — Observabilité + qualité continue

### Story Tx.1 — Caractérisation reste VERTE (gate CI inter-slice)

- **Goal** : Job CI dédié `test:characterization` qui tourne sur **chaque PR** des slices 1-5. Échec = blocage merge.
- **FR/INV** : FR43 ; mémoire [[fe-refactor-test-driven]]
- **Effort** : S
- **Deps** : 0.1
- **AC 4-cat** :
  - `@happy` : PR slice 2 → characterization VERT → mergeable
  - `@edge` : Caractérisation partiellement modifiée (commit qui ajuste un test bugué) → review explicite obligatoire
  - `@security` : Bypass via `--no-verify` impossible (CI server-side)
  - `@negative` : Characterization ROUGE → blocage + alerte + investigation
- **Files** :
  - `.github/workflows/ci.yml` (job characterization)
  - `package.json` (script `test:characterization`)
- **ADR refs** : ADR-0013
- **Cluster coord** : —

### Story Tx.2 — Helpers shared multi-rôle (extension #550)

- **Goal** : Compléter `frontend/tests/e2e/helpers/auth.ts` avec tous les rôles + variantes WithBuilding/WithAcp/WithMagicLink. Zéro helper local autorisé dans `refonte-ux/`.
- **FR/INV** : FR44 ; #550
- **Effort** : S
- **Deps** : 0.1
- **AC 4-cat** :
  - `@happy` : `loginAsContractorMagicLink(page, token)` fonctionne sur mobile Pixel 7
  - `@edge` : Helper avec building inexistant → fail clair, pas timeout silencieux
  - `@security` : Helpers ne loggent jamais credentials en clair
  - `@negative` : Helper UI-login détecté dans `refonte-ux/` → CI lint fail
- **Files** :
  - `frontend/tests/e2e/helpers/auth.ts` (extension)
  - `frontend/tests/e2e/helpers/building.ts`
  - `frontend/tests/e2e/helpers/magic-link.ts`
  - `.github/workflows/ci.yml` (lint check)
- **ADR refs** : ADR-0013
- **Cluster coord** : —

### Story Tx.3 — Documentation `docs/agent-activity/` (Tier 2 log)

- **Goal** : Pour chaque slice, créer `docs/agent-activity/YYYY-MM-DD-bob-slice-N.md` log Tier 2 (lecture/diagnostic/proposal). Conforme règle CRITICAL.md §11.
- **FR/INV** : transverse (gouvernance agent)
- **Effort** : S (continu)
- **Deps** : —
- **AC 4-cat** :
  - `@happy` : PR slice N inclut le log activity Tier 2 daté
  - `@edge` : Slice étalée sur plusieurs semaines → 1 fichier log par semaine
  - `@security` : Logs ne contiennent ni token ni secret
  - `@negative` : PR sans log Tier 2 → reviewer demande mise à jour
- **Files** :
  - `docs/agent-activity/2026-MM-DD-bob-slice-N.md` (N=1..5)
- **ADR refs** : —
- **Cluster coord** : —

---

## 9. Matrice dépendances inter-stories

```
0.1 (caractérisation, immuable)
 │
 ├─ 1.1 ─┬─ 1.2 ─┬─ 1.3
 │       │       └─ 1.4 ─┬─ 2.3
 │       │              └─ 4.9
 │       ├─ 2.1 ─ 2.2 ─ 2.3 ─ 2.4 ─ 2.5
 │       ├─ 3.1 ─┬─ 3.2 ─ 3.3
 │       │       ├─ 3.4 ─ 3.8 ─ 3.9
 │       │       ├─ 3.5
 │       │       └─ 3.6 ─ 3.7
 │       └─ 4.1 ─┬─ 4.2
 │               ├─ 4.4 ─ 4.3
 │               └─ 4.5 ─ 4.6 ─ 4.7 ─ 4.8 ─ 4.9
 │
 ├─ Tx.1 (continuous CI gate)
 ├─ Tx.2 (helpers shared)
 │
 └─ 5.1 ─┬─ 5.2 ─ 5.3 ─ 5.4 ─ 5.5
         ├─ 5.6
         ├─ 5.7
         └─ 5.8 (final gate slice 5)

Tx.3 (log Tier 2 continu, indépendant)
```

---

## 10. Coordination cross-épics par story

| Cluster / Epic | Stories impactées | Convention |
|---|---|---|
| **#433 Decimal umbrella** | 1.4, 3.1 (expense/cff), 4.1 (quorum), 4.8 (PCMN), 4.9 (validate-before-compute) | 1 PR par use-case = 2 migrations atomiques (refonte + Decimal) |
| **#555 Result&lt;_, String&gt; epic** | 3.1 (legacy), 3.6 (legacy), 4.1, 4.5, 4.8, 4.9 | idem, simultané dans la même PR |
| **#553 Building admin UX** | 1.4, 2.5 | Closed by ces stories |
| **#554 World-model seed + AG state** | 4.5, 4.6 + Tx.1 (seeds caractérisation) | Closed by ces stories |
| **#550 Playwright stratification** | Tx.2 | Closed by Tx.2 |
| **#48 itsme/eID** | 4.2, 4.4 | Promu in-scope, closed by ces stories |
| **#552 work-reports 400** | hors scope direct | Contract Types Check CI reste VERT |

---

## 11. Gate de validation Phase 4 — sign-off humain

> ✅ **Stories SIGNÉES par @gilmry le 2026-05-20** — Phase 4 verrouillée, Phase 5 (Validation PO) débloquée.

- [x] Découpage 6 slices validé (5 fonctionnelles + 1 transversal + slice 0 caractérisation)
- [x] 31 stories validées (1 story = 1 PR, sauf `[cluster-coord]` ex 4.9)
- [x] Convention naming `<slice>.<n>-<entity>-<action>` + template story acceptés
- [x] AC 4-cat condensés validés (Gherkin complet à produire en ticket GH)
- [x] data-testid listés cohérents avec ADR-0012
- [x] Dépendances inter-stories validées (matrice §9)
- [x] Coordination clusters #433/#555/#553/#554/#550/#48 (§10) confirmée
- [x] Gate par slice + gate transversal CI accepté (Tx.1/Tx.2/Tx.3)

**Date signature** : **2026-05-20**
**Signature** : **@gilmry** ✅

---

## 12. Phase suivante

Phase 5 (Validation Product Owner) débloquée par sign-off Phase 4. Le PO valide la priorisation slice/story vs valeur métier et budget tokens. Une fois Phase 5 signée :

1. **Création issues GitHub** : 1 Epic + 31 sous-issues (1 par story), avec frontmatter Maury, AC 4-cat, ADR refs, cluster coord, data-testid, files
2. **Intégration WBS go-live v0.1.0** : ajout stories slice 1 (refacto ACP) comme bloqueurs Track H Conformité, stories slice 2 (sélecteur+banner+#553) au WP-D1/E1 enrichis (déjà en place via commit `7c24150`)
3. **Phase 6 (Exécution dev/qa/release-manager)** : 1 story = 1 branche `story/<slice>.<n>-<entity>-<action>` = 1 PR. Gate caractérisation VERT + 4-cat + a11y + testid CI obligatoires.

---

## 13. Liens

- Brief Phase 1 : [`brief.md`](brief.md)
- PRD Phase 2 : [`prd.md`](prd.md)
- Architecture Phase 3 : [`architecture.md`](architecture.md)
- README pipeline : [`README.md`](README.md)
- WBS go-live v0.1.0 : [`../../WBS_GO_LIVE_v0.1.0.md`](../../WBS_GO_LIVE_v0.1.0.md)
- ADRs existants : [`docs/adr/`](../../adr/)
- Mémoires d'agent applicables (cf. [`README.md`](README.md))

🤖 Stories rédigées par Bob (Scrum Master) — Tier 1 acceptance pending @gilmry sign-off.

---
feature: refonte-ux-multi-role-acp
phase: architecture
phase_togaf: D (Technology)
agent_bmad: Winston (Architecte hexagonal)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 1.0
status: Signed by @gilmry 2026-05-20
signed_at: 2026-05-20
signed_by: "@gilmry"
brief_source: brief.md (Mary, v1.0 signé 2026-05-20)
prd_source: prd.md (John, v1.0 signé 2026-05-20)
adrs_inline: [ADR-0010, ADR-0011, ADR-0012, ADR-0013, ADR-0014, ADR-0015]
changelog:
  - "1.0 (2026-05-20) — SIGNÉE par @gilmry. Phase 3 verrouillée, Phase 4 (Stories Bob) débloquée. 8 BC DDD, 6 ADRs inline (ADR-0010 à ADR-0015), 9 migrations SQL, 18 endpoints REST, 7 composants Svelte 5, stratégie tests 3-niveaux."
---

# Architecture — Refonte UX multi-rôle + modèle ACP

## Méthode Maury — Phase TOGAF D (Technology)

> ✅ **Architecture SIGNÉE par @gilmry le 2026-05-20** — Phase 3 verrouillée, Phase 4 (Stories, Bob) débloquée. 8 BC DDD, 6 ADRs inline (ADR-0010 à ADR-0015), 9 migrations SQL, 18 endpoints REST nouveaux, 7 composants Svelte 5, stratégie tests 3-niveaux.
>
> Toute incohérence détectée post-signature → ADR de correction ou boucle de retour vers PRD/Brief, pas patch silencieux.

---

## 1. Vue d'ensemble

### 1.1 Principes invariables

1. **Hexagonal strict** : `domain/` n'importe ni `sqlx` ni `actix_web` ni aucun crate infra.
2. **Validate-before-compute** : tout use-case calcul commence par `building.assert_conformant()?` (cf. [[validate-before-compute]]).
3. **Modularité par ACP** : un module désactivé = endpoints 403 typé `ModuleDisabledError` + UI masquée + données archivées (jamais supprimées).
4. **Decimal partout** : aucun `f64` ne traverse une frontière monétaire (cf. ADR-0007 et [[no-f64-in-money]]).
5. **Result&lt;_, AppError&gt; typé** : aucun nouveau `Result<_, String>` introduit (cf. règle CRITICAL.md §4 + epic #555).
6. **Audit immuable** : plaintes/réponses/cahier des charges/évaluations/votes/PV → append-only (no edit, no delete).
7. **Tests 3-niveaux obligatoires** par PR : caractérisation FE + RED-GREEN-BLUE Vitest + multi-rôle Playwright (cf. [[fe-refactor-test-driven]]).

### 1.2 Bounded Contexts (cf. brief §5)

```
┌────────────────────────────────────────────────────────────────────┐
│                    Identity & Access + ACP                        │
│  Organization(0..1) → ACP(1..N) → Building(1..N) → Unit(1..N)    │
│  User → UserRoleAssignment(role, scope, valid_until?)             │
│  MagicLink(token, expires_at, scope, single_use?)                 │
│  Mandate(party_user_id, kind=lawyer|notary|amo, scope, valid_until)│
└─────────┬─────────────────────────────────────────────────────────┘
          │
          ├─→ Property Management ──────┐
          │   Building.is_conformant()  │
          │   Unit, EtatDate (Art.577)  │
          │                              │
          ├─→ Governance ───────────────┤
          │   Meeting(mode=hybrid)      │
          │   Resolution, Vote, Minutes │
          │   CdC, Commissaire          │
          │                              │
          ├─→ Accounting (PCMN BE) ─────┤
          │   Invoice (Encodeur)        │
          │   Expense, CallForFunds     │
          │     (Émetteur)              │
          │   VerificationCertificate   │
          │     (Commissaire)           │
          │                              │
          ├─→ Community ────────────────┤
          │   SEL, Poll, Notice         │
          │   SharedObject, Reservation │
          │   Syndic = Moderator (no    │
          │     personal participation) │
          │                              │
          ├─→ Maintenance & Ops ────────┤
          │   Ticket(kind=request|      │
          │     complaint), SyndicResp  │
          │   TechnicalSpec (versionnée)│
          │   ContractorEvaluation      │
          │                              │
          ├─→ Portfolio ────────────────┤
          │   Portfolio(user, org,      │
          │     buildings[], shared?)   │
          │                              │
          └─→ Module Registry (transv.) ┤
              acp_enabled_modules        │
              ModuleDisabledError 403    │
                                         │
   Cross-cutting: AuditEvent, Notification, I18n, Decimal, AppError
```

### 1.3 Vue déploiement (logique)

Aucun changement infra : 1 backend Actix-web, 1 PostgreSQL, 1 frontend Astro+Svelte 5, Traefik. La refonte est applicative (domaine + UX), pas infra. La PWA Contractor magic-link est servie par le **même frontend Astro** (route publique `/c/<token>`), pas une app séparée.

---

## 2. Diagrammes d'agrégat

### 2.1 BC Identity & Access + ACP (refacto majeur)

```
[Organization] (Aggregate Root, optional — cabinet syndic OU absent si ACP auto-gérée)
   │ 0..1
   ▼
[ACP] (Aggregate Root, NEW)
   │ name, slug, address, vat?, legal_status="copropriete_belge"
   │ organization_id NULLABLE FK
   │ ─ enabled_modules ←→ acp_enabled_modules (M:N)
   │
   │ 1..N
   ▼
[Building] (Entity, refacto : organization_id → acp_id)
   │ name, address, total_units (declared)
   │ acp_id FK NOT NULL (post-backfill)
   │ ─ is_conformant() := count(units)==total_units && SUM(units.quota)==1000
   │
   │ 1..N
   ▼
[Unit] (Entity)
   reference, owners[], quota (Decimal, scale=4)

[User] (Aggregate Root)
   │
   │ 1..N
   ▼
[UserRoleAssignment] (Entity)
   role (admin|syndic|accountant.encodeur|accountant.emetteur|owner|cdc|commissaire|contractor|moderator|lawyer|notary|amo|warden)
   scope (organization|acp|building|portfolio)
   scope_id UUID
   valid_until TIMESTAMPTZ NULLABLE (NULL = permanent)
   delegated_from_user_id NULLABLE (cas délégation temporaire)

[MagicLink] (Aggregate Root, NEW)
   token (uuid v4, signé HMAC)
   subject_user_id (Contractor invité)
   scope (ticket_id | quote_id | invoice_id | evaluation_id)
   expires_at TIMESTAMPTZ
   single_use BOOL
   consumed_at TIMESTAMPTZ NULLABLE

[Mandate] (Aggregate Root, NEW)
   party_user_id (Avocat/Notaire/AMO/Architecte/BET)
   kind (lawyer | notary | amo | architect | bet)
   issued_by_user_id (Syndic ou AG)
   scope_acp_id FK
   scope_building_id NULLABLE
   purpose TEXT
   valid_until TIMESTAMPTZ NOT NULL
   audit_event_id FK (signature trace)
```

### 2.2 BC Governance (extension AG hybride + Art. 3.84-3.89 CC)

```
[Meeting] (Aggregate Root)
   acp_id FK
   building_id FK NULLABLE (AG ACP-wide vs AG building-spécifique)
   kind (AGO | AGE | ConseilCdC | ConseilSyndic)
   mode (in_person | remote | hybrid) ← NEW
   scheduled_at, convened_at, completed_at?
   status (Draft | Convened | InProgress | Completed | Cancelled)
   minutes_pdf_id NULLABLE
   ─ assert_can_complete() ?ConvocationsSent + ?QuorumMet + ?ResolutionsClosed + ?MinutesSigned2x

   │ 1..N
   ▼
[Resolution] (Entity)
   meeting_id FK
   kind (Decision | EvaluationContractors_AUTO | ApprovalAccounts | …)
   text, voting_majority_required (Decimal %)
   is_auto_generated BOOL (true pour EvaluationContractors_AUTO → non retirable)
   closed_at? NULLABLE

   │ 1..N
   ▼
[Vote] (Entity)
   resolution_id FK
   voter_user_id FK
   value (For | Against | Abstain)
   weight (Decimal — quota du voter)
   cast_at TIMESTAMPTZ
   auth_method (presence | proxy | itsme | eid)  ← NEW, contrainte AG distant/hybride
   proxy_for_user_id FK NULLABLE

[Minutes] (Aggregate Root, NEW formalisation)
   meeting_id FK
   content_html, content_pdf_id FK
   president_signature_id FK NOT NULL (eIDAS qualifié)
   secretary_signature_id FK NOT NULL (eIDAS qualifié)
   ─ assert_complete() ?content + ?2 signatures

[CdC] (Aggregate Root, conseil de copropriété — Art. 3.87 §1 CC)
   acp_id FK
   members []user_id (élus en AG)
   elected_at, mandate_until
   ─ create_alert(text, severity, target=AG_next)

[CommissaireAuxComptes] (Aggregate Root, Art. 3.88 CC)
   acp_id FK
   user_id FK (peut être owner non-syndic)
   appointed_by_meeting_id FK
   mandate_until
   ─ sign_certificate(financial_period_id) → VerificationCertificate
```

### 2.3 BC Accounting (split sous-rôles + validate-before-compute)

```
[Invoice] (Aggregate Root, NEW — créable par Encodeur)
   acp_id, building_id NULLABLE
   supplier_id, amount_ht Decimal, vat_rate Decimal, amount_ttc Decimal (calculé)
   document_id FK, status (Draft | Validated | Paid | Disputed)
   created_by_user_id FK (Encodeur)
   ─ permission: accountant.encodeur ou accountant.emetteur ou syndic ou admin

[Expense] (Aggregate Root — créable par Émetteur)
   acp_id, building_id, invoice_id FK NULLABLE
   amount Decimal, distribution_rule_id FK
   ─ permission: accountant.emetteur ou syndic ou admin
   ─ pre-check: building.assert_conformant()?

[CallForFunds] (Aggregate Root — créable par Émetteur)
   acp_id, building_id, period_id FK
   total_amount Decimal, allocations [{unit_id, amount Decimal}]
   ─ permission: accountant.emetteur ou syndic ou admin
   ─ pre-check: building.assert_conformant()?
   ─ allocations calculated by use-case allocate_call_for_funds(building, period)

[VerificationCertificate] (Aggregate Root, NEW)
   commissaire_user_id FK
   financial_period_id FK
   findings TEXT
   signature_id FK (eIDAS qualifié)
   issued_at
   ─ enforced PRE-clôture comptes annuels (cf. workflow)
```

### 2.4 BC Maintenance & Operations (extension)

```
[Ticket] (Aggregate Root, refacto)
   acp_id, building_id, unit_id NULLABLE
   reporter_user_id (Owner|CdC|Warden)
   kind (Request | Complaint)  ← NEW Complaint
   severity (Low | Normal | High | Critical) ← NEW
   incident_date NULLABLE
   evidence_attachments []document_id ← NEW
   witnesses []user_id ← NEW
   status (Open | Assigned | InProgress | Resolved | Closed)
   sla_due_at? (calculé from severity + policy)
   escalated_to_cdc_at?

   │ 1..N
   ▼
[SyndicResponse] (Entity, NEW)
   ticket_id FK
   responder_user_id FK (Syndic)
   response_text, action_proposed
   responded_at
   ─ append-only (audit immuable INV-24)

[TechnicalSpec] (Aggregate Root, NEW)
   acp_id, building_id
   version (semver)
   scope, deliverables[], deadlines, criteria, attachments []document_id
   signatures [{user_id, role, signed_at}] (ACP/Syndic/AMO requis)
   status (Draft | Approved | Archived)

[ContractorEvaluation] (Aggregate Root, NEW)
   contractor_user_id FK
   technical_spec_id FK NOT NULL  ← INV-21 : refus 422 sans spec
   scope_meeting_id FK (point AGO obligatoire)
   scores {quality, timeliness, communication, value_for_money}
   tickets_linked []ticket_id (plaintes ayant motivé l'évaluation)
   evaluated_at
   ─ append-only
```

### 2.5 BC Portfolio (NEW, entité backend)

```
[Portfolio] (Aggregate Root, NEW — cf. ADR-0011)
   id UUID
   owner_user_id FK
   organization_id FK NULLABLE (cabinet)
   name (str)
   shared_with_user_ids []user_id (équipe cabinet)
   created_at, updated_at

[PortfolioBuilding] (Entity de liaison M:N)
   portfolio_id FK
   building_id FK
   is_favorite BOOL (star/épinglage)
   added_at
```

### 2.6 BC Module Registry (cross-cutting NEW)

```
[AcpEnabledModule] (Entity)
   acp_id FK
   module_name ENUM (Community | Ticketing | Accounting | Governance | Maintenance | Portfolio | Maintenance | Identity)
   enabled_at, enabled_by_user_id
   plan VARCHAR (free | starter | pro | enterprise) — informatif, pas RBAC
   archived_at NULLABLE (désactivation = archivage, jamais delete)
```

Module guard middleware (Actix) sur chaque route : si `req.acp_id` mappé à un module désactivé → 403 `ModuleDisabledError{module, acp_id}`.

---

## 3. Ports & Adapters (Hexagonal)

### 3.1 Nouveaux Ports (traits Rust)

```rust
// backend/src/application/ports/acp_repository.rs
#[async_trait]
pub trait AcpRepository: Send + Sync {
    async fn create(&self, acp: NewAcp) -> Result<Acp, AppError>;
    async fn find_by_id(&self, id: AcpId) -> Result<Option<Acp>, AppError>;
    async fn list_for_user(&self, user_id: UserId, scope: ListScope)
        -> Result<Vec<Acp>, AppError>;
    async fn update(&self, acp: Acp) -> Result<Acp, AppError>;
    async fn archive(&self, id: AcpId) -> Result<(), AppError>;
}

// backend/src/application/ports/portfolio_repository.rs
#[async_trait]
pub trait PortfolioRepository: Send + Sync {
    async fn create(&self, p: NewPortfolio) -> Result<Portfolio, AppError>;
    async fn find_by_id(&self, id: PortfolioId)
        -> Result<Option<Portfolio>, AppError>;
    async fn list_for_user(&self, user_id: UserId)
        -> Result<Vec<Portfolio>, AppError>;
    async fn add_building(&self, p_id: PortfolioId, b_id: BuildingId,
        is_favorite: bool) -> Result<(), AppError>;
    async fn share_with(&self, p_id: PortfolioId, user_id: UserId)
        -> Result<(), AppError>;
}

// backend/src/application/ports/magic_link_repository.rs
#[async_trait]
pub trait MagicLinkRepository: Send + Sync {
    async fn issue(&self, ml: NewMagicLink) -> Result<MagicLink, AppError>;
    async fn validate_and_consume(&self, token: &str)
        -> Result<MagicLinkClaims, AppError>;  // 1-shot if single_use
}

// backend/src/application/ports/mandate_repository.rs
// backend/src/application/ports/technical_spec_repository.rs
// backend/src/application/ports/contractor_evaluation_repository.rs
// backend/src/application/ports/module_registry.rs
//   fn is_enabled(acp_id, module) -> bool
//   fn enable(acp_id, module, actor, plan) -> Result<(), AppError>
//   fn disable(acp_id, module, actor) -> Result<(), AppError>  (archive)
// backend/src/application/ports/electronic_signature_provider.rs (cf. ADR-0014)
//   fn request_signature(...) -> Result<SignatureRequestId, AppError>
//   fn fetch_signature(...) -> Result<QualifiedSignature, AppError>
```

### 3.2 Adapters

- `infrastructure/database/repositories/acp_repository_impl.rs` (sqlx PostgreSQL)
- `infrastructure/database/repositories/portfolio_repository_impl.rs`
- `infrastructure/database/repositories/magic_link_repository_impl.rs` (HMAC + scrypt sur token)
- `infrastructure/database/repositories/mandate_repository_impl.rs`
- `infrastructure/database/repositories/technical_spec_repository_impl.rs`
- `infrastructure/database/repositories/contractor_evaluation_repository_impl.rs`
- `infrastructure/database/repositories/module_registry_impl.rs`
- `infrastructure/external/signature_provider_eid.rs` (cf. ADR-0014 prestataire choisi)
- `infrastructure/web/middleware/module_guard.rs` (NEW — bloque routes par module désactivé)
- `infrastructure/web/middleware/scope_guard.rs` (NEW — bloque accès cross-ACP/portfolio)

### 3.3 Use cases impactés (extension/refacto)

| Use-case | Statut | Impact #433 Decimal | Impact #555 Result&lt;_, String&gt; |
|---|---|---|---|
| `acp_use_cases.rs` (NEW) | Création | — | OK natif AppError |
| `portfolio_use_cases.rs` (NEW) | Création | — | OK natif AppError |
| `building_use_cases.rs` | Refacto FK | — | À migrer si touché |
| `list_buildings_use_case.rs` | Refacto filtrage rôle | — | À migrer si touché |
| `meeting_use_cases.rs` | `assert_can_complete()` + mode hybrid | — | **Oui** (cf. #554 sortie ré-intégrée ici) |
| `vote_use_cases.rs` | auth_method check itsme/eID | — | À migrer si touché |
| `expense_use_cases.rs` | `assert_conformant()` pre-check | **Oui** (cluster #433) | **Oui** (cluster #555) |
| `call_for_funds_use_cases.rs` | `assert_conformant()` pre-check | **Oui** | **Oui** |
| `invoice_use_cases.rs` (NEW) | Encodeur | **Oui dès création** | OK natif AppError |
| `verification_certificate_use_cases.rs` (NEW) | Commissaire signe | — | OK natif AppError |
| `ticket_use_cases.rs` | kind=complaint + SLA + escalade | — | À migrer si touché |
| `technical_spec_use_cases.rs` (NEW) | versionning + signatures | — | OK natif AppError |
| `contractor_evaluation_use_cases.rs` (NEW) | requires TechnicalSpec | — | OK natif AppError |
| `magic_link_use_cases.rs` (NEW) | issue+consume | — | OK natif AppError |
| `mandate_use_cases.rs` (NEW) | issue + valid_until check | — | OK natif AppError |
| `module_registry_use_cases.rs` (NEW) | enable/disable + audit | — | OK natif AppError |

**Convention #433 ↔ #555** : tout use-case « À migrer si touché » fait les **2 migrations simultanément** dans **1 PR** (cf. PRD §7 et meta-comment #433).

---

## 4. ADRs inline

### ADR-0010 : ACP comme racine d'agrégat distincte d'Organization

- **Status** : Proposed (signature avec architecture)
- **Date** : 2026-05-20
- **Track** : Domain modeling / DDD
- **Authors** : Winston (Architecte hexagonal) + @gilmry sign-off
- **Related** : ADR-0002 (Hexagonal), brief §2.B + §6, PRD FR1-FR4

#### Context

Le modèle actuel `Building.organization_id` saute le niveau **ACP** (Association des Copropriétaires) requis par le droit belge (Art. 3.84-3.89 Code Civil). Une ACP a une personnalité juridique propre, indépendante du syndic qui la gère. Conséquences observées :

- Un cabinet syndic gère plusieurs ACPs distinctes → `organization_id` confond cabinet et ACP
- Une ACP auto-gérée (sans syndic professionnel) → aucun cabinet correspondant
- Une ACP qui change de cabinet → migration data sans changement légal de personne morale

#### Decision

Introduire **`ACP` comme racine d'agrégat distincte** dans le BC Identity & Access. La hiérarchie devient :

```
Organization (0..1, cabinet syndic professionnel)
   └── ACP (1..N) [Aggregate Root]
        └── Building (1..N)
             └── Unit (1..N)
```

- `ACP.organization_id` est **NULLABLE** (ACP auto-gérée → null)
- `Building.organization_id` est **supprimé** et remplacé par `Building.acp_id NOT NULL`
- Migration data : intermédiaire `Building.acp_id NULLABLE`, backfill via création d'ACP miroir par cabinet existant, puis `NOT NULL`
- `ACP` porte les attributs juridiques : raison sociale, n° BCE, statut légal, adresse siège
- Toutes les opérations cross-building (AG, comptabilité, modules) sont scopées par `acp_id`

#### Consequences

**Positive** :
- Conformité juridique Art. 3.84 CC native dans le modèle
- ACP auto-gérée supportée sans hack
- Changement de cabinet syndic = update `ACP.organization_id` sans impact sur historique
- Cluster #433 Decimal et epic #555 Result trouvent leur frontière d'agrégat naturelle

**Negative** :
- Migration data lourde : ~28 tables référencent `organization_id` ou `building_id`
- Risk de building orphelin (mitigé par backfill obligatoire + audit)
- API publique change `organization_id` → `acp_id` (rupture mineure, internal seulement à v0.1.0)

**Neutral** :
- L'`Organization` reste utile pour modéliser le **cabinet syndic** (employés, mandats, facturation interne SaaS)

#### Alternatives Considered

- **Garder `Building.organization_id` et faire `Organization` polymorphe (cabinet OU ACP auto-gérée)** : rejeté, viole SRP, complique RBAC
- **Renommer simplement `Organization` en `ACP`** : rejeté, perd la distinction cabinet vs ACP (essentielle pour gestion multi-clients)
- **Introduire ACP sans modifier Building** : rejeté, laisse incohérence FK long-terme

#### Enforcement

- Migration SQL avec étape intermédiaire NULLABLE + script backfill + script rollback
- Test `@negative` : insert `Building` sans `acp_id` post-migration → 422
- Test `@security` : syndic cabinet B tente accès ACP cabinet A → 403

#### References

- Brief Phase 1 §2.B « modèle juridique faux » + §6 hiérarchie cible
- PRD Phase 2 FR1-FR4
- Code Civil belge Art. 3.84-3.89 (Loi du 18 juin 2018, MB 02.07.2018)

---

### ADR-0011 : Portefeuille comme entité backend (vs préférence UI localStorage)

- **Status** : Proposed
- **Date** : 2026-05-20
- **Track** : Domain modeling / UX
- **Authors** : Winston + @gilmry sign-off
- **Related** : brief §4 C5, PRD FR36-FR38

#### Context

Le brief identifie le besoin de **portefeuilles immeubles** pour les cabinets multi-ACP : un gestionnaire veut épingler 5 immeubles, partager une liste avec son équipe, retrouver sa sélection après changement de poste. Deux options :

- **A. UI-only** : preferences localStorage du navigateur
- **B. Backend entité** : table `portfolios` + endpoints REST + partage équipe

#### Decision

**Portfolio = entité backend** (option B).

```sql
CREATE TABLE portfolios (
  id UUID PRIMARY KEY,
  owner_user_id UUID NOT NULL REFERENCES users(id),
  organization_id UUID NULL REFERENCES organizations(id),
  name VARCHAR(120) NOT NULL,
  created_at, updated_at
);
CREATE TABLE portfolio_buildings (
  portfolio_id UUID REFERENCES portfolios(id) ON DELETE CASCADE,
  building_id UUID REFERENCES buildings(id),
  is_favorite BOOL DEFAULT false,
  added_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (portfolio_id, building_id)
);
CREATE TABLE portfolio_shares (
  portfolio_id UUID REFERENCES portfolios(id) ON DELETE CASCADE,
  user_id UUID REFERENCES users(id),
  granted_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (portfolio_id, user_id)
);
```

#### Consequences

**Positive** :
- Portabilité multi-device (navigateur change → portfolio préservé)
- Partage équipe natif (cabinet syndic)
- Audit possible (qui a partagé quoi avec qui)
- Couche d'autorisation backend (RBAC cohérent — pas de fuite cross-cabinet via localStorage)

**Negative** :
- Léger surcoût DB (3 tables + RLS/scope_guard)
- Nécessite migration de données si users existants avaient déjà des localStorage favoris (faible — v0.1.0 pre-release)

**Neutral** :
- Cache côté FE (Svelte store) reste pertinent pour perf (lecture instantanée après login)

#### Alternatives Considered

- **localStorage only** : rejeté, pas de partage équipe, perte au reset navigateur
- **Cookie chiffré côté serveur sans table** : rejeté, taille limitée + pas d'audit
- **Hybrid (backend + localStorage cache)** : adopté implicitement (FE cache du résultat backend)

#### Enforcement

- Test `@security` : user A tente accès portfolio B non partagé → 403
- Test `@happy` : partage portfolio entre 2 users même org → OK ; entre 2 orgs différentes → 403
- Test `@edge` : portfolio avec 100 buildings → autocomplete < 200ms (cf. PRD NFR1)

---

### ADR-0012 : Convention `data-testid="<entity>-<action>"` systématique

- **Status** : Proposed
- **Date** : 2026-05-20
- **Track** : Frontend / Testing
- **Authors** : Winston + @gilmry sign-off
- **Related** : mémoire [[data-testid-systematic]], PRD FR44-FR45

#### Context

Les sélecteurs Playwright actuels mélangent `text=`, `nth-child`, `role=button`. Conséquences :
- Tests cassent à chaque refacto i18n (label change)
- Sélecteurs nth-child cassent à chaque réordonnancement DOM
- Multi-rôle E2E : helpers shared difficiles à factoriser

#### Decision

**Tout élément interactif** (button, link, input, select, dialog, banner) du périmètre refonte expose :

```html
<button data-testid="building-edit" ...>
<input data-testid="login-email" ...>
<a data-testid="navigation-portfolio" ...>
```

Convention :
- `<entity>` = nom métier en kebab-case (`building`, `acp`, `meeting`, `vote`, `portfolio`, `ticket`, `magic-link`)
- `<action>` = verbe en kebab-case (`edit`, `create`, `submit`, `cancel`, `select`, `delete`, `share`)
- Pas d'ID dynamique dans le testid (sauf cas spécifique table → `<entity>-<action>-<id>` autorisé pour tables itérables, ex `building-edit-{{id}}`)

Sélecteurs Playwright autorisés : `getByTestId()` **uniquement**. Sélecteurs `text=`, `nth-child`, `role=` → warning CI (sauf cas a11y explicite).

#### Consequences

**Positive** :
- Tests stables aux refactos i18n et DOM
- Helpers shared simplifiés (`page.getByTestId('login-email').fill(email)`)
- Audit a11y compatible (testid ≠ aria-label, complémentaire)

**Negative** :
- Léger gonflement HTML (~5-10 bytes par élément interactif)
- Discipline review nécessaire (testids manquants doivent être détectés en PR)

#### Enforcement

- ESLint plugin custom `koprogo-testid-required` sur composants Svelte interactifs (warning d'abord, erreur post-slice 5)
- CI gate : `grep -r "getByText\|nth-child\|locator\(" frontend/tests/e2e/` doit décroître à chaque slice
- PR review template inclut « tous les nouveaux éléments interactifs ont un data-testid ? »

---

### ADR-0013 : Arborescence tests caractérisation + refonte

- **Status** : Proposed
- **Date** : 2026-05-20
- **Track** : Frontend / Testing strategy
- **Authors** : Winston + @gilmry sign-off
- **Related** : mémoire [[fe-refactor-test-driven]], PRD FR43, §6

#### Context

3 niveaux de tests FE coexistent durant la refonte :
1. **Caractérisation** : fige le comportement actuel (régression safety net)
2. **RED-GREEN-BLUE Vitest** : TDD composants nouveaux/refactorés
3. **Playwright multi-rôle** : intégration end-to-end avec rôles métier corrects

Sans arborescence claire, risque de mélange (un test « caractérisation » qui couvre en fait un comportement cible refonte).

#### Decision

```
frontend/tests/
├── e2e/
│   ├── characterization/         ← Niveau 1 : fige existant, IMMUABLE durant refonte
│   │   ├── 00-login-and-dashboards.spec.ts
│   │   ├── 01-building-creation-flow.spec.ts
│   │   ├── 02-ag-full-cycle.spec.ts
│   │   ├── 03-expense-and-payment.spec.ts
│   │   ├── 04-owner-view.spec.ts
│   │   └── 05-notifications-sync.spec.ts
│   │
│   ├── refonte-ux/               ← Niveau 3 : cibles refonte, MUTABLE par slice
│   │   ├── slice-1-acp-refacto/
│   │   ├── slice-2-selector-banner/
│   │   ├── slice-3-magic-link-pwa/
│   │   ├── slice-4-ag-hybride/
│   │   └── slice-5-modularity/
│   │
│   ├── helpers/                  ← Shared, multi-rôle (cf. FR44)
│   │   ├── auth.ts (loginAsSyndic, loginAsAdmin, loginAsOwner, loginAsContractorMagicLink, ...)
│   │   ├── building.ts (loginAsSyndicWithBuilding, …)
│   │   └── pause.ts
│   │
│   └── (specs existantes legacy à migrer ou archiver)
│
└── unit/                         ← Niveau 2 : Vitest RED-GREEN-BLUE
    └── src/lib/**/__tests__/*.test.ts
    └── src/components/**/__tests__/*.test.ts
```

**Règles** :
- `characterization/` **gelé** : aucun commit ne le modifie pendant la refonte sauf bug évident dans le test lui-même (justifié en PR description)
- `refonte-ux/slice-N/` créé en début de slice, gardé après la slice (régression long-terme)
- Vitest unit colocalisés `**/__tests__/*.test.ts` (proche du code testé)

#### Consequences

**Positive** :
- Séparation nette régression safety net vs cibles refonte
- CI peut runner les 3 niveaux en parallèle (`npm run test:characterization && npm run test:refonte && npm run test:unit`)
- Audit facile « as-tu écrit la caractérisation avant la slice ? » (git log sur `characterization/` antérieur à `refonte-ux/slice-N/`)

**Negative** :
- 2× spécifications E2E à maintenir si fonctionnalité présente avant ET après refonte (acceptable : la caractérisation se supprime post-slice si elle devient redondante avec refonte-ux, après audit)

#### Enforcement

- Gate Phase 4 (Stories) : pré-requis création slice = caractérisation correspondante VERTE
- CI : si `frontend/tests/e2e/characterization/` modifié dans une PR labellisée `slice-N`, le diff est demandé en review explicite
- Convention naming `slice-N-<nom-court>` enforce-d par lint script

---

### ADR-0014 : Signature électronique eIDAS — prestataire

- **Status** : Proposed (sous-décision à finaliser en story dédiée)
- **Date** : 2026-05-20
- **Track** : Compliance / External integration
- **Authors** : Winston + @gilmry sign-off
- **Related** : brief §6 Governance + Mandat, PRD FR16, FR20, FR25

#### Context

Les artefacts suivants requièrent signature **électronique qualifiée eIDAS** (Règlement UE 910/2014) :
- PV d'AG : 2 signatures (président + secrétaire) — Art. 3.87 CC
- Certificat du commissaire aux comptes (clôture comptes annuels) — Art. 3.88 CC
- Mandat avocat/notaire/AMO (émission formelle) — pratique professionnelle
- Cahier des charges TechnicalSpec (versionnage signé ACP/Syndic/AMO)
- Vote distant AG hybride : auth forte (≠ signature, mais même infrastructure idP envisageable)

Coût et UX divergent fortement selon le prestataire.

#### Decision

**Adopter une abstraction `ElectronicSignatureProvider` (port hexagonal)** avec **3 adapters cibles** :

1. **`signature_provider_eid.rs`** — eID belge (BeID) via federal identity provider (FAS, ItsMe). Gratuit citoyens belges, niveau qualifié, conforme eIDAS. **Adapter par défaut pour v0.1.0**.
2. **`signature_provider_itsme.rs`** — itsme.be, auth forte + signature qualifiée mobile. Tarif au pull (~0,30-0,50€/signature). Recommandé pour vote distant + signatures owners non BE-résidents.
3. **`signature_provider_universign.rs`** — Universign (EU). Tarif abonnement + au pull. Réservé prestataires externes (avocats/notaires/AMO) qui n'ont pas eID belge.

Le choix d'adapter par signature est **stratégique au niveau ACP** (préférence cabinet) avec **fallback Universign** pour parties non-BE.

#### Consequences

**Positive** :
- Port abstrait → switch prestataire sans modifier domain/use-cases
- Gratuité eID par défaut → coût v0.1.0 maîtrisé
- itsme couvre les cas mobile-first owners
- Universign garantit support cas edge (non-BE)

**Negative** :
- 3 adapters à maintenir (mitigé : minimal SDK calls + tests d'intégration mockés)
- Dépendance vendor itsme/Universign (mitigé : abstraction port → switchable)

**Neutral** :
- Coût opérationnel à monitorer (KPI v0.1.0 : € par AG, € par certificat)

#### Alternatives Considered

- **DocuSign EU** : rejeté, coût élevé, non conforme eIDAS "qualified" stricto sensu
- **Adobe Sign EU** : idem
- **Self-signed PGP** : rejeté, pas eIDAS qualifié, non opposable en justice belge
- **eID belge uniquement** : rejeté, exclut non-BE résidents

#### Enforcement

- Test `@happy` : signature eID 2× sur PV → Meeting passe à status Completed
- Test `@negative` : tentative `Meeting.complete()` sans 2 signatures → 422 typé
- Test `@security` : tentative signature par user non-président/secrétaire → 403
- Audit : tout `QualifiedSignature` persisté avec `signed_at`, `provider`, `subject_user_id`, `document_id`, hash document

#### References

- Règlement (UE) 910/2014 (eIDAS)
- Loi du 21 juillet 2016 (eIDAS en droit belge)
- FAS / CSAM federal IdP : https://iamapps.belgium.be
- itsme : https://www.itsme-id.com
- PRD FR16/FR20/FR25

---

### ADR-0015 : Modularité par ACP — module registry

- **Status** : Proposed
- **Date** : 2026-05-20
- **Track** : Product / Architecture transversale
- **Authors** : Winston + @gilmry sign-off
- **Related** : mémoire [[koprogo-modular-toolbox]], brief §7 C20-C22, PRD FR39-FR42

#### Context

Le brief établit que KoproGo est une **boîte à outils modulaire** : une ACP active uniquement les modules dont elle a besoin (Communauté seule, Ticketing seul, Compta seule, AG seule, Gestion complète, ou combinaison). Dialectique marketing « gardez ce qui marche, prenez ce qui vous manque ».

Sans architecture explicite, risque de couplage caché entre BCs.

#### Decision

**Table `acp_enabled_modules` + middleware `ModuleGuard` + UI conditionnelle**.

```sql
CREATE TABLE acp_enabled_modules (
  acp_id UUID REFERENCES acps(id) ON DELETE CASCADE,
  module_name VARCHAR(32) NOT NULL,
  enabled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  enabled_by_user_id UUID NOT NULL REFERENCES users(id),
  plan VARCHAR(16) NOT NULL DEFAULT 'free',
  archived_at TIMESTAMPTZ NULL,
  PRIMARY KEY (acp_id, module_name)
);
```

Modules nommés : `community`, `ticketing`, `accounting`, `governance`, `maintenance`, `portfolio`, `identity` (toujours actif).

**Activation/désactivation** :
- `community`, `ticketing`, `maintenance`, `portfolio` → activable par admin SaaS sur demande syndic
- `accounting`, `governance` → activable **par vote AG** ≥ 50% (impact légal)
- `identity` → toujours actif (cœur produit)

**Désactivation = archivage** (`archived_at != NULL`, jamais DELETE row). Données existantes (tickets, AGs, écritures) restent en DB mais inaccessibles. Réactivation possible → `archived_at = NULL`, données restaurées intactes.

**Middleware Actix** `ModuleGuard` :
```rust
async fn module_guard(req: ServiceRequest, module: ModuleName) -> Result<ServiceRequest, Error> {
    let acp_id = req.extensions().get::<AcpScope>()
        .ok_or(AppError::Unauthorized)?.acp_id;
    if !module_registry.is_enabled(acp_id, module).await? {
        return Err(AppError::ModuleDisabled { module, acp_id }.into());
    }
    Ok(req)
}
```

**UI** : composant `<ModuleGate module="community">…</ModuleGate>` Svelte 5 qui masque le bloc si désactivé. Menus de Navigation conditionnels via store `enabled_modules`.

#### Consequences

**Positive** :
- Adoption modulaire : ACP teste 1 module sans s'engager sur tout
- Pricing modulable (futur : module premium)
- Conformité légale ciblée (ACP sans obligation PCMN n'active pas `accounting`)
- Analytics produit (taux d'adoption par module)

**Negative** :
- Garde-fou à mettre sur **tous** les endpoints non-identity (audit complet)
- Risque de couplage caché si un module en a vraiment besoin d'un autre désactivé (ex : `accounting` qui nécessite des données `governance` pour budget AG). À auditer en story dédiée par BC.

**Neutral** :
- Couche middleware ajoute léger overhead (mitigé par cache LRU `is_enabled(acp_id, module)`)

#### Enforcement

- Test `@security` 4× (1 par module désactivable) : route correspondante → 403 `ModuleDisabledError`
- Test `@happy` : activation/désactivation/réactivation → données intactes
- Test `@negative` : tentative `module.disable('accounting')` sans vote AG → 403
- Test `@edge` : ACP avec 0 modules activés (sauf identity) → onboarding s'affiche

#### References

- Mémoire [[koprogo-modular-toolbox]] (« gardez ce qui marche, prenez ce qui vous manque »)
- Brief §7 C20-C22, INV-25/26/27
- PRD FR39-FR42, slice 5

---

## 5. Migrations SQL

### 5.1 Ordre d'exécution (Slice 1, atomic per file)

```
backend/migrations/
├── 20260601_010000_create_acps.sql               (Slice 1)
├── 20260601_020000_add_buildings_acp_id.sql      (Slice 1, NULLABLE)
├── 20260601_030000_backfill_buildings_acp_id.sql (Slice 1, script)
├── 20260601_040000_buildings_acp_id_not_null.sql (Slice 1, ALTER)
├── 20260601_050000_create_portfolios.sql         (Slice 2)
├── 20260605_010000_create_magic_links.sql        (Slice 3)
├── 20260605_020000_create_mandates.sql           (Slice 3)
├── 20260605_030000_extend_user_role_assignments.sql (Slice 3, valid_until)
├── 20260605_040000_extend_tickets_complaint.sql  (Slice 3, kind/severity/evidence)
├── 20260605_050000_create_syndic_responses.sql   (Slice 3)
├── 20260605_060000_create_technical_specs.sql    (Slice 3)
├── 20260605_070000_create_contractor_evaluations.sql (Slice 3)
├── 20260610_010000_extend_meetings_hybrid.sql    (Slice 4)
├── 20260610_020000_extend_votes_auth_method.sql  (Slice 4)
├── 20260610_030000_create_minutes.sql            (Slice 4)
├── 20260610_040000_create_verification_certificates.sql (Slice 4)
├── 20260615_010000_split_accountant_roles.sql    (Slice 4, sous-rôles via UserRoleAssignment.role string)
├── 20260620_010000_create_acp_enabled_modules.sql (Slice 5)
```

### 5.2 Migration emblématique : ACP refacto (Slice 1)

```sql
-- 20260601_010000_create_acps.sql
CREATE TABLE acps (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID NULL REFERENCES organizations(id),
  name VARCHAR(160) NOT NULL,
  slug VARCHAR(80) NOT NULL UNIQUE,
  legal_status VARCHAR(32) NOT NULL DEFAULT 'copropriete_belge',
  bce_number VARCHAR(20) NULL,
  address_street VARCHAR(200) NOT NULL,
  address_postal_code VARCHAR(10) NOT NULL,
  address_city VARCHAR(100) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_acps_organization_id ON acps(organization_id);

-- 20260601_020000_add_buildings_acp_id.sql
ALTER TABLE buildings ADD COLUMN acp_id UUID NULL REFERENCES acps(id);
CREATE INDEX idx_buildings_acp_id ON buildings(acp_id);

-- 20260601_030000_backfill_buildings_acp_id.sql (executable Rust binaire OU SQL)
-- Pour chaque organization_id distinct dans buildings :
--   1. Créer une ACP miroir (name = organization.name, organization_id = org.id)
--   2. UPDATE buildings SET acp_id = nouvelle_acp_id WHERE organization_id = org.id
-- Audit : log {organization_id, new_acp_id, buildings_count} dans audit_events

-- 20260601_040000_buildings_acp_id_not_null.sql
-- Pré-check : SELECT COUNT(*) FROM buildings WHERE acp_id IS NULL → doit être 0
ALTER TABLE buildings ALTER COLUMN acp_id SET NOT NULL;
ALTER TABLE buildings DROP COLUMN organization_id;
```

**Rollback prévu** : `20260601_040000_DOWN.sql` ré-ajoute `organization_id`, backfill inverse, drop `acp_id`. Tests `@negative` couvrent ce parcours.

### 5.3 Autres migrations clés (squelette)

```sql
-- 20260605_010000_create_magic_links.sql
CREATE TABLE magic_links (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  token_hash CHAR(64) NOT NULL UNIQUE,  -- scrypt(token + pepper)
  subject_user_id UUID NOT NULL REFERENCES users(id),
  scope_kind VARCHAR(32) NOT NULL,  -- ticket | quote | invoice | evaluation
  scope_id UUID NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  single_use BOOL NOT NULL DEFAULT true,
  consumed_at TIMESTAMPTZ NULL,
  issued_by_user_id UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_magic_links_subject ON magic_links(subject_user_id);
CREATE INDEX idx_magic_links_expires ON magic_links(expires_at) WHERE consumed_at IS NULL;

-- 20260605_020000_create_mandates.sql
CREATE TABLE mandates (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  party_user_id UUID NOT NULL REFERENCES users(id),
  kind VARCHAR(16) NOT NULL CHECK (kind IN ('lawyer','notary','amo','architect','bet')),
  issued_by_user_id UUID NOT NULL REFERENCES users(id),
  acp_id UUID NOT NULL REFERENCES acps(id),
  building_id UUID NULL REFERENCES buildings(id),
  purpose TEXT NOT NULL,
  valid_until TIMESTAMPTZ NOT NULL,
  audit_event_id UUID NULL REFERENCES audit_events(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_mandates_party ON mandates(party_user_id);
CREATE INDEX idx_mandates_acp_active ON mandates(acp_id) WHERE valid_until > now();

-- 20260605_030000_extend_user_role_assignments.sql
ALTER TABLE user_role_assignments
  ADD COLUMN valid_until TIMESTAMPTZ NULL,
  ADD COLUMN delegated_from_user_id UUID NULL REFERENCES users(id);
CREATE INDEX idx_ura_valid_until ON user_role_assignments(valid_until)
  WHERE valid_until IS NOT NULL;

-- 20260620_010000_create_acp_enabled_modules.sql
CREATE TABLE acp_enabled_modules (
  acp_id UUID REFERENCES acps(id) ON DELETE CASCADE,
  module_name VARCHAR(32) NOT NULL
    CHECK (module_name IN ('community','ticketing','accounting','governance','maintenance','portfolio','identity')),
  enabled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  enabled_by_user_id UUID NOT NULL REFERENCES users(id),
  plan VARCHAR(16) NOT NULL DEFAULT 'free',
  archived_at TIMESTAMPTZ NULL,
  PRIMARY KEY (acp_id, module_name)
);
```

**Tests `@negative` migration** : chaque migration a un test integration testcontainer qui :
1. Applique la migration sur DB de test
2. Vérifie schema attendu
3. Applique rollback (DOWN)
4. Vérifie schema initial restauré

---

## 6. Endpoints API REST

### 6.1 Endpoints nouveaux

| Verbe | Path | Use-case | RBAC | Module |
|---|---|---|---|---|
| POST | `/acps` | `acp_use_cases::create` | admin | identity |
| GET | `/acps` | `acp_use_cases::list_for_user` | tous (filtré par rôle) | identity |
| GET | `/acps/:id` | `acp_use_cases::get` | scope_guard | identity |
| PUT | `/acps/:id` | `acp_use_cases::update` | admin OR syndic_of_acp | identity |
| GET | `/acps/:id/units/:unit_id/etat-date` | `etat_date_use_cases::generate` | notary mandate active | identity+property |
| POST | `/portfolios` | `portfolio_use_cases::create` | syndic OR admin | portfolio |
| GET | `/portfolios` | `portfolio_use_cases::list_for_user` | auth | portfolio |
| POST | `/portfolios/:id/buildings` | `portfolio_use_cases::add_building` | owner_of_portfolio | portfolio |
| POST | `/portfolios/:id/share` | `portfolio_use_cases::share_with` | owner_of_portfolio | portfolio |
| POST | `/magic-links` | `magic_link_use_cases::issue` | syndic | identity |
| GET | `/c/:token` | `magic_link_use_cases::validate_and_get_payload` | public (token-gated) | identity |
| POST | `/mandates` | `mandate_use_cases::issue` | syndic OR ag_decision | identity |
| POST | `/invoices` | `invoice_use_cases::create` | accountant.encodeur OR + | accounting |
| POST | `/expenses` | `expense_use_cases::create` (refacto) | accountant.emetteur OR syndic | accounting |
| POST | `/call-for-funds` | `call_for_funds_use_cases::create` (refacto) | accountant.emetteur OR syndic | accounting |
| POST | `/verification-certificates` | `verification_certificate_use_cases::sign` | commissaire | accounting |
| POST | `/tickets` | `ticket_use_cases::create` (refacto kind+severity) | owner OR cdc OR warden | ticketing |
| POST | `/tickets/:id/responses` | `ticket_use_cases::respond` | syndic | ticketing |
| POST | `/technical-specs` | `technical_spec_use_cases::create` | syndic OR amo | maintenance |
| POST | `/contractor-evaluations` | `contractor_evaluation_use_cases::create` | ag_decision | maintenance |
| POST | `/reservations` (refacto) | `reservation_use_cases::create` + `on_behalf_of_acp: bool` | owner OR (syndic AND on_behalf_of_acp) | community |
| POST | `/acps/:id/modules/:module/enable` | `module_registry_use_cases::enable` | admin OR ag_decision | identity |
| POST | `/acps/:id/modules/:module/disable` | `module_registry_use_cases::disable` | admin OR ag_decision | identity |
| GET | `/acps/:id/modules` | `module_registry_use_cases::list` | auth scope_guard | identity |

### 6.2 Endpoints refactorés (filtrage par rôle)

| Verbe | Path | Modification |
|---|---|---|
| GET | `/buildings` | Filtre par rôle : admin voit tout, syndic voit ACPs de son cabinet, owner voit ses ACPs, contractor voit ses missions (via MagicLink) |
| GET | `/buildings/:id` | scope_guard : refuse 403 si user n'a aucun rôle scope ACP/Building/Portfolio matching |
| POST | `/meetings/:id/complete` | `assert_can_complete()` → 422 typé si pré-conditions manquantes |
| POST | `/votes` | Vérifie `auth_method` selon `Meeting.mode` (hybrid + remote → itsme/eID requis) |

### 6.3 Codes d'erreur typés (extension AppError)

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    // … existing variants …
    #[error("ACP {acp_id:?} not found or out of scope")]
    AcpNotInScope { acp_id: Uuid },

    #[error("Module {module} is not enabled for ACP {acp_id:?}")]
    ModuleDisabled { module: ModuleName, acp_id: Uuid },

    #[error("Building {building_id:?} is not conformant: {reason}")]
    BuildingNotConformant { building_id: Uuid, reason: String },

    #[error("Magic link expired or already consumed")]
    MagicLinkInvalid,

    #[error("Mandate {mandate_id:?} expired at {valid_until}")]
    MandateExpired { mandate_id: Uuid, valid_until: DateTime<Utc> },

    #[error("Meeting cannot be completed: missing {missing:?}")]
    MeetingNotCompletable { missing: Vec<String> },

    #[error("Vote requires strong authentication (itsme/eID), got {got}")]
    VoteAuthInsufficient { got: String },

    #[error("Resolution {resolution_id:?} is auto-generated and cannot be removed")]
    ResolutionAutoNotRemovable { resolution_id: Uuid },

    #[error("TechnicalSpec required before ContractorEvaluation")]
    TechnicalSpecRequired,
}
```

Mapping HTTP : `AcpNotInScope|MagicLinkInvalid|MandateExpired|ModuleDisabled|VoteAuthInsufficient` → 403 ; `BuildingNotConformant|MeetingNotCompletable|TechnicalSpecRequired` → 422 ; `ResolutionAutoNotRemovable` → 403.

---

## 7. Impact frontend (Svelte 5 + Astro)

### 7.1 Composants nouveaux

| Composant | Path | Description |
|---|---|---|
| `BuildingSelector.svelte` | `frontend/src/lib/components/global/BuildingSelector.svelte` | Dropdown + autocomplete + favoris star + portefeuilles équipe (top-left). Conditionné par rôle (visible si user.role ∈ {admin, syndic, accountant.*}). `data-testid="building-selector-input"` |
| `ContextBanner.svelte` | `frontend/src/lib/components/global/ContextBanner.svelte` | Bannière 3-niveaux `Cabinet · ACP · Immeuble` quand building sélectionné. Couleur conformité (vert/orange/rouge). `data-testid="context-banner"` |
| `ModuleGate.svelte` | `frontend/src/lib/components/global/ModuleGate.svelte` | `<ModuleGate module="community"><Slot /></ModuleGate>` masque enfants si module désactivé. Pas de rendu si désactivé (vs disabled visuellement). |
| `ConformityBadge.svelte` | `frontend/src/lib/components/buildings/ConformityBadge.svelte` | Affiche `is_conformant` + delta units/quotas. |
| `MagicLinkContractorPage.svelte` | `frontend/src/pages/c/[token].astro` (page) + composant interne | PWA ultra-simplifiée Contractor : 3 écrans max, install prompt, offline-safe (cache scope_id, response draft). |
| `OnboardingWizard.svelte` | `frontend/src/lib/components/onboarding/OnboardingWizard.svelte` | 5 étapes : profil ACP → recommandation modules → activation → démo. KPI < 5min mesuré client-side. |
| `RoleSubmenu.svelte` | `frontend/src/lib/components/navigation/RoleSubmenu.svelte` | Menus conditionnels par rôle ET sélection ACP/Building/Portfolio. Slot pour items. |

### 7.2 Composants refactorés

| Composant | Modification |
|---|---|
| `Navigation.svelte` | Conditionnement rôle + module + sélection (5 menus : Gestion / Compta / Gouvernance / Communauté / Ticketing). Sous-menus collapsibles. |
| `BuildingDetail.svelte` | Count réel units + somme réelle quotas + `ConformityBadge` + delta. Coordonne avec [#553](https://github.com/gilmry/koprogo/issues/553) Bug 3. |
| `MeetingCreate.svelte` | Champ `mode` (in_person/remote/hybrid). Pré-charge `acp_id` depuis sélecteur global. |
| `VoteCast.svelte` | Auth method conditional : si Meeting.mode ∈ {remote, hybrid} → guide vers itsme/eID. |
| `ReservationCreate.svelte` | Toggle `on_behalf_of_acp` visible uniquement si user.role == syndic. |

### 7.3 Stores Svelte 5 (runes)

```typescript
// frontend/src/stores/scope.svelte.ts
export const scope = $state({
  selectedBuildingId: null as string | null,
  selectedAcpId: null as string | null,
  selectedPortfolioId: null as string | null,
});

// frontend/src/stores/enabled_modules.svelte.ts
export const enabledModules = $state({
  byAcp: new Map<string, Set<ModuleName>>(),
  isEnabled: (acpId: string, m: ModuleName) =>
    enabledModules.byAcp.get(acpId)?.has(m) ?? false,
});
```

### 7.4 Service Worker / Cache (PWA Contractor)

- Cache-first sur assets statiques `/c/*`
- Network-first sur API `/magic-links/*` (avec fallback IndexedDB pour reprise offline)
- Purge SW à chaque release : `sw-version` query param sur fetch, force re-register si mismatch (cf. learning #549)

---

## 8. Stratégie tests architecturale

### 8.1 Pyramide 3-niveaux (cf. [[fe-refactor-test-driven]])

```
                    ┌─────────────────────┐
                    │   E2E Playwright    │  (multi-rôle, 4-cat per FR)
                    │ tests/e2e/refonte-  │
                    │ ux/slice-N/         │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │ E2E Caractérisation │  (IMMUABLE)
                    │ tests/e2e/          │
                    │ characterization/   │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │ Vitest unit + comp. │  (RED-GREEN-BLUE)
                    │ **/__tests__/*.test.│
                    │ ts                  │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │ Rust unit + BDD     │  (4-cat backend)
                    │ cargo test --lib    │
                    │ + --tests           │
                    └─────────────────────┘
```

### 8.2 Couverture par catégorie

Chaque FR du PRD a 4 scénarios minimum (cf. PRD §4 condensé). Détaillé en Phase 4 (stories.md).

### 8.3 Gates CI par slice

- **Slice 0** : caractérisation 100% VERT pré-refonte
- **Slice 1-5** : caractérisation reste VERT + nouveaux 4-cat VERTS + Vitest unit VERTS + axe-core ≥ 90 + lint testid OK

---

## 9. Coordination cross-épics

| Épic / Cluster | Coordination |
|---|---|
| **[#433 Decimal umbrella](https://github.com/gilmry/koprogo/issues/433)** | Tout use-case touché par cette refonte (FR12, FR14, FR22-25) fait sa migration Decimal **dans la même PR**. Convention : 1 PR par use-case = 2 migrations atomiques (refonte + Decimal). |
| **[#555 Result&lt;_, String&gt; epic](https://github.com/gilmry/koprogo/issues/555)** | Idem : migration vers AppError dans la même PR. FR17 (Meeting.complete) déjà identifiée hors #554 → reprise ici. |
| **[#553 Building admin UX](https://github.com/gilmry/koprogo/issues/553)** | Résolu par FR9-FR12 (slice 1). |
| **[#554 World-model seed + AG state](https://github.com/gilmry/koprogo/issues/554)** | FR17, FR18 + slice 4. Seed BDD/E2E unifié via WorldBuilder fluent (mémoire [[world-model-seed]]). |
| **[#550 Playwright stratification](https://github.com/gilmry/koprogo/issues/550)** | FR44 prolonge (zéro helper local, shared `helpers/auth.ts` complet). |
| **[#48 itsme/eID](https://github.com/gilmry/koprogo/issues/48)** | FR15 + ADR-0014 promeuvent in-scope. |
| **[#552 work-reports 400](https://github.com/gilmry/koprogo/issues/552)** | Hors scope direct ; Contract Types Check CI reste VERT. |

---

## 10. Gate de validation Phase 3 — sign-off humain

> ✅ **Architecture SIGNÉE par @gilmry le 2026-05-20** — Phase 3 verrouillée, Phase 4 (Stories Bob) débloquée.

- [x] Diagrammes d'agrégat validés (8 BC DDD)
- [x] ADR-0010 ACP racine d'agrégat — accepté
- [x] ADR-0011 Portefeuille entité backend — accepté
- [x] ADR-0012 Convention data-testid — acceptée
- [x] ADR-0013 Arborescence tests caractérisation — acceptée
- [x] ADR-0014 Signature électronique eIDAS (3 adapters) — accepté
- [x] ADR-0015 Modularité par ACP — acceptée
- [x] Migrations SQL (9 fichiers + rollback) — revues
- [x] Endpoints API (18 nouveaux + 4 refactorés) — validés
- [x] Codes erreur typés (extension AppError) — validés
- [x] Impact frontend (7 nouveaux + 5 refactorés) — validé
- [x] Stratégie tests 3-niveaux — validée
- [x] Coordination cross-épics (#433/#555/#553/#554/#550/#48) — confirmée

**Date signature** : **2026-05-20**
**Signature** : **@gilmry** ✅

---

## 11. Phase suivante

Phase 4 (Stories — Bob) débloquée par sign-off Phase 3. Bob découpera les 5 slices du PRD §8 en **stories individuelles** (1 story = 1 PR cible), avec critères d'acceptation 4-cat exhaustifs (Gherkin complet vs condensé du PRD), data-testid listés, fichiers à toucher, dépendances inter-stories.

Une fois Phase 4 signée → création **issues GitHub** + intégration **WBS go-live v0.1.0 Track H + nouveaux WPs** (cf. décision humaine 2026-05-20).

---

## 12. Liens

- Brief Phase 1 : [`brief.md`](brief.md)
- PRD Phase 2 : [`prd.md`](prd.md)
- Stories Phase 4 : [`stories.md`](stories.md) (à venir post-signature)
- Mémoires d'agent applicables (cf. [`README.md`](README.md))
- ADRs existants : [`docs/adr/`](../../adr/) (0001-0009 + 0044)

🤖 Architecture rédigée par Winston (Architecte hexagonal) — Tier 1 acceptance pending @gilmry sign-off.

# Architecture Technique -- KoproGo

## Methode Maury -- Phase TOGAF C-D (SI + Technique)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : Winston (Architecte)
**Date** : 29/03/2026
**Version** : 2.0
**Brief source** : `Maury/product-brief.md` v1.0 (29/03/2026, Mary -- Analyste)
**PRD source** : `Maury/PRD.md` v1.0 (29/03/2026, John -- Product Manager)

**Stack** : Rust 1.82+ / Actix-web 4.9 (backend), Astro 4.x / Svelte 4.x (frontend), PostgreSQL 15
**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Organisation** : Scrum -> Nexus -> SAFe -> ITIL
**Dev** : Agents IA supervises par Gilles Maury

**Metriques actuelles** : 559 endpoints API, 60 entites domaine, 82 migrations PostgreSQL, 137k+ LOC Rust, 819 scenarios BDD, 74 features Gherkin, 178 composants Svelte, 60 repository ports

**Tracabilite** : Ce document d'architecture consomme et transforme le product brief (Mary, Phase TOGAF A) et le PRD (John, Phase TOGAF B-C) en decisions architecturales concretes mappees sur le code reel. Chaque composant trace vers un FR du PRD, un invariant du brief, ou une NFR.

---

## 1. Vue d'ensemble

```
                            ┌─────────────────────────────────────────────┐
                            │              UTILISATEURS                   │
                            │  Marc (Syndic) · Sophie (Coproprietaire)    │
                            │  Jean-Pierre (Comptable) · Ahmed (Prestataire) │
                            └──────────────────┬──────────────────────────┘
                                               │ HTTPS / WSS
                            ┌──────────────────▼──────────────────────────┐
                            │          FRONTEND (Astro + Svelte)          │
                            │  178 composants · 22 API clients · 4 langues │
                            │  Islands Architecture · Tailwind CSS 3.x    │
                            │  SSG (Static Site Generation) + hydratation │
                            │  Port: 3000 (dev) / 443 (prod via Traefik)  │
                            └──────────────────┬──────────────────────────┘
                                               │ REST JSON / multipart
                  ┌────────────────────────────▼────────────────────────────┐
                  │                    TRAEFIK (Reverse Proxy)              │
                  │  TLS termination · Rate limiting · HSTS · CSP headers   │
                  └────────────────────────────┬────────────────────────────┘
                                               │
┌──────────────────────────────────────────────▼────────────────────────────────────┐
│                         BACKEND (Rust + Actix-web 4.9)                            │
│  Port: 8080 · 559 endpoints REST · /api/v1/*                                     │
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐  │
│  │                    INFRASTRUCTURE (Adapters externes)                       │  │
│  │  web/handlers/ (60 fichiers) · web/routes.rs · database/repositories/ (60) │  │
│  │  Principes SOLID : LSP (contrats respectes), DIP (depend des ports)        │  │
│  └─────────────────────────────────┬───────────────────────────────────────────┘  │
│                                    │ implements                                    │
│  ┌─────────────────────────────────▼───────────────────────────────────────────┐  │
│  │                    APPLICATION (Use Cases + Ports)                          │  │
│  │  use_cases/ (50 fichiers) · ports/ (60 traits) · dto/ (50+ DTOs)           │  │
│  │  Principes SOLID : ISP (traits granulaires), DIP, OCP (extensible)         │  │
│  └─────────────────────────────────┬───────────────────────────────────────────┘  │
│                                    │ depends on                                    │
│  ┌─────────────────────────────────▼───────────────────────────────────────────┐  │
│  │                    DOMAIN (Logique metier pure)                             │  │
│  │  entities/ (60 fichiers) · services/ · ZERO dependance externe             │  │
│  │  Invariants dans constructeurs · Langage ubiquitaire belge                 │  │
│  │  Principes SOLID : SRP (1 entite = 1 responsabilite), DIP (definit ports) │  │
│  └─────────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────┬────────────────────────────────────┘
                                               │ sqlx (compile-time verified)
                            ┌──────────────────▼──────────────────────────┐
                            │          POSTGRESQL 15                       │
                            │  82 migrations · 59 tables · UUIDs PK       │
                            │  Triggers (validate_unit_ownership_total)    │
                            │  ENUMs custom · Indexes partiels             │
                            │  Connection pool: max 10                     │
                            │  Port: 5432                                  │
                            └──────────────────┬──────────────────────────┘
                                               │
                            ┌──────────────────▼──────────────────────────┐
                            │          OBSERVABILITE                       │
                            │  Prometheus :9090 · Grafana :3001            │
                            │  Loki (logs) · Alertmanager :9093            │
                            │  Backend /metrics endpoint                   │
                            └─────────────────────────────────────────────┘

                            ┌─────────────────────────────────────────────┐
                            │          SECURITE (Defense en profondeur)    │
                            │  LUKS (AES-XTS-512) · Suricata IDS          │
                            │  CrowdSec WAF · fail2ban · 2FA TOTP         │
                            │  Lynis >80/100 · rkhunter · AIDE            │
                            └─────────────────────────────────────────────┘
```

**Tracabilite** : Ce diagramme materialise les principes d'architecture du brief (section 15) et les NFR du PRD (section 7). La separation en 3 couches internes (Domain / Application / Infrastructure) est la pierre angulaire de l'architecture hexagonale imposee par la Methode Maury.

---

## 2. Bounded Contexts -> Modules Rust

> Mapping complet des 13 bounded contexts identifies dans le brief (section 9) et le PRD (section 5) vers les chemins reels du code Rust.

### 2.1 Table de correspondance complete

| # | Bounded Context (brief s9, PRD s5) | Entites Domain (`backend/src/domain/entities/`) | Ports (`backend/src/application/ports/`) | Use Cases (`backend/src/application/use_cases/`) | Handlers (`backend/src/infrastructure/web/handlers/`) | Repository Impl (`backend/src/infrastructure/database/repositories/`) | FRs PRD |
|---|-----|------|------|------|------|------|------|
| 1 | **Building Management** | `building.rs`, `unit.rs`, `unit_owner.rs` | `building_repository.rs`, `unit_repository.rs`, `unit_owner_repository.rs` | `building_use_cases.rs`, `unit_use_cases.rs`, `unit_owner_use_cases.rs` | `building_handlers.rs`, `unit_handlers.rs`, `unit_owner_handlers.rs`, `public_handlers.rs` | `building_repository_impl.rs`, `unit_repository_impl.rs`, `unit_owner_repository_impl.rs` | FR-001, FR-002 |
| 2 | **Identity & Access** | `user.rs`, `user_role_assignment.rs`, `two_factor_secret.rs`, `organization.rs`, `refresh_token.rs` | `user_repository.rs`, `user_role_repository.rs`, `two_factor_repository.rs`, `organization_repository.rs`, `refresh_token_repository.rs` | `auth_use_cases.rs`, `two_factor_use_cases.rs` | `auth_handlers.rs`, `two_factor_handlers.rs`, `user_handlers.rs`, `organization_handlers.rs` | `user_repository_impl.rs`, `user_role_repository_impl.rs`, `two_factor_repository_impl.rs`, `organization_repository_impl.rs`, `refresh_token_repository_impl.rs` | FR-018 |
| 3 | **General Assembly** | `meeting.rs`, `resolution.rs`, `vote.rs`, `convocation.rs`, `convocation_recipient.rs`, `ag_session.rs`, `age_request.rs` | `meeting_repository.rs`, `resolution_repository.rs`, `vote_repository.rs`, `convocation_repository.rs`, `convocation_recipient_repository.rs`, `ag_session_repository.rs`, `age_request_repository.rs` | `meeting_use_cases.rs`, `resolution_use_cases.rs`, `convocation_use_cases.rs`, `ag_session_use_cases.rs`, `age_request_use_cases.rs` | `meeting_handlers.rs`, `resolution_handlers.rs`, `convocation_handlers.rs`, `ag_session_handlers.rs`, `age_request_handlers.rs` | idem `_impl.rs` | FR-003, FR-004, FR-005 |
| 4 | **Accounting** | `account.rs`, `journal_entry.rs`, `budget.rs`, `etat_date.rs` | `account_repository.rs`, `journal_entry_repository.rs`, `budget_repository.rs`, `etat_date_repository.rs` | `account_use_cases.rs`, `journal_entry_use_cases.rs`, `budget_use_cases.rs`, `etat_date_use_cases.rs`, `financial_report_use_cases.rs`, `dashboard_use_cases.rs` | `account_handlers.rs`, `budget_handlers.rs`, `etat_date_handlers.rs`, `financial_report_handlers.rs`, `dashboard_handlers.rs` | idem `_impl.rs` | FR-006, FR-007, FR-008 |
| 5 | **Billing & Payments** | `expense.rs`, `invoice_line_item.rs`, `payment.rs`, `payment_method.rs`, `payment_reminder.rs`, `owner_contribution.rs`, `call_for_funds.rs`, `charge_distribution.rs` | `expense_repository.rs`, `payment_repository.rs`, `payment_method_repository.rs`, `payment_reminder_repository.rs`, `owner_contribution_repository.rs`, `call_for_funds_repository.rs`, `charge_distribution_repository.rs` | `expense_use_cases.rs`, `payment_use_cases.rs`, `payment_method_use_cases.rs`, `payment_reminder_use_cases.rs`, `owner_contribution_use_cases.rs`, `call_for_funds_use_cases.rs`, `charge_distribution_use_cases.rs` | `expense_handlers.rs`, `payment_handlers.rs`, `payment_method_handlers.rs`, `payment_reminder_handlers.rs`, `owner_contribution_handlers.rs`, `call_for_funds_handlers.rs`, `charge_distribution_handlers.rs` | idem `_impl.rs` | FR-009, FR-010, FR-011, FR-012 |
| 6 | **Maintenance** | `ticket.rs`, `quote.rs`, `work_report.rs`, `technical_inspection.rs`, `contractor_report.rs` | `ticket_repository.rs`, `quote_repository.rs`, `work_report_repository.rs`, `technical_inspection_repository.rs`, `contractor_report_repository.rs` | `ticket_use_cases.rs`, `quote_use_cases.rs`, `work_report_use_cases.rs`, `technical_inspection_use_cases.rs`, `contractor_report_use_cases.rs` | `ticket_handlers.rs`, `quote_handlers.rs`, `work_report_handlers.rs`, `technical_inspection_handlers.rs`, `contractor_report_handlers.rs` | idem `_impl.rs` | FR-013, FR-014 |
| 7 | **Notifications** | `notification.rs` | `notification_repository.rs`, `notification_preference_repository.rs` | `notification_use_cases.rs` | `notification_handlers.rs` | `notification_repository_impl.rs`, `notification_preference_repository_impl.rs` | FR-019 |
| 8 | **GDPR & Compliance** | `gdpr_export.rs`, `gdpr_rectification.rs`, `gdpr_restriction.rs`, `gdpr_objection.rs`, `consent.rs` | `gdpr_repository.rs`, `consent_repository.rs` | `gdpr_use_cases.rs`, `consent_use_cases.rs` | `gdpr_handlers.rs`, `gdpr_art30_handlers.rs`, `admin_gdpr_handlers.rs`, `consent_handlers.rs` | `gdpr_repository_impl.rs`, `consent_repository_impl.rs` | FR-015 |
| 9 | **Community** | `local_exchange.rs`, `owner_credit_balance.rs`, `poll.rs`, `poll_vote.rs`, `notice.rs`, `skill.rs`, `shared_object.rs`, `resource_booking.rs` | `local_exchange_repository.rs`, `owner_credit_balance_repository.rs`, `poll_repository.rs`, `poll_vote_repository.rs`, `notice_repository.rs`, `skill_repository.rs`, `shared_object_repository.rs`, `resource_booking_repository.rs` | `local_exchange_use_cases.rs`, `poll_use_cases.rs`, `notice_use_cases.rs`, `skill_use_cases.rs`, `shared_object_use_cases.rs`, `resource_booking_use_cases.rs` | `poll_handlers.rs`, `notice_handlers.rs`, `skill_handlers.rs`, `shared_object_handlers.rs`, `resource_booking_handlers.rs` | idem `_impl.rs` | FR-016, FR-017 |
| 10 | **Gamification** | `achievement.rs`, `challenge.rs` (+ `user_achievement` et `challenge_progress` integres) | `achievement_repository.rs`, `challenge_repository.rs` | `gamification_use_cases.rs` | `gamification_handlers.rs` | `achievement_repository_impl.rs`, `challenge_repository_impl.rs` | — (SHOULD, Jalon 3) |
| 11 | **Documents** | `document.rs` | `document_repository.rs` | `document_use_cases.rs` | `document_handlers.rs` | `document_repository_impl.rs` | — |
| 12 | **Energy & IoT** | `energy_campaign.rs`, `energy_bill_upload.rs`, `iot_reading.rs`, `linky_device.rs` | `energy_campaign_repository.rs`, `energy_bill_upload_repository.rs`, `iot_repository.rs` | `energy_campaign_use_cases.rs`, `energy_bill_upload_use_cases.rs`, `iot_use_cases.rs` | `energy_campaign_handlers.rs`, `energy_bill_upload_handlers.rs`, `iot_handlers.rs`, `iot_grid_handlers.rs` | idem `_impl.rs` | — (COULD, Jalon 4) |
| 13 | **Board Management** | `board_member.rs`, `board_decision.rs` | `board_member_repository.rs`, `board_decision_repository.rs` | `board_member_use_cases.rs`, `board_decision_use_cases.rs`, `board_dashboard_use_cases.rs` | `board_member_handlers.rs`, `board_decision_handlers.rs` | `board_member_repository_impl.rs`, `board_decision_repository_impl.rs` | — |

### 2.2 Dependances inter-contextes

```
Building Management ──────────┐
       │                      │
       ├──▶ General Assembly  ├──▶ Accounting
       │         │            │         │
       │         ▼            │         ▼
       ├──▶ Billing & Payments ◀────────┘
       │         │
       │         ▼
       ├──▶ Maintenance ──────▶ Notifications
       │
       ├──▶ Community ────────▶ Gamification
       │
       ├──▶ Board Management
       │
       └──▶ Energy & IoT

Identity & Access ──▶ TOUS les contextes (transversal)
GDPR & Compliance ──▶ Identity & Access (operations sur User)
Documents ──────────▶ TOUS les contextes (liaison universelle)
```

**Tracabilite** : Cette carte de dependances materialise le diagramme ASCII du brief (section 9) et les dependances entre FRs du PRD (section 6).

---

## 3. Architecture hexagonale SOLID

### 3.1 Couche Domain (SOLID : SRP, DIP)

**Emplacement** : `backend/src/domain/`

```
backend/src/domain/
├── entities/           # 60 agregats (SRP) avec invariants
│   ├── building.rs     # INV-9 : name non vide (brief s10)
│   ├── unit.rs         # Lots dans immeubles
│   ├── unit_owner.rs   # INV-1, INV-6 : quotes-parts (brief s10)
│   ├── resolution.rs   # INV-4 : majorites legales (brief s10)
│   ├── vote.rs         # Suffrage individuel avec tantiemes
│   ├── convocation.rs  # INV-3 : delai 15 jours (brief s10)
│   ├── payment.rs      # INV-7 : idempotency (brief s10)
│   ├── journal_entry.rs # INV-5 : double-entree equilibree (brief s10)
│   ├── age_request.rs  # INV-8 : seuil AGE 1/5 (brief s10)
│   ├── quote.rs        # INV-10 : 3 devis >5000 EUR (brief s10)
│   ├── account.rs      # PCMN belge AR 12/07/2012
│   ├── expense.rs      # Workflow approbation factures
│   ├── meeting.rs      # INV-2 : quorum 50% (brief s10)
│   ├── ... (47 autres entites)
│   └── mod.rs
├── services/           # Logique inter-entites (domain services)
└── mod.rs
```

**Principes SOLID appliques** :

- **SRP (Single Responsibility)** : Chaque entite a une responsabilite unique. `Building` gere les donnees immobilieres, pas la comptabilite. `Resolution` gere la proposition de vote, `Vote` gere le suffrage individuel (PRD FR-004, SOLID explique).
- **DIP (Dependency Inversion)** : Le Domain definit les interfaces (ports) que l'Infrastructure implemente. Le Domain n'a AUCUNE dependance externe (ni actix-web, ni sqlx, ni tokio). Seules les crates `uuid`, `chrono`, `serde` sont autorisees.

**Invariants dans les constructeurs** (brief section 10) :

Chaque invariant metier est code dans le constructeur `::new() -> Result<Self, String>` de l'entite concernee. Une violation = erreur immediate, jamais silencieuse.

#### INV-9 : Building.name non vide (brief s10, PRD FR-001)

```rust
// backend/src/domain/entities/building.rs
impl Building {
    pub fn new(
        organization_id: Uuid,
        name: String,
        address: String,
        city: String,
        postal_code: String,
        country: String,
        total_units: i32,
        total_tantiemes: i32,
        construction_year: Option<i32>,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Building name cannot be empty".to_string());
        }
        if total_units <= 0 {
            return Err("Total units must be greater than 0".to_string());
        }
        if total_tantiemes <= 0 {
            return Err("Total tantiemes must be greater than 0".to_string());
        }
        // ... construction avec slug SEO auto-genere
    }
}
```

#### INV-6 : Quote-part 0 < p <= 1.0 (brief s10, PRD FR-002)

```rust
// backend/src/domain/entities/unit_owner.rs
impl UnitOwner {
    pub fn new(
        unit_id: Uuid,
        owner_id: Uuid,
        ownership_percentage: f64,
        is_primary_contact: bool,
    ) -> Result<Self, String> {
        if ownership_percentage <= 0.0 || ownership_percentage > 1.0 {
            return Err("Ownership percentage must be between 0 and 1".to_string());
        }
        // ...
    }
}
```

**INV-1 : Somme des quotes-parts actives = 100%** (brief s10, PRD FR-002) : Applique au niveau PostgreSQL via le trigger `validate_unit_ownership_total` (migration `20251120230000_add_unit_ownership_validation.sql`) avec tolerance +/-0.01% pour arrondis. Ce choix architectural place la validation au niveau base de donnees car elle implique une requete d'agregation sur tous les proprieatires actifs d'un lot.

#### INV-4 : Majorites legales belges (brief s10, PRD FR-004)

```rust
// backend/src/domain/entities/resolution.rs
pub enum MajorityType {
    /// >50% des presents/representes, abstentions EXCLUES -- Art. 3.88 ss1
    Absolute,
    /// >=2/3 des presents/representes -- Art. 3.88 ss1, 1deg
    TwoThirds,
    /// >=4/5 des presents/representes -- Art. 3.88 ss1, 2deg
    FourFifths,
    /// 100% de TOUS les tantiemes (y compris absents) -- Art. 3.88 ss1, 3deg
    Unanimity,
}
```

#### INV-3 : Delai legal convocation >= 15 jours (brief s10, PRD FR-003)

La `Convocation` calcule automatiquement `minimum_send_date = meeting_date - 15 jours`. Le workflow Draft -> Scheduled -> Sent valide que la date d'envoi respecte le delai legal. Violation = erreur `"La convocation ne respecte pas le delai legal de 15 jours"`.

#### INV-5 : Comptabilite double-entree equilibree (brief s10, PRD FR-006)

Le `JournalEntry` valide que la somme des debits = somme des credits sur toutes les `JournalEntryLine` associees. Violation = erreur `"Ecriture desequilibree : debits != credits"`.

#### INV-7 : Idempotency des paiements (brief s10, PRD FR-010)

Le `Payment` valide que l'`idempotency_key` fait au minimum 16 caracteres et est unique. La contrainte UNIQUE sur la colonne PostgreSQL previent les doubles charges.

#### INV-8 : Seuil AGE 1/5 des quotes-parts (brief s10, PRD FR-005)

L'`AgeRequest` verifie a chaque cosignature si `total_shares_pct >= threshold_pct` (0.20). Quand le seuil est atteint, le statut passe automatiquement de `Open` a `Reached`.

#### INV-10 : 3 devis minimum pour travaux > 5000 EUR (brief s10, PRD FR-014)

Le `Quote` compare les devis avec scoring automatique (prix 40%, delai 30%, garantie 20%, reputation 10%). Un avertissement est emis si moins de 3 devis sont presents pour des travaux > 5000 EUR.

#### INV-2 : Quorum AG >= 50% (brief s10, PRD FR-004)

Le `Meeting` verifie que `present_quotas / total_quotas >= quorum_percentage` (50% par defaut, Art. 3.87 ss5 CC) avant d'autoriser l'ouverture des votes.

### 3.2 Couche Application (SOLID : ISP, DIP, OCP)

**Emplacement** : `backend/src/application/`

```
backend/src/application/
├── dto/                    # Data Transfer Objects (50+ fichiers)
│   ├── building_dto.rs     # CreateBuildingDto, BuildingResponseDto, BuildingFilters, PageRequest
│   ├── resolution_dto.rs   # CreateResolutionDto, ResolutionResponseDto
│   ├── vote_dto.rs         # CastVoteDto, VoteResponseDto
│   ├── payment_dto.rs      # CreatePaymentDto, PaymentResponseDto
│   ├── gdpr_dto.rs         # GdprRectifyRequest, GdprActionResponse
│   ├── convocation_dto.rs  # CreateConvocationDto, ConvocationResponseDto
│   ├── poll_dto.rs         # CreatePollDto, PollResultsDto
│   ├── gamification_dto.rs # CreateAchievementDto, LeaderboardResponseDto
│   ├── local_exchange_dto.rs # CreateLocalExchangeDto, SelStatisticsDto
│   ├── public_dto.rs       # PublicSyndicInfoResponse (sans auth)
│   └── ...
├── ports/                  # Traits (interfaces) -- 60 fichiers
│   ├── building_repository.rs      # BuildingRepository trait (7 methodes)
│   ├── unit_repository.rs          # UnitRepository trait
│   ├── unit_owner_repository.rs    # UnitOwnerRepository trait
│   ├── meeting_repository.rs       # MeetingRepository trait
│   ├── resolution_repository.rs    # ResolutionRepository trait
│   ├── vote_repository.rs          # VoteRepository trait
│   ├── convocation_repository.rs   # ConvocationRepository trait (13 methodes)
│   ├── convocation_recipient_repository.rs # ConvocationRecipientRepository (18 methodes)
│   ├── payment_repository.rs       # PaymentRepository trait (21 methodes)
│   ├── payment_method_repository.rs # PaymentMethodRepository trait (13 methodes)
│   ├── gdpr_repository.rs          # GdprRepository trait
│   ├── poll_repository.rs          # PollRepository trait (16 methodes)
│   ├── poll_vote_repository.rs     # PollVoteRepository trait (10 methodes)
│   ├── local_exchange_repository.rs # LocalExchangeRepository trait (18 methodes)
│   ├── ticket_repository.rs        # TicketRepository trait (18 methodes)
│   ├── quote_repository.rs         # QuoteRepository trait (15 methodes)
│   ├── achievement_repository.rs   # AchievementRepository trait (8 methodes)
│   ├── challenge_repository.rs     # ChallengeRepository trait (10 methodes)
│   ├── audit_log_repository.rs     # AuditLogRepository trait (GDPR Art. 30)
│   └── ... (41 autres ports)
└── use_cases/              # Orchestration -- 50 fichiers
    ├── building_use_cases.rs       # BuildingUseCases (CRUD + find_by_slug)
    ├── unit_use_cases.rs           # UnitUseCases
    ├── unit_owner_use_cases.rs     # UnitOwnerUseCases (somme quotes-parts, transfert)
    ├── auth_use_cases.rs           # AuthUseCases (login, switch_role, refresh_token)
    ├── meeting_use_cases.rs        # MeetingUseCases
    ├── resolution_use_cases.rs     # ResolutionUseCases (14 methodes, calcul majorites)
    ├── convocation_use_cases.rs    # ConvocationUseCases (21 methodes, multi-repo)
    ├── account_use_cases.rs        # AccountUseCases (PCMN seed, CRUD)
    ├── journal_entry_use_cases.rs  # JournalEntryUseCases (double-entree)
    ├── budget_use_cases.rs         # BudgetUseCases (workflow approbation)
    ├── etat_date_use_cases.rs      # EtatDateUseCases (delai 10j, expiration 3m)
    ├── expense_use_cases.rs        # ExpenseUseCases (workflow Draft->Approved)
    ├── payment_use_cases.rs        # PaymentUseCases (26 methodes, Stripe lifecycle)
    ├── payment_method_use_cases.rs # PaymentMethodUseCases (14 methodes)
    ├── payment_reminder_use_cases.rs # PaymentReminderUseCases (escalade 4 niveaux)
    ├── gdpr_use_cases.rs           # GdprUseCases (Articles 15-21, audit trail)
    ├── poll_use_cases.rs           # PollUseCases (18 methodes, 4 types sondage)
    ├── local_exchange_use_cases.rs # LocalExchangeUseCases (20 methodes, SEL)
    ├── ticket_use_cases.rs         # TicketUseCases (18 methodes, SLA)
    ├── quote_use_cases.rs          # QuoteUseCases (20 methodes, scoring belge)
    ├── gamification_use_cases.rs   # GamificationUseCases (stats, leaderboard)
    ├── financial_report_use_cases.rs # FinancialReportUseCases (bilan, resultats)
    └── ... (28 autres use cases)
```

**Principes SOLID appliques** :

- **ISP (Interface Segregation)** : Chaque bounded context a ses propres traits de repository. Pas de "god interface". Exemples : `BuildingRepository` (7 methodes), `UnitRepository` (methodes distinctes), `UnitOwnerRepository` (methodes separees). Un handler de buildings n'a pas besoin de connaitre `PaymentRepository`.

- **DIP (Dependency Inversion)** : Les use cases dependent uniquement des traits (ports), jamais des implementations concretes. Exemple : `ResolutionUseCases` prend `Box<dyn ResolutionRepository>` et `Box<dyn VoteRepository>`, pas `PostgresResolutionRepository`.

- **OCP (Open/Closed)** : Ajout de nouveaux adaptateurs sans modifier le Domain. Exemple : remplacer PostgreSQL par ScyllaDB = nouvelle implementation du trait `BuildingRepository`, zero changement dans `BuildingUseCases` ou `Building`.

**Exemple concret de port (PRD FR-001)** :

```rust
// backend/src/application/ports/building_repository.rs
#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
    async fn find_all(&self) -> Result<Vec<Building>, String>;
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String>;
    async fn update(&self, building: &Building) -> Result<Building, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
}
```

### 3.3 Couche Infrastructure (SOLID : LSP, DIP)

**Emplacement** : `backend/src/infrastructure/`

```
backend/src/infrastructure/
├── database/
│   └── repositories/       # 60 implementations PostgreSQL
│       ├── building_repository_impl.rs       # PostgresBuildingRepository
│       ├── unit_repository_impl.rs           # PostgresUnitRepository
│       ├── unit_owner_repository_impl.rs     # PostgresUnitOwnerRepository
│       ├── resolution_repository_impl.rs     # PostgresResolutionRepository
│       ├── vote_repository_impl.rs           # PostgresVoteRepository
│       ├── convocation_repository_impl.rs    # PostgresConvocationRepository (600 lignes)
│       ├── payment_repository_impl.rs        # PostgresPaymentRepository (21 methodes)
│       ├── poll_repository_impl.rs           # PostgresPollRepository (511 lignes)
│       ├── local_exchange_repository_impl.rs # PostgresLocalExchangeRepository (466 lignes)
│       ├── ... (51 autres implementations)
│       └── mod.rs
├── web/
│   ├── handlers/           # 60 fichiers de handlers Actix-web
│   │   ├── building_handlers.rs          # CRUD buildings
│   │   ├── public_handlers.rs            # GET /public/buildings/:slug/syndic (sans auth)
│   │   ├── auth_handlers.rs              # Login, switch-role, 2FA
│   │   ├── resolution_handlers.rs        # 9 endpoints vote AG
│   │   ├── convocation_handlers.rs       # 14 endpoints convocations
│   │   ├── payment_handlers.rs           # 22 endpoints paiements
│   │   ├── payment_method_handlers.rs    # 16 endpoints methodes de paiement
│   │   ├── gdpr_handlers.rs              # Articles 15-21
│   │   ├── poll_handlers.rs              # 12 endpoints sondages
│   │   ├── gamification_handlers.rs      # 22 endpoints achievements/challenges
│   │   ├── ticket_handlers.rs            # 17 endpoints maintenance
│   │   ├── quote_handlers.rs             # 15 endpoints devis
│   │   ├── contractor_report_handlers.rs # Magic link PWA
│   │   ├── health.rs                     # Health check
│   │   ├── metrics.rs                    # Prometheus /metrics
│   │   └── ... (44 autres handlers)
│   └── routes.rs           # Configuration de toutes les routes API
└── services/               # Implementations de services externes
```

**Principes SOLID appliques** :

- **LSP (Liskov Substitution)** : Toute implementation de `BuildingRepository` est substituable. `PostgresBuildingRepository` respecte exactement le contrat du trait. Les tests d'integration valident ce contrat avec testcontainers.

- **DIP (Dependency Inversion)** : Les handlers dependent des use cases (qui dependent des ports), pas directement des repositories. Le couplage est unidirectionnel : Infrastructure -> Application -> Domain.

**Exemple d'adapter concret (PRD FR-001)** :

```rust
// backend/src/infrastructure/database/repositories/building_repository_impl.rs
pub struct PostgresBuildingRepository {
    pool: PgPool,
}

#[async_trait]
impl BuildingRepository for PostgresBuildingRepository {
    async fn create(&self, building: &Building) -> Result<Building, String> {
        sqlx::query_as!(Building, "INSERT INTO buildings (...) VALUES (...)")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }
    // ... 6 autres methodes
}
```

---

## 4. Strategie de tests (TDD + BDD)

> Derivee du brief section 15 (principes TDD/BDD) et PRD section 7.5 (testabilite, NFR-005).

### 4.1 Pyramide de tests

```
                    ┌────────────┐
                    │ Benchmarks │  Criterion (performance critique)
                    │ (backend/  │
                    │  benches/) │
                    ├────────────┤
                    │    E2E     │  49 smoke tests + 12 Documentation Vivante
                    │ (frontend/ │  Playwright (frontend/tests/e2e/)
                    │  tests/e2e │  + cargo test --test e2e (backend)
                    │  + backend │
                    │  tests/e2e)│
                ┌───┴────────────┴───┐
                │        BDD         │  819 scenarios, 74 features
                │  backend/tests/    │  Cucumber/Gherkin
                │  features/*.feature│  cargo test --test bdd
                ├────────────────────┤
                │    Integration     │  testcontainers PostgreSQL
                │  backend/tests/    │  cargo test --test integration
                │  integration/      │
            ┌───┴────────────────────┴───┐
            │       Unit (Domain)        │  100% couverture cible
            │  #[cfg(test)] in-module    │  cargo test --lib
            │  60 fichiers entites       │
            │  + mockall pour use cases  │
            └────────────────────────────┘
```

### 4.2 Couverture cible par couche

| Couche | Type de test | Couverture cible | Outil | Commande |
|--------|-------------|-----------------|-------|----------|
| Domain (`entities/`) | Unitaires in-module | **100%** | `#[cfg(test)]`, assertions | `cargo test --lib` |
| Application (`use_cases/`) | Unitaires + mocks | > 90% | `mockall` | `cargo test --lib` |
| Infrastructure (`repositories/`) | Integration | > 80% | `testcontainers` PostgreSQL | `cargo test --test integration` |
| API (endpoints) | E2E | Flux critiques | `reqwest` | `cargo test --test e2e` |
| Metier (scenarios) | BDD | **Tous les scenarios PRD** | `cucumber` 0.21 | `cargo test --test bdd` |
| Frontend (parcours) | E2E | 49 smoke + 12 Doc Vivante | Playwright | `npm run test:e2e` |
| Performance | Benchmarks | Latence P99 < 5ms | Criterion 0.5 | `cargo bench` |
| Securite | Audit | Score Lynis > 80 | Lynis, rkhunter, AIDE | Automatise hebdo/quotidien |
| Couverture globale | Report | Dashboard | Tarpaulin | `make coverage` |

### 4.3 BDD : Scenarios Gherkin par bounded context

Les 74 fichiers `.feature` dans `backend/tests/features/` couvrent les 819 scenarios metier. Correspondance avec les FRs du PRD :

| Bounded Context | Features Gherkin | FR PRD |
|----------------|-----------------|---------|
| Building Management | `building.feature`, `unit_owner_validation.feature` | FR-001, FR-002 |
| Identity & Access | `auth.feature`, `two_factor.feature`, `multitenancy.feature`, `organizations.feature`, `api_keys.feature` | FR-018 |
| General Assembly | `meetings.feature`, `meetings_manage.feature`, `resolutions.feature`, `vote_ag_workflow.feature`, `resolution_agenda.feature`, `convocations.feature`, `second_convocation.feature`, `ag_sessions.feature`, `age_requests.feature` | FR-003, FR-004, FR-005 |
| Accounting | `accounts.feature`, `journal_entries.feature`, `budget.feature`, `etat_date.feature`, `dashboard.feature` | FR-006, FR-007, FR-008 |
| Billing & Payments | `expenses.feature`, `expenses_pagination.feature`, `expenses_pcn.feature`, `invoices.feature`, `payments.feature`, `payment_methods.feature`, `payment_recovery.feature`, `call_for_funds.feature`, `charge_distribution.feature`, `owner_contributions.feature` | FR-009, FR-010, FR-011, FR-012 |
| Maintenance | `tickets.feature`, `ticket_workflow.feature`, `quotes.feature`, `work_reports.feature`, `work_orders.feature`, `technical_inspections.feature`, `contractor_reports.feature`, `contract_evaluation.feature`, `service_providers.feature` | FR-013, FR-014 |
| GDPR & Compliance | `gdpr.feature`, `gdpr_art30.feature`, `consent.feature`, `legal_compliance.feature`, `legal_api.feature`, `security_incidents.feature` | FR-015 |
| Community | `local_exchange.feature`, `sel_workflow.feature`, `polls.feature`, `poll_workflow.feature`, `notices.feature`, `notice_board_workflow.feature`, `skills.feature`, `shared_objects.feature`, `resource_bookings.feature`, `marketplace.feature` | FR-016, FR-017 |
| Gamification | `gamification.feature` | — |
| Documents | `documents.feature`, `documents_delete.feature`, `documents_expenses.feature`, `documents_linking.feature` | — |
| Energy & IoT | `energy_campaigns.feature`, `iot.feature`, `iot_mqtt_boinc.feature` | — |
| Board Management | `board.feature`, `board_members.feature`, `board_decisions.feature`, `board_dashboard.feature` | — |
| Transversal | `i18n.feature`, `pagination_filtering.feature`, `public_syndic.feature`, `individual_members.feature`, `mcp_sse.feature` | — |

### 4.4 Test-Driven Emergence (brief section 15)

Le meme scenario metier est la source de verite unique exprimee a 3 niveaux :

```
Scenario metier (narratif multi-roles avec semantique copropriete belge)
  ↓ Gherkin
BDD integration (backend/tests/features/) -- valide le contrat comportemental
  ↓ meme narratif
E2E Documentation Vivante (frontend/tests/e2e/scenarios/) -- prouve le parcours UI
  ↓ video generee
Preuve visuelle (YouTube/stakeholders) -- couronne la spec
```

**Alignement BDD <-> E2E** : Si le BDD passe mais le E2E echoue -> bug frontend. Si les deux echouent -> probleme de spec/backend. La matrice RACE (Reach, Act, Convert, Engage) decouple la plastique UI de la fonctionnalite testee.

---

## 5. Modele de donnees (DDD -> PostgreSQL)

> 60 entites domaine -> 82 migrations PostgreSQL -> 59+ tables. Toutes les tables utilisent UUID pour les PK et incluent `created_at`/`updated_at` timestamps. Derivee du PRD section 8.

### 5.1 Schema relationnel par bounded context

```
┌─────────────────────────────────────────────────────────────────┐
│                    BUILDING MANAGEMENT                           │
│                                                                  │
│  buildings ◄──────────── units ◄──────────── unit_owners         │
│  (id, org_id, name,      (id, building_id,   (id, unit_id,      │
│   slug, syndic_*,         unit_number,         owner_id,          │
│   total_tantiemes)        floor, area)         percentage,        │
│                                                 start/end_date)   │
│                                     ┌────────── owners            │
│                                     │           (id, org_id,      │
│                                     │            name, email)     │
└─────────────────────────────────────┤────────────────────────────┘
                                      │
┌─────────────────────────────────────┤────────────────────────────┐
│                    GENERAL ASSEMBLY  │                            │
│                                     │                            │
│  meetings ◄─── resolutions ◄─── votes                            │
│  (quorum_pct,   (majority_type,    (choice, voting_power,        │
│   total_quotas)  status)            proxy_owner_id)               │
│      │                                                            │
│      ├─── convocations ◄─── convocation_recipients               │
│      │    (meeting_type,      (email_opened_at,                   │
│      │     min_send_date)      attendance_status)                  │
│      │                                                            │
│      ├─── ag_sessions                                             │
│      │    (platform, video_url, remote_attendees)                 │
│      │                                                            │
│      └─── age_requests ◄─── age_request_cosignatories            │
│           (threshold_pct=0.20,  (owner_id, shares_pct)            │
│            total_shares_pct)                                      │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                    ACCOUNTING                                     │
│                                                                   │
│  accounts (PCMN ~90 comptes, 8 classes)                           │
│      │                                                            │
│      └─── journal_entries ◄─── journal_entry_lines               │
│           (journal_type:         (account_code,                   │
│            ACH/VEN/FIN/ODS)       debit, credit)                  │
│                                                                   │
│  budgets (fiscal_year, status, total_budget, variance)            │
│  etats_dates (reference_number, financial_data, pdf_file_path)    │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                    BILLING & PAYMENTS                              │
│                                                                   │
│  expenses ◄─── invoice_line_items (vat_rate: 6/12/21%)           │
│      │    ◄─── charge_distributions (percentage, amount)          │
│      │    ◄─── payment_reminders (level: Gentle..LegalAction)     │
│      │                                                            │
│      └─── payments (idempotency_key, stripe_intent_id)            │
│                ◄─── payment_methods (stripe_method_id, is_default) │
│                                                                   │
│  call_for_funds ◄─── owner_contributions (payment_status)         │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                    MAINTENANCE                                    │
│                                                                   │
│  tickets (priority/SLA, status, category, due_date)               │
│      ◄─── quotes (scoring: prix 40%, delai 30%, garantie 20%)    │
│      ◄─── contractor_reports (magic_token_hash, photos)           │
│                                                                   │
│  work_reports (warranty_years, warranty_type)                      │
│  technical_inspections (next_inspection_date, certificates)        │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                    COMMUNITY & GAMIFICATION                       │
│                                                                   │
│  local_exchanges (exchange_type, credits, ratings)                │
│      ◄─── owner_credit_balances (balance, participation_level)   │
│                                                                   │
│  polls ◄─── poll_options ◄─── poll_votes (UNIQUE poll+owner)     │
│                                                                   │
│  achievements ◄─── user_achievements (times_earned)               │
│  challenges ◄─── challenge_progress (current_value, completed)    │
│                                                                   │
│  notices · skills · shared_objects · resource_bookings             │
└──────────────────────────────────────────────────────────────────┘
```

### 5.2 Contraintes et triggers notables

| Contrainte | Table | Type | Reference |
|-----------|-------|------|-----------|
| `validate_unit_ownership_total` | `unit_owners` | Trigger PostgreSQL | INV-1, FR-002 |
| `UNIQUE(poll_id, owner_id)` | `poll_votes` | Constraint | FR-017 |
| `UNIQUE(idempotency_key)` | `payments` | Constraint | INV-7, FR-010 |
| `CHECK(starts_at < ends_at)` | `polls`, `challenges` | Check constraint | FR-017 |
| `CHECK(credits >= 1 AND credits <= 100)` | `local_exchanges` | Check constraint | FR-016 |
| `CHECK(provider_id != requester_id)` | `local_exchanges` | Check constraint | FR-016 |
| Partial index `WHERE status = 'Active'` | `challenges`, `polls` | Index partiel | Optimisation requetes |
| Partial index `WHERE processing_restricted = TRUE` | `users` | Index partiel | FR-015 (GDPR Art. 18) |
| ENUMs custom | 20+ tables | Type PostgreSQL | Toutes les machines a etats |

---

## 6. API REST

> 559 endpoints organises par module. Base URL : `http://localhost:8080/api/v1`. Derivee du PRD section 6 (FRs) et du CLAUDE.md (liste complete).

### 6.1 Repartition par bounded context

| Bounded Context | Nombre d'endpoints | Fichier handler principal | FR PRD |
|----------------|-------------------|--------------------------|--------|
| Building Management | ~15 | `building_handlers.rs`, `unit_handlers.rs`, `unit_owner_handlers.rs`, `public_handlers.rs` | FR-001, FR-002 |
| Identity & Access | ~20 | `auth_handlers.rs`, `two_factor_handlers.rs`, `user_handlers.rs`, `organization_handlers.rs` | FR-018 |
| General Assembly | ~65 | `meeting_handlers.rs`, `resolution_handlers.rs`, `convocation_handlers.rs`, `ag_session_handlers.rs`, `age_request_handlers.rs` | FR-003, FR-004, FR-005 |
| Accounting | ~30 | `account_handlers.rs`, `budget_handlers.rs`, `etat_date_handlers.rs`, `financial_report_handlers.rs`, `dashboard_handlers.rs` | FR-006, FR-007, FR-008 |
| Billing & Payments | ~80 | `expense_handlers.rs`, `payment_handlers.rs`, `payment_method_handlers.rs`, `payment_reminder_handlers.rs`, `owner_contribution_handlers.rs`, `call_for_funds_handlers.rs`, `charge_distribution_handlers.rs` | FR-009, FR-010, FR-011, FR-012 |
| Maintenance | ~60 | `ticket_handlers.rs`, `quote_handlers.rs`, `work_report_handlers.rs`, `technical_inspection_handlers.rs`, `contractor_report_handlers.rs` | FR-013, FR-014 |
| Notifications | ~11 | `notification_handlers.rs` | FR-019 |
| GDPR & Compliance | ~15 | `gdpr_handlers.rs`, `gdpr_art30_handlers.rs`, `admin_gdpr_handlers.rs`, `consent_handlers.rs` | FR-015 |
| Community | ~90 | `poll_handlers.rs`, `notice_handlers.rs`, `skill_handlers.rs`, `shared_object_handlers.rs`, `resource_booking_handlers.rs` + SEL | FR-016, FR-017 |
| Gamification | ~25 | `gamification_handlers.rs` | — |
| Documents | ~10 | `document_handlers.rs` | — |
| Energy & IoT | ~25 | `energy_campaign_handlers.rs`, `energy_bill_upload_handlers.rs`, `iot_handlers.rs` | — |
| Board Management | ~20 | `board_member_handlers.rs`, `board_decision_handlers.rs` | — |
| Transversal | ~5 | `health.rs`, `metrics.rs`, `seed_handlers.rs`, `stats_handlers.rs` | — |
| **TOTAL** | **~559** | **60 fichiers handlers** | **FR-001 a FR-019** |

### 6.2 Endpoints publics (sans authentification)

| Endpoint | Methode | Description | FR PRD |
|----------|---------|-------------|--------|
| `/api/v1/health` | GET | Health check | — |
| `/api/v1/public/buildings/:slug/syndic` | GET | Information syndic publique (loi belge) | FR-001 |
| `/api/v1/contractor-reports/magic/:token` | GET | Rapport prestataire via magic link 72h | FR-014 |

### 6.3 Authentification

Toutes les routes (sauf publiques ci-dessus) sont protegees par middleware JWT. Le token contient : `user_id`, `organization_id`, `role_id`, `active_role`. Le refresh token rotation est implemente avec revocation.

---

## 7. Frontend (Astro + Svelte)

> Derivee du brief section 15 (Green IT, anti-bloatware) et PRD section 7.6 (accessibilite), 7.8 (ecologie), 7.9 (i18n).

### 7.1 Architecture

```
frontend/
├── src/
│   ├── components/     # 178 composants Svelte (Islands)
│   │   ├── BuildingList.svelte
│   │   ├── BuildingDetail.svelte
│   │   ├── UnitOwners.svelte
│   │   ├── OwnerList.svelte
│   │   ├── OwnerCreateModal.svelte
│   │   ├── ResolutionVoting.svelte
│   │   ├── ConvocationForm.svelte
│   │   ├── PaymentForm.svelte
│   │   ├── TicketCreate.svelte
│   │   ├── PollResults.svelte
│   │   ├── SELExchange.svelte
│   │   ├── Navigation.svelte (selecteur multi-role)
│   │   └── ... (166 autres)
│   ├── layouts/        # Layouts Astro
│   ├── pages/          # 82 pages Astro
│   │   ├── buildings/
│   │   ├── admin/
│   │   ├── accountant/
│   │   ├── contractor/
│   │   └── ...
│   ├── lib/
│   │   ├── api/        # 22 API clients (fetch wrappers)
│   │   ├── stores/     # Svelte stores (auth, locale, etc.)
│   │   ├── utils/      # 13 shared utils/validators/services
│   │   └── i18n/       # 4 langues (FR/NL/EN/DE), ~2000 cles
│   └── styles/         # Tailwind CSS 3.x
├── tests/
│   └── e2e/
│       ├── scenarios/  # 12 Documentation Vivante scenarios
│       └── *.spec.ts   # 49 smoke tests Playwright
└── package.json
```

### 7.2 Principes architecturaux

- **Islands Architecture** (Astro) : Le JavaScript n'est charge que pour les composants interactifs (Svelte islands). Les pages statiques sont du HTML pur. Cela garantit < 50 KB de JS par page (brief section 15, Green IT).
- **SSG (Static Site Generation)** : Pages pre-rendues au build. Hydratation partielle uniquement pour les composants Svelte interactifs.
- **i18n** : 4 langues (FR/NL/EN/DE), ~2000 cles par locale, 73% couverture actuelle (PRD section 7.9).
- **Architecture hexagonale legere** : Les API clients encapsulent les appels REST (equivalent des "ports" cote frontend). Les stores Svelte gerent l'etat local.

### 7.3 Mapping pages -> FRs PRD

| Page/Section | Composants cles | FR PRD |
|-------------|----------------|--------|
| `/buildings` | BuildingList, BuildingDetail | FR-001 |
| `/buildings/:id/units` | UnitOwners, OwnerCreateModal | FR-002 |
| `/convocations` | ConvocationForm, RecipientTracking | FR-003 |
| `/meetings/:id/resolutions` | ResolutionVoting, VoteResults | FR-004 |
| `/age-requests` | AgeRequestForm, CosignatoryList | FR-005 |
| `/accountant/*` | AccountingDashboard, JournalEntryForm | FR-006 |
| `/budgets` | BudgetWorkflow, VarianceChart | FR-007 |
| `/etats-dates` | EtatDateForm, OverdueList | FR-008 |
| `/expenses` | ExpenseForm, InvoiceLines, ApprovalWorkflow | FR-009 |
| `/payments` | PaymentForm, PaymentMethodManager | FR-010 |
| `/payment-reminders` | ReminderTimeline, EscalationView | FR-011 |
| `/call-for-funds` | CallForFundsForm, ContributionTracker | FR-012 |
| `/tickets` | TicketCreate, TicketWorkflow, SLATracker | FR-013 |
| `/quotes` | QuoteComparison, ScoringChart | FR-014 |
| `/gdpr/*` | GdprExport, GdprRectify, GdprRestrict | FR-015 |
| `/exchanges` | SELExchange, Leaderboard, CreditBalance | FR-016 |
| `/polls` | PollCreate, PollVote, PollResults | FR-017 |
| `/login`, `/auth/*` | LoginForm, TwoFactorSetup, RoleSwitcher | FR-018 |
| `/notifications` | NotificationList, PreferenceManager | FR-019 |

---

## 8. ADR (Architecture Decision Records)

### ADR-001 : SOLID + Architecture Hexagonale (impose par Methode Maury)

- **Contexte** : La Methode Maury impose une architecture hexagonale stricte avec les 5 principes SOLID.
- **Decision** : 3 couches (Domain / Application / Infrastructure) avec dependances unidirectionnelles vers l'interieur. Le Domain n'a AUCUNE dependance externe.
- **Rationale** : Isolation de la logique metier. Testabilite maximale (domain testable sans infrastructure). Extensibilite (nouveaux adaptateurs sans modifier le core). La legislation belge evolue regulierement (reforms Art. 577 CC), et les regles metier isolees permettent des modifications sans impact infrastructure (brief section 14, risque "Evolution legislative").
- **Consequences** : 60 fichiers ports + 60 fichiers implementations. Overhead initial mais maintenabilite superieure a long terme.
- **Tracabilite** : Brief section 15, PRD section 10.1 contrainte 2.

### ADR-002 : TDD + BDD (impose par Methode Maury)

- **Contexte** : La Methode Maury impose le developpement guide par les tests.
- **Decision** : Test-first systematique. 819 scenarios BDD comme specifications vivantes. Pyramide complete (unit -> integration -> BDD -> E2E -> benchmarks).
- **Rationale** : Les scenarios BDD Gherkin sont la documentation vivante du systeme. Ils tracent directement vers les FRs du PRD et les invariants du brief. La Test-Driven Emergence (brief section 15) garantit que l'application emerge des tests, pas l'inverse.
- **Consequences** : 74 fichiers .feature, 49 E2E smoke tests, 12 Documentation Vivante scenarios. Temps de CI significatif mais confiance elevee dans les regressions.
- **Tracabilite** : Brief section 15, PRD section 7.5, NFR-005.

### ADR-003 : DDD Ubiquitous Language belge (impose par Methode Maury)

- **Contexte** : La Methode Maury impose le Domain-Driven Design avec langage ubiquitaire.
- **Decision** : Le glossaire metier (brief section 8, PRD section 4) est la source de verite unique. Les termes belges (tantieme, quorum, syndic, PCMN, etat date, AGE, procuration, etc.) sont utilises tel quel dans le code, les tests BDD, et la documentation. Aucune traduction ou synonyme.
- **Rationale** : Un syndic belge qui lit le code doit reconnaitre ses concepts metier. Les scenarios BDD doivent etre lisibles par un non-developpeur. Le risque de malentendu est elimine quand tout le monde utilise les memes termes.
- **Consequences** : 60 entites domaine avec noms metier belges. `Resolution`, `Vote`, `Convocation`, `EtatDate`, `PaymentReminder`, `CallForFunds`, etc.
- **Tracabilite** : Brief section 8, PRD section 4.

### ADR-004 : Rust + Actix-web pour le backend

- **Contexte** : Choix du langage et framework backend.
- **Decision** : Rust 1.82+ avec Actix-web 4.9.
- **Rationale** : Performance P99 < 5ms (PRD NFR-001). Memoire < 128 MB par instance. Empreinte CO2 < 0.5g/requete (actuel: 0.12g). Safety (pas de null pointer, pas de data race). Compilation avec LTO + codegen-units=1 pour binaire optimal.
- **Consequences** : Courbe d'apprentissage Rust. Compilation lente (mitigee par cargo-watch en dev). 137k+ LOC Rust deja ecrites.
- **Tracabilite** : Brief section 13 (stack imposee), PRD section 7.1, section 7.8 (Green IT).

### ADR-005 : PostgreSQL 15 avec sqlx compile-time verification

- **Contexte** : Choix de la base de donnees.
- **Decision** : PostgreSQL 15 avec sqlx 0.8 pour verification compile-time des requetes SQL.
- **Rationale** : Conformite PCMN belge (triggers, ENUMs custom, indexes partiels). ACID pour la comptabilite double-entree. Connection pool max 10 pour economiser les ressources. Le compile-time check de sqlx elimine les erreurs SQL au runtime.
- **Consequences** : 82 migrations SQL. Dependance forte a PostgreSQL (acceptable car pas de changement de SGBD prevu). Necesssite `sqlx prepare` pour le cache offline.
- **Tracabilite** : Brief section 13 (stack imposee), PRD section 10.1 contrainte 1.

### ADR-006 : Astro + Svelte Islands pour le frontend

- **Contexte** : Choix du framework frontend.
- **Decision** : Astro 4.x (SSG) + Svelte 4.x (Islands) + Tailwind CSS 3.x.
- **Rationale** : JavaScript minimal (Green IT, < 50 KB/page). SSG pour les pages statiques (SEO, performance). Svelte pour les composants interactifs uniquement (hydratation partielle). 178 composants Svelte deja ecrits.
- **Consequences** : Pas de SPA (Single Page Application). Navigation entre pages = rechargement HTML. Acceptable pour une application de gestion (pas un jeu temps reel).
- **Tracabilite** : Brief section 13, section 15 (Green IT), PRD section 7.8.

### ADR-007 : Multi-tenancy par organisation_id

- **Contexte** : Isolation des donnees entre syndics/coproprietes.
- **Decision** : Multi-tenancy au niveau applicatif via `organization_id` sur chaque table. Middleware JWT injecte `organization_id` dans chaque requete.
- **Rationale** : Simple a implementer. Pas de schema separee par tenant (complexite). Isolation garantie par les use cases (filtrage systematique par `organization_id`).
- **Consequences** : Toutes les tables ont `organization_id`. Risque de fuite de donnees si un use case oublie le filtre (mitige par tests BDD multitenancy).
- **Tracabilite** : Brief section 7 capacite 16, PRD FR-018.

### ADR-008 : Invariants dans les constructeurs Domain

- **Contexte** : Comment garantir les regles metier critiques.
- **Decision** : Chaque entite Domain a un constructeur `::new() -> Result<Self, String>` qui valide les invariants. Une violation = erreur immediate. Jamais silencieuse.
- **Rationale** : Les 10 invariants du brief (section 10) sont des regles metier inviolables. Les coder dans le constructeur garantit qu'aucune entite invalide n'est jamais creee, independamment du chemin d'appel (API, seed, migration).
- **Consequences** : Toute creation d'entite passe par `::new()`. Pas de constructeur par defaut. Chaque FR du PRD qui reference un invariant (INV-1 a INV-10) est trace vers le constructeur correspondant.
- **Tracabilite** : Brief section 10 (10 invariants), PRD section 6 (invariants par FR).

---

## 9. Securite & RGPD

> Derivee du brief section 15 (securite by design) et PRD section 7.2 (NFR-002). Implementation complete des Issues #39, #40, #41, #43, #78.

### 9.1 Defense en profondeur

```
Couche 1 : Reseau
├── Traefik (TLS termination, reverse proxy)
├── fail2ban (SSH, Traefik, API abuse, PostgreSQL brute-force)
├── CrowdSec WAF (community threat intelligence)
└── Suricata IDS (regles custom SQL injection, XSS, path traversal)

Couche 2 : Systeme
├── LUKS AES-XTS-512 (chiffrement at-rest PostgreSQL + uploads)
├── SSH Hardening (key-only, modern ciphers)
├── Kernel Hardening (SYN cookies, IP spoofing protection, ASLR)
└── Audit (Lynis hebdomadaire >80/100, rkhunter quotidien, AIDE integrity)

Couche 3 : Application
├── 2FA TOTP (backup codes, FR-018)
├── Rate Limiting (5 tentatives/15 min/IP)
├── JWT (secret >= 32 chars, CORS sans wildcards)
├── Headers (HSTS 1 an, CSP, X-Frame-Options, X-Content-Type-Options)
└── RBAC (SuperAdmin, Syndic, Owner, Comptable)

Couche 4 : Donnees
├── RGPD Articles 15-21 + 30 (FR-015)
├── Backups GPG-encrypted + S3 off-site (7j local)
├── Audit trail complet (IP, user-agent, timestamp)
└── Anonymisation (Art. 17: donnees personnelles -> "Anonyme_xxxxx")
```

### 9.2 Conformite RGPD complete (FR-015)

| Article RGPD | Implementation | Endpoint | Audit Event |
|-------------|---------------|----------|-------------|
| Art. 15 (Acces) | Export JSON toutes donnees personnelles | `GET /gdpr/export` | `GdprDataExported` |
| Art. 16 (Rectification) | Correction email, nom, prenom | `PUT /gdpr/rectify` | `GdprDataRectified` |
| Art. 17 (Effacement) | Anonymisation + verification eligibilite (obligation legale) | `DELETE /gdpr/erase` | `GdprDataErased` |
| Art. 18 (Restriction) | `processing_restricted = true`, suspension traitements automatises | `PUT /gdpr/restrict-processing` | `GdprProcessingRestricted` |
| Art. 21 (Opposition) | `marketing_opt_out = true`, seules notifications legales conservees | `PUT /gdpr/marketing-preference` | `GdprMarketingOptOut` |
| Art. 30 (Registre) | Audit trail complet via `AuditLogRepository` | `GET /gdpr/art30/*` | Tous les evenements |

### 9.3 Securite des paiements (FR-010, INV-7)

- **PCI-DSS** : Aucun stockage de donnees carte raw. Uniquement Stripe tokens (`pm_xxx`, `sepa_debit_xxx`).
- **Idempotency** : Cle unique >= 16 caracteres. Contrainte UNIQUE PostgreSQL. Previent les doubles charges sur retry reseau.
- **Anti-over-refund** : Validation `refunded_amount_cents <= amount_cents`. Le remboursement ne peut jamais depasser le montant paye.

---

## 10. Infrastructure de deploiement

> Derivee du brief section 13 (contraintes infra) et PRD section 7.4 (exploitabilite ITIL).

### 10.1 Environments

| Environnement | Infra | URL | Usage |
|--------------|-------|-----|-------|
| Local | Docker Compose | `localhost:8080` (API), `localhost:3000` (frontend) | Developpement |
| Dev | VPS OVH | — | Integration continue |
| Staging | VPS OVH | — | Pre-production |
| Production | VPS OVH -> K3s -> K8s | — | Production |

### 10.2 Docker Compose (developpement)

```
docker-compose.yml
├── postgres (PostgreSQL 15, port 5432)
├── backend (Rust + cargo-watch, port 8080)
├── frontend (Astro dev server, port 3000)
├── prometheus (port 9090)
├── grafana (port 3001)
├── loki (logs)
└── alertmanager (port 9093)
```

**Commandes** :
```bash
make docker-up           # PostgreSQL seul
make dev-all             # Tous les services
docker-compose logs -f   # Logs
```

### 10.3 Progression infrastructure par capacite

| Jalon | Coproprietes | Infra | Justification |
|-------|-------------|-------|---------------|
| 0-1 | 10-100 | VPS OVH (Docker Compose) | Cout minimal, suffisant pour beta |
| 2-3 | 200-1000 | K3s (monosite) | Orchestration legere, HA |
| 4-5 | 1000-5000 | K3s (multisite) | Scalabilite horizontale |
| 6-7 | 5000-10000+ | K8s OVH Managed | Scalabilite enterprise |

### 10.4 CI/CD (GitHub Actions)

```
.github/workflows/
├── ci.yml                 # Tests (unit, integration, BDD, E2E), lint, fmt
├── security.yml           # Audit securite, dependances
├── docker-build-push.yml  # Build images Docker
└── docs.yml               # Generation documentation Sphinx
```

**Pipeline CI** :
1. `cargo fmt -- --check` (formatage)
2. `cargo clippy -- -D warnings` (lint)
3. `cargo test --lib` (unit tests)
4. `cargo test --test bdd` (819 scenarios BDD)
5. `cargo test --test e2e` (E2E backend)
6. `npm run build` (frontend build check)
7. `npm run format -- --check` (prettier)
8. `npm audit` (securite dependencies)

---

## 11. IaC (Terraform + Ansible)

> Preparation pour la phase ITIL/production. Structure reelle dans `infrastructure/` et le repo `koprogo-infra-restructure`.

### 11.1 Terraform : Provisioning

```
infrastructure/
├── _shared/
│   └── terraform/
│       └── modules/
│           ├── ovh-vps/        # Module VPS OVH
│           ├── ovh-k3s/        # Module K3s
│           ├── ovh-k8s/        # Module K8s OVH Managed
│           └── networking/     # Reseau, DNS
├── monosite/
│   ├── vps/
│   │   ├── dev/terraform/
│   │   ├── integration/terraform/
│   │   ├── staging/terraform/
│   │   └── production/terraform/
│   └── k3s/
│       ├── dev/terraform/
│       ├── integration/terraform/
│       ├── staging/terraform/
│       └── production/terraform/
└── multisite/
    └── k8s/
        ├── dev/terraform/
        ├── integration/terraform/
        ├── staging/terraform/
        └── production/terraform/
```

### 11.2 Ansible : Configuration et securite

```
infrastructure/_shared/ansible/
├── group_vars/              # Variables par environnement
├── roles/
│   ├── common/tasks/        # Configuration de base
│   ├── docker/tasks/        # Installation Docker
│   ├── security/
│   │   ├── tasks/           # LUKS, fail2ban, Suricata, CrowdSec, SSH hardening
│   │   └── handlers/
│   ├── monitoring/tasks/    # Prometheus, Grafana, Loki, Alertmanager
│   ├── backup/tasks/        # GPG-encrypted backups + S3
│   ├── gitops/
│   │   ├── tasks/
│   │   └── templates/
│   ├── k3s-master/
│   │   ├── defaults/
│   │   └── tasks/
│   ├── k3s-agent/
│   │   ├── defaults/
│   │   └── tasks/
│   ├── argocd/tasks/        # GitOps
│   ├── vault/
│   │   ├── defaults/
│   │   └── tasks/
│   ├── velero/tasks/        # Backups Kubernetes
│   ├── dns/tasks/
│   └── pgo/tasks/           # PostgreSQL Operator
└── ...
```

### 11.3 Policy-as-Code

- **OPA/Conftest** : Validation des configurations Terraform et Kubernetes contre les politiques de securite.
- **Conformite ISO 27001** : Preparation pour certification (Jalon 5+).

### 11.4 Deploiement rapide

```bash
# VPS (Docker Compose)
make -f infrastructure/Makefile.infra ansible-setup ENV=production ARCH=vps SITE=monosite

# K3s
make -f infrastructure/Makefile.infra ansible-setup ENV=production ARCH=k3s SITE=monosite
```

---

## 12. Observabilite

> Derivee du PRD section 7.7 (NFR-007) et brief section 16 (metriques de succes).

### 12.1 Stack d'observabilite

| Outil | Role | Retention | Port | Metrique cible PRD |
|-------|------|-----------|------|-------------------|
| **Prometheus** | Metriques applicatives + infra | 30 jours | 9090 | P99 < 5ms, throughput > 100k req/s, memoire < 128 MB |
| **Grafana** | Dashboards + alertes visuelles | — | 3001 | Uptime 99.9%, latence, throughput |
| **Loki** | Agregation logs | 7 jours | — | Debug, audit trail |
| **Alertmanager** | Routage alertes | — | 9093 | Incidents P1 = 0/mois |
| **Backend `/metrics`** | Endpoint Prometheus scrape | — | 8080 | Toutes metriques applicatives |

### 12.2 Metriques applicatives cles

| Metrique | Source | Seuil alerte | Tracabilite |
|----------|--------|-------------|-------------|
| `http_request_duration_p99` | Actix-web middleware | > 5ms | PRD NFR-001, brief s16 |
| `http_requests_total` | Actix-web middleware | < 100k req/s | PRD NFR-001 |
| `process_resident_memory_bytes` | Prometheus | > 128 MB | PRD NFR-001 |
| `pg_pool_connections_active` | sqlx | > 10 | ADR-005 |
| `login_failed_total` | auth_handlers | > 5/15min/IP | FR-018 (rate limiting) |
| `gdpr_operations_total` | gdpr_handlers | — | FR-015 (audit Art. 30) |
| `payment_status_total{status}` | payment_handlers | — | FR-010 |
| `bdd_scenarios_passed` | CI/CD | < 819 | PRD S6 |

### 12.3 Alertes critiques

| Alerte | Condition | Severite | Action |
|--------|----------|---------|--------|
| `HighLatencyP99` | P99 > 5ms pendant 5 min | Critical | Investigation immediate |
| `HighMemory` | RSS > 128 MB | Warning | Restart + investigation |
| `DatabasePoolExhausted` | Connections = 10/10 | Critical | Scale ou optimize queries |
| `HighErrorRate` | HTTP 5xx > 1% | Critical | Investigation + rollback si necessaire |
| `LynisScoreLow` | Score < 80/100 | Warning | Remediation securite |
| `BackupFailed` | Backup quotidien echoue | Critical | Intervention manuelle |

---

## 13. Scalabilite organisationnelle

> Derivee du PRD section 7.3 (NFR-003) et brief section 13 (contraintes organisation).

### 13.1 Progression Scrum -> Nexus -> SAFe

| Phase | Organisation | Taille equipe | Trigger | Comment l'architecture le supporte |
|-------|-------------|---------------|---------|-------------------------------------|
| **Phase 1** (actuelle) | **Scrum** | 1 dev + IA | Initial | 1 backlog, 1 sprint. Les 13 bounded contexts sont developpes sequentiellement. |
| **Phase 2** | **Nexus** | 3-9 equipes | > 7 devs OU > 3 bounded contexts en parallele | Chaque equipe prend 1-3 bounded contexts. Les ports (traits) sont les contrats inter-equipes. L'equipe "Building Management" ne casse pas l'equipe "General Assembly" car elles communiquent via les ports definis. |
| **Phase 3** | **SAFe** | 50+ agents | > 9 equipes | ARTs (Agile Release Trains) par domaine metier. Feature flags pour releases progressives par PI (Program Increment). |

### 13.2 Comment l'architecture hexagonale active la scalabilite equipes

**Modules independants = equipes independantes** : Chaque bounded context (Building, Assembly, Accounting, etc.) a ses propres entites, ports, use cases, handlers et repositories. Deux equipes peuvent travailler en parallele sur des contextes differents sans conflit de merge.

**Ports (traits) = contrats inter-equipes** : Quand l'equipe "Billing" a besoin de lire les quotes-parts des coproprietaires, elle depend du trait `UnitOwnerRepository` (contrat), pas de l'implementation PostgreSQL. L'equipe "Building" peut refactorer son implementation sans casser l'equipe "Billing".

**DTOs = API contracts** : Les DTOs definissent le contrat entre les couches et entre les equipes. Un changement de DTO est un changement d'API contract qui necesssite une coordination inter-equipes (revue Nexus).

**Tests BDD = specifications partagees** : Les 74 fichiers .feature sont la documentation vivante commune. Quand une equipe modifie un comportement, le scenario BDD correspondant echoue, signalant le changement aux equipes dependantes.

### 13.3 Preparation ITIL

| Processus ITIL | Implementation | Outil |
|---------------|---------------|-------|
| Incident Management | Alertmanager -> Grafana -> runbooks | Alertmanager |
| Change Management | PR review obligatoire + CI/CD | GitHub Actions |
| Release Management | Tags semantiques, progression par jalons | GitHub Releases |
| Configuration Management | IaC (Terraform + Ansible) | Terraform, Ansible |
| Service Level Management | SLA tickets (PRD FR-013) : Critical 1h, Urgent 4h, High 24h | Prometheus + Grafana |
| Capacity Management | Metriques Prometheus, progression infra par jalon | Prometheus |

---

## Tracabilite Architecture -> PRD -> Brief

| Section Architecture | Section PRD | Section Brief | FRs/INVs/NFRs |
|---------------------|-------------|--------------|----------------|
| 1. Vue d'ensemble | s7 (NFRs) | s15 (principes) | NFR-001 a NFR-009 |
| 2. Bounded Contexts | s5 (modules) | s9 (bounded contexts) | Tous les 13 contextes |
| 3. Architecture SOLID | s6 (FRs, principes SOLID) | s10 (invariants), s15 (SOLID) | INV-1 a INV-10 |
| 4. Tests | s7.5 (testabilite) | s15 (TDD/BDD) | NFR-005, S5-S7 |
| 5. Modele de donnees | s8 (tables) | s9 (entites DDD) | FR-001 a FR-019 |
| 6. API REST | s6 (endpoints par FR) | s7 (capacites) | FR-001 a FR-019 |
| 7. Frontend | s7.6 (accessibilite), s7.8 (ecologie) | s15 (Green IT) | NFR-006, NFR-008 |
| 8. ADRs | s10 (contraintes) | s13 (contraintes), s15 (principes) | Toutes contraintes |
| 9. Securite & RGPD | s7.2 (securite) | s15 (securite by design) | NFR-002, FR-015 |
| 10. Infrastructure | s7.4 (ITIL) | s13 (infra) | NFR-004 |
| 11. IaC | s10.1 contrainte 5 | s13 (IaC) | — |
| 12. Observabilite | s7.7 (observabilite) | s16 (metriques) | NFR-007, S1-S4 |
| 13. Scalabilite org | s7.3 (scalabilite) | s13 (Scrum->Nexus->SAFe) | NFR-003 |

---

## Pipeline suivant

Ce document d'architecture sera consomme par :
- **Etape 4** : Scrum Master (stories TDD pour agents IA) -> `epics-and-stories.md`
- **Etape 5** : Validation croisee -> `validation-report.md`

---

*Document genere par Winston (Architecte BMAD) -- Methode Maury Phase TOGAF C-D*
*Pipeline : product-brief.md (Mary, Phase A) -> PRD.md (John, Phase B-C) -> architecture.md (Winston, Phase C-D)*
*Prochaine etape : Epics & Stories (Phase D-E) par le Scrum Master BMAD*

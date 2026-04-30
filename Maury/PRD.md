# PRD — KoproGo

## Methode Maury — Phase TOGAF B-C (Business + SI)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : John (Product Manager)
**Date** : 29/03/2026
**Version** : 1.0
**Brief source** : `Maury/product-brief.md` v1.0 (29/03/2026, Mary — Analyste)

**Stack** : Rust 1.75+ / Actix-web 4.9 + Astro 4.x / Svelte 4.x + PostgreSQL 15
**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Organisation** : Scrum → Nexus → SAFe → ITIL
**Dev** : Agents IA supervises par Gilles Maury

---

## 1. Resume executif

KoproGo est une plateforme SaaS de gestion de copropriete belge qui remplace les outils fragmentes (Excel, courrier papier, logiciels obsoletes) par une solution integree, performante (P99 < 5ms) et conforme au droit belge (Article 577 et suivants du Code Civil). Elle couvre l'integralite du cycle de vie d'une copropriete : gestion immobiliere avec tantiemes, comptabilite PCMN normalisee (AR 12/07/2012), assemblees generales numeriques avec vote et quorum legal, facturation TVA belge, recouvrement automatise, conformite GDPR complete (Articles 15-21), ticketing maintenance avec SLA, paiements Stripe/SEPA, et modules communautaires (SEL, sondages, gamification).

L'architecture repose sur le Domain-Driven Design et l'Architecture Hexagonale en Rust, garantissant une empreinte memoire < 128 MB par instance et une empreinte ecologique < 0.5g CO2/requete. Le frontend Astro + Svelte (Islands Architecture) assure une experience reactive avec un JavaScript minimal. La base de code existante comprend 559 endpoints API, 59 entites domaine, 80 migrations PostgreSQL et 137k+ LOC Rust.

La progression suit un modele par capacites : chaque jalon debloque un palier d'adoption mesurable, de 10-20 coproprietes (Jalon 0, acheve) jusqu'a 10 000+ (Jalon 7). Le MVP (Jalons 0-3) cible 500-1 000 coproprietes avec les modules fondamentaux. KoproGo est structure en ASBL, licence AGPL-3.0, finance par cotisations (5 EUR/copro/mois cloud).

**Tracabilite** : Ce PRD consomme et transforme le product brief (Mary, Phase TOGAF A) en exigences fonctionnelles detaillees avec scenarios BDD Gherkin. Chaque FR trace vers les personas (brief section 6), le glossaire (brief section 8), les bounded contexts (brief section 9), et les invariants (brief section 10).

---

## 2. Objectifs produit (mesurables)

| # | Objectif | Metrique cible | Echeance (Jalon) | Tracabilite brief |
|---|----------|----------------|------------------|-------------------|
| O1 | Conformite legale belge integrale | 100% checklist Art. 577 CC | Jalon 5 (80% J2, 95% J4) | Brief section 16 |
| O2 | Performance sous charge | P99 < 5ms, >100k req/s | Continu | Brief section 16 |
| O3 | Adoption beta publique | 50-100 coproprietes | Jalon 1 | Brief section 11 |
| O4 | Adoption production | 200-500 coproprietes | Jalon 2 | Brief section 11 |
| O5 | Conformite GDPR complete | 6/6 articles (15,16,17,18,21,30) | Jalon 1 (acheve) | Brief section 16 |
| O6 | Reduction temps administratif syndic | < 5 min par convocation AG (vs 2h) | Jalon 2 | Brief section 16 |
| O7 | Taux recouvrement automatise | > 85% avant niveau LegalAction | Jalon 3 | Brief section 16 |
| O8 | Securite infrastructure | Score Lynis > 80/100 | Continu | Brief section 16 |
| O9 | Empreinte ecologique | < 0.5g CO2/requete (actuel: 0.12g) | Continu | Brief section 16 |
| O10 | Couverture tests domain | 100% | Continu | Brief section 16 |
| O11 | Coproprietes hebergees | 500-1000 | Jalon 3 | Brief section 16 |
| O12 | Scenarios BDD | >= 819 scenarios, 69 features | Continu | Brief section 16 |

---

## 3. Perimetre

### 3.1 MVP (Jalons 0-3 — release v0.1.0)

> Repris du brief section 11 — fonctionnalites cles.

| Module | Priorite MoSCoW | Jalon | Statut |
|--------|----------------|-------|--------|
| Gestion immobiliere (Building, Unit, UnitOwner) | MUST | 0 | Acheve |
| Authentification multi-role + 2FA TOTP | MUST | 1 | Acheve |
| Conformite GDPR (Art. 15-21) | MUST | 1 | Acheve |
| Securite infrastructure (LUKS, IDS, WAF) | MUST | 1 | Acheve |
| Comptabilite PCMN belge | MUST | 2 | Acheve |
| Assemblees generales numeriques | MUST | 1-3 | En cours |
| Facturation TVA belge + Paiements Stripe/SEPA | MUST | 2-3 | Acheve |
| Recouvrement automatise 4 niveaux | MUST | 3 | Acheve |
| Ticketing maintenance + Devis | SHOULD | 3 | Acheve |
| Modules communautaires (SEL, sondages, annonces) | SHOULD | 3 | Acheve |
| Gamification (achievements, challenges) | SHOULD | 3 | Acheve |

### 3.2 Post-MVP (Jalons 4-7)

> Repris du brief section 12 — fonctionnalites secondaires.

| Module | Priorite | Jalon |
|--------|----------|-------|
| MCP AI Syndic (serveur SSE + JSON-RPC) | COULD | 4 |
| Authentification itsme/eID | COULD | 4 |
| Application Tauri (desktop/mobile, offline SQLite) | COULD | 5 |
| PWA + API publique + SDK | COULD | 5 |
| IA Assistant Syndic reglementaire | WON'T v0.1.0 | 6 |
| API Bancaire PSD2 (reconciliation automatique) | WON'T v0.1.0 | 6 |
| IoT temps reel (MQTT + TimescaleDB) | WON'T v0.1.0 | 6 |
| Blockchain Voting (Polygon) | WON'T v0.1.0 | 7 |
| White-label federation multi-tenant | WON'T v0.1.0 | 7 |

### 3.3 Hors scope

- Gestion locative (relation bailleur-locataire)
- Immobilier commercial (bureaux, commerces)
- Marches hors Belgique (v0.1.0)
- Applications natives iOS/Android (pre-Jalon 5)
- Comptabilite hors PCMN (plan comptable francais, OHADA)

---

## 4. Glossaire metier (Ubiquitous Language DDD)

> Repris du brief section 8 et enrichi avec les termes techniques du PRD. Ce glossaire est la source de verite unique pour le langage ubiquitaire. Chaque terme est utilise tel quel dans le code Rust, les scenarios BDD Gherkin, les DTOs, et la documentation. Aucune traduction ou synonyme.

| Terme | Definition | Entite DDD | Exemple |
|-------|------------|------------|---------|
| **Tantieme / Dix-millieme** | Quote-part de copropriete (0.0 < p <= 1.0), base pour charges et pouvoir de vote. Exprimee en milliemes (/1000) ou dix-milliemes (/10000) selon la taille de l'immeuble | `UnitOwner.percentage` | Apt 3B : 450/10000 = 4.5% |
| **Quorum** | Seuil minimum de presences (physiques + procurations) pour AG valable. 50% des quotes-parts (Art. 3.87 ss5 CC) | `Meeting.quorum_percentage` | AG 10000 dix-milliemes : quorum si >= 5001 |
| **Majorite** | 4 types de majorite selon Art. 3.88 CC : Absolute (>50%), TwoThirds (>=2/3), FourFifths (>=4/5), Unanimity (100% de tous les tantiemes). Abstentions exclues sauf unanimite | `Resolution.majority_required` | Travaux structurels : majorite 2/3, dissolution : unanimite |
| **Syndic** | Personne mandatee pour gerer la copropriete (administration, comptabilite, AG, maintenance) | `User` (role Syndic) | Marc, persona brief section 6.1 |
| **Coproprietaire** | Proprietaire d'un ou plusieurs lots dans l'immeuble | `Owner` + `UnitOwner` | Sophie, persona brief section 6.2 |
| **Assemblee Generale (AG)** | Reunion obligatoire des coproprietaires pour voter les decisions relatives a l'immeuble | `Meeting` | AG annuelle "Residence du Parc" |
| **AGE** | AG Extraordinaire, declenchable sur petition 1/5 des quotes-parts (Art. 3.87 ss2 CC) | `AgeRequest` | 5 proprietaires = 22% demandent AGE |
| **Convocation** | Document officiel invitant a l'AG, delai legal 15 jours minimum (Art. 3.87 ss3 CC) | `Convocation` | Envoyee 1er mars pour AG 20 mars |
| **Resolution** | Proposition soumise au vote avec majorite requise | `Resolution` | "Travaux toiture 45 000 EUR" |
| **Procuration** | Delegation de pouvoir de vote a un autre coproprietaire present | `Vote.proxy_owner_id` | Sophie delegue a Alice |
| **PCMN** | Plan Comptable Minimum Normalise belge (AR 12/07/2012), 8 classes comptables | `Account` | Classe 6 : Charges |
| **Appel de fonds** | Demande de paiement collective aux coproprietaires | `CallForFunds` | Appel Q1 : 25 000 EUR |
| **Etat date** | Document legal pour vente de lot (Art. 577-11 ss2 CC) | `EtatDate` | Lot 12A : solde 1 250 EUR |
| **Conseil de copropriete** | Organe de controle obligatoire pour coproprietes >20 lots | `BoardMember` | 5 membres elus |
| **SEL** | Systeme d'Echange Local, monnaie temps (1h = 1 credit). Legal en Belgique si non commercial | `LocalExchange` | Pierre : 2h jardinage = 2 credits |
| **Charge distribution** | Repartition d'une facture selon quotes-parts | `ChargeDistribution` | Facture 3 000 EUR : lot 3B paie 135 EUR |
| **Ecriture journal** | Operation comptable double-entree (debit = credit) | `JournalEntry` + `JournalEntryLine` | ACH : 1 000 EUR debit 6000, credit 4400 |
| **Magic link** | Lien JWT temporaire (72h) pour acces prestataire sans compte | `ContractorReport.magic_token_hash` | Ahmed accede via email |
| **Idempotency key** | Cle unique >= 16 chars prevenant les doubles charges | `Payment.idempotency_key` | `pay_abc123def456gh` |
| **Port** | Interface trait Rust definie par la couche Application | `BuildingRepository` trait | Contrat pour l'adaptateur PostgreSQL |
| **Adaptateur** | Implementation concrete d'un port | `PostgresBuildingRepository` | Implemente `BuildingRepository` |

---

## 5. Bounded Contexts → Modules

> Repris du brief section 9 et mappes aux modules code Rust existants.

| Bounded Context | Module Rust | Responsabilite | Entites principales |
|-----------------|-------------|----------------|---------------------|
| **Building Management** | `domain::entities::building`, `unit`, `unit_owner` | Immeubles, lots, quotes-parts, transferts | Building, Unit, UnitOwner |
| **Identity & Access** | `domain::entities::user`, `user_role_assignment`, `two_factor_secret` | Auth, RBAC, 2FA, multi-tenant | User, UserRoleAssignment, RefreshToken, TwoFactorSecret, Organization |
| **General Assembly** | `domain::entities::meeting`, `resolution`, `vote`, `convocation`, `ag_session`, `age_request` | AG, convocations, votes, visio, AGE | Meeting, Convocation, Resolution, Vote, AgSession, AgeRequest |
| **Accounting** | `domain::entities::account`, `journal_entry`, `budget`, `etat_date` | PCMN, ecritures, rapports financiers | Account, JournalEntry, JournalEntryLine, Budget, EtatDate |
| **Billing & Payments** | `domain::entities::expense`, `payment`, `payment_reminder`, `call_for_funds` | Factures, paiements, recouvrement | Expense, InvoiceLineItem, Payment, PaymentMethod, PaymentReminder, OwnerContribution, CallForFunds, ChargeDistribution |
| **Maintenance** | `domain::entities::ticket`, `quote`, `work_report`, `technical_inspection`, `contractor_report` | Tickets, devis, rapports, inspections | Ticket, Quote, WorkReport, TechnicalInspection, ContractorReport |
| **Notifications** | `domain::entities::notification`, `notification_preference` | Multi-canal, preferences, tracking | Notification, NotificationPreference |
| **GDPR & Compliance** | `application::use_cases::gdpr_use_cases` | Articles 15-21 + 30, audit trail | Operations sur User (export, erasure, rectify, restrict) |
| **Community** | `domain::entities::local_exchange`, `poll`, `community_notice`, `skill`, `shared_object`, `resource_booking` | SEL, sondages, annonces, partage | LocalExchange, Poll, CommunityNotice, Skill, SharedObject, ResourceBooking |
| **Gamification** | `domain::entities::achievement`, `challenge` | Achievements, challenges, leaderboard | Achievement, UserAchievement, Challenge, ChallengeProgress |
| **Documents** | `domain::entities::document` | Stockage fichiers, liaison entites | Document |
| **Energy & IoT** | `domain::entities::iot_reading`, `linky_device`, `energy_campaign` | Achats groupes, compteurs, Linky | EnergyCampaign, IoTReading, LinkyDevice |
| **Board Management** | `domain::entities::board_member`, `board_decision` | Conseil copropriete, decisions post-AG | BoardMember, BoardDecision |

---

## 6. Exigences fonctionnelles

---

### Module : Building Management (brief section 9, bounded context 1)

#### FR-001 : Gestion des immeubles (CRUD)

- **Description** : Creer, lire, modifier, supprimer des immeubles avec informations legales (nom, adresse, nombre total de lots, annee de construction, slug SEO, coordonnees syndic publiques).
- **Priorite** : MUST HAVE (Jalon 0, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux enregistrer la "Residence du Parc" avec ses 45 lots pour gerer la copropriete de maniere centralisee.
- **Entite DDD** : `Building` (brief section 9)
- **Invariants** (brief section 10, INV-9) :
  - `Building.name` non vide — `Building::new("", ...)` retourne `Err("Name cannot be empty")`
  - Slug SEO genere automatiquement a partir de `name + address + city`
- **Principes SOLID** :
  - SRP : `Building` ne gere que les donnees immobilieres, pas la comptabilite ni les AG
  - OCP : Ajout de champs syndic sans modifier les use cases existants
  - DIP : `BuildingRepository` trait defini dans Application, implemente par `PostgresBuildingRepository`
- **Scenarios BDD** :

```gherkin
Scenario: Creer un immeuble avec un nom valide
  Given Marc est connecte en tant que syndic
  When Marc cree l'immeuble "Residence du Parc" a "Rue de la Loi 45, 1000 Bruxelles" avec 45 lots
  Then l'immeuble "Residence du Parc" est cree avec succes
  And un slug "residence-du-parc-rue-de-la-loi-45-bruxelles" est genere automatiquement

Scenario: Refuser la creation d'un immeuble sans nom (INV-9)
  Given Marc est connecte en tant que syndic
  When Marc cree un immeuble avec un nom vide
  Then une erreur "Name cannot be empty" est retournee
  And aucun immeuble n'est cree

Scenario: Afficher les coordonnees syndic publiques sans authentification
  Given l'immeuble "Residence du Parc" a les coordonnees syndic de Marc renseignees
  When un visiteur anonyme accede a "/public/buildings/residence-du-parc-bruxelles/syndic"
  Then les coordonnees du syndic sont affichees (nom, email, telephone, horaires)
  And aucune donnee privee des coproprietaires n'est exposee
```

- **Dependances** : Aucune (module fondation)
- **Endpoints** : `GET/POST /buildings`, `GET/PUT/DELETE /buildings/:id`, `GET /public/buildings/:slug/syndic`

---

#### FR-002 : Gestion des lots et quotes-parts

- **Description** : Gerer les lots (appartements, caves, garages) au sein d'un immeuble, avec attribution de quotes-parts (tantiemes/milliemes) aux coproprietaires. Validation stricte que la somme des quotes-parts actives = 100%.
- **Priorite** : MUST HAVE (Jalon 0, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux attribuer a Sophie (brief section 6.2) sa quote-part de 450/10000 dix-milliemes pour le lot 3B, afin que ses charges et son pouvoir de vote soient correctement calcules.
- **Entites DDD** : `Unit`, `UnitOwner` (brief section 9)
- **Invariants** (brief section 10) :
  - INV-1 : Somme des quotes-parts actives = 100% (tolerance +/-0.01%) — Art. 577-2 ss4 CC
  - INV-6 : Quote-part comprise entre 0% exclus et 100% inclus (0.0 < p <= 1.0)
  - Trigger PostgreSQL `validate_unit_ownership_total` bloque les depassements > 100%
- **Principes SOLID** :
  - SRP : `UnitOwner` gere uniquement la relation lot-proprietaire, pas les paiements
  - LSP : Tout repository implementant `UnitOwnerRepository` est substituable
  - ISP : Traits separes pour `UnitRepository` et `UnitOwnerRepository`
- **Scenarios BDD** :

```gherkin
Scenario: Attribuer une quote-part valide a un coproprietaire
  Given l'immeuble "Residence du Parc" a le lot "3B" au 3eme etage
  And les quotes-parts actives totalisent 85.0%
  When Marc attribue a Sophie une quote-part de 4.5% sur le lot "3B"
  Then la quote-part de Sophie est enregistree (4.5%)
  And le total des quotes-parts actives est 89.5%

Scenario: Refuser une quote-part qui depasse 100% (INV-1)
  Given les quotes-parts actives du lot "3B" totalisent 92.0%
  When Marc tente d'ajouter un coproprietaire avec 15.0%
  Then une erreur de violation de contrainte est retournee (total 107% > 100%)
  And aucun coproprietaire n'est ajoute

Scenario: Refuser une quote-part de 0% ou negative (INV-6)
  When Marc tente de creer un coproprietaire avec une quote-part de 0%
  Then une erreur "Quote-part must be between 0% exclusive and 100% inclusive" est retournee

Scenario: Transferer la propriete d'un lot
  Given Sophie possede 4.5% du lot "3B" depuis le 01/01/2024
  When Marc enregistre le transfert de Sophie vers Pierre pour le lot "3B" a la date du 15/03/2026
  Then l'historique de Sophie est clos au 15/03/2026 (end_date)
  And Pierre est enregistre comme nouveau coproprietaire de "3B" avec 4.5% a partir du 15/03/2026
```

- **Dependances** : FR-001 (Building)
- **Endpoints** : `GET/POST /units`, `GET /buildings/:id/units`, `POST /units/:id/owners`, `PUT /unit-owners/:id`, `POST /units/:id/owners/transfer`, `GET /units/:id/owners/total-percentage`

---

### Module : General Assembly (brief section 9, bounded context 3)

#### FR-003 : Convocations legales avec delai 15 jours

- **Description** : Generer et envoyer les convocations d'AG avec respect automatique du delai legal belge (15 jours ordinaire, 8 jours extraordinaire). Calcul automatique de la date minimum d'envoi. Tracking email (ouverture, relances J-3). Support procurations et multi-langue (FR/NL/DE/EN).
- **Priorite** : MUST HAVE (Jalon 2)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux que KoproGo calcule automatiquement la date limite d'envoi des convocations pour l'AG du 20 mars, afin de ne jamais risquer l'annulation pour vice de forme. (Scenario typique brief section 6.1)
- **Entites DDD** : `Convocation`, `ConvocationRecipient` (brief section 9)
- **Invariants** (brief section 10, INV-3) :
  - Delai legal de convocation >= 15 jours (ordinaire et extraordinaire) — Art. 3.87 ss3 CC
  - `minimum_send_date` calcule automatiquement = `meeting_date - 15 jours`
  - Workflow : Draft -> Scheduled -> Sent -> Cancelled
- **Principes SOLID** :
  - SRP : `Convocation` gere l'invitation, pas le vote ni le quorum
  - OCP : Ajout de canaux (SMS, Push) sans modifier le Domain
  - DIP : `ConvocationRepository` + `ConvocationRecipientRepository` traits
- **Scenarios BDD** :

```gherkin
Scenario: Calcul automatique du delai legal pour AG ordinaire (INV-3)
  Given Marc prepare l'AG ordinaire de "Residence du Parc" prevue le 20/03/2026
  When Marc cree la convocation
  Then la date minimum d'envoi est calculee au 05/03/2026 (15 jours avant)
  And le statut de la convocation est "Draft"

Scenario: Refuser l'envoi d'une convocation hors delai legal (INV-3)
  Given une convocation pour l'AG du 20/03/2026 avec date minimum d'envoi le 05/03/2026
  When Marc tente d'envoyer la convocation le 12/03/2026 (8 jours avant, < 15 jours)
  Then une erreur "La convocation ne respecte pas le delai legal de 15 jours" est retournee
  And la convocation n'est pas envoyee

Scenario: Envoyer une convocation dans les delais avec tracking
  Given une convocation "Draft" pour l'AG du 20/03/2026
  And 30 coproprietaires sont enregistres pour "Residence du Parc"
  When Marc envoie la convocation le 01/03/2026
  Then 30 recipients sont crees avec email_sent_at = maintenant
  And le statut passe a "Sent"
  And le total_recipients est 30

Scenario: Relance automatique J-3 pour emails non ouverts
  Given la convocation pour l'AG du 20/03/2026 a ete envoyee le 01/03/2026
  And 12 coproprietaires sur 30 n'ont pas ouvert l'email
  When le job de relance J-3 s'execute le 17/03/2026
  Then 12 emails de rappel sont envoyes
  And reminder_sent_at est mis a jour pour chaque recipient concerne

Scenario: Delegation par procuration
  Given Sophie (brief section 6.2) est convoquee a l'AG du 20/03/2026
  When Sophie delegue sa procuration a Alice (coproprietaire presente)
  Then le proxy_owner_id de Sophie est mis a jour avec l'ID d'Alice
  And Alice dispose du pouvoir de vote de Sophie en plus du sien
```

- **Dependances** : FR-001 (Building), FR-018 (Identity & Access)
- **Endpoints** : `POST /convocations`, `PUT /convocations/:id/schedule`, `POST /convocations/:id/send`, `POST /convocations/:id/reminders`, `PUT /convocation-recipients/:id/proxy`

---

#### FR-004 : Vote numerique avec majorites legales belges

- **Description** : Systeme de vote numerique pour les resolutions d'AG, avec calcul automatique des 4 types de majorite belge (Art. 3.88 CC : absolue, 2/3, 4/5, unanimite). Support des tantiemes/dix-milliemes comme pouvoir de vote (0-10000), procurations (max 3 mandats), et cloture automatique avec resultat calcule. Abstentions exclues du calcul sauf pour l'unanimite.
- **Priorite** : MUST HAVE (Jalon 2-3)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux voter "Pour" la resolution de travaux de toiture lors de l'AG, avec mon pouvoir de vote de 45 milliemes, et voir le resultat en temps reel. (Scenario typique brief section 6.2)
- **Entites DDD** : `Resolution`, `Vote` (brief section 9)
- **Invariants** (brief section 10) :
  - INV-2 : Quorum AG >= 50% des quotes-parts (physiques + procurations) — Art. 3.87 ss5 CC
  - INV-4 : Majorite de vote respectee selon le type de resolution (Art. 3.88 CC) :
    - Absolute : >50% des presents/representes (abstentions exclues du denominateur)
    - TwoThirds : >=2/3 des presents/representes (abstentions exclues)
    - FourFifths : >=4/5 des presents/representes (abstentions exclues)
    - Unanimity : 100% de TOUS les tantiemes (y compris absents, abstentions = votes contre)
  - Voting power : 0 a 10000 dix-milliemes par lot
  - Etats resolution : Pending -> Adopted/Rejected (calcul automatique)
- **Principes SOLID** :
  - SRP : `Resolution` gere la proposition, `Vote` gere le suffrage individuel
  - OCP : Ajout de nouveaux types de majorite sans modifier le moteur de calcul existant
  - LSP : `ResolutionRepository` et `VoteRepository` substituables
- **Scenarios BDD** :

```gherkin
Scenario: Verifier le quorum avant ouverture des votes (INV-2)
  Given l'AG de "Residence du Parc Royal" a 10000 dix-milliemes au total
  And les coproprietaires presents et representes totalisent 4500 dix-milliemes
  When le syndic tente d'ouvrir les votes
  Then une erreur "Quorum non atteint : 45% < 50% requis" est retournee
  And aucun vote ne peut etre enregistre

Scenario: Voter avec majorite absolue — resolution adoptee (INV-4, Art. 3.88 §1)
  Given l'AG a le quorum valide (5200 dix-milliemes sur 10000)
  And la resolution "Changer de fournisseur d'electricite" requiert une majorite Absolute
  When Alice vote "Pour" avec 800 dix-milliemes (dont 300 par procuration de Pierre)
  And Sophie vote "Pour" avec 450 dix-milliemes
  And Jean vote "Contre" avec 600 dix-milliemes
  And le syndic cloture le scrutin
  Then les votes "Pour" totalisent 1250 dix-milliemes (67.6% des exprimes, abstentions exclues)
  And les votes "Contre" totalisent 600 dix-milliemes (32.4% des exprimes)
  And la resolution est "Adopted" (1250 > 50% de 1850 exprimes)

Scenario: Voter avec majorite 2/3 — resolution rejetee (INV-4, Art. 3.88 §1, 1°)
  Given l'AG a le quorum valide
  And la resolution "Travaux structurels de toiture pour 45 000 EUR" requiert une majorite TwoThirds
  When les votes "Pour" totalisent 3000 dix-milliemes sur 5000 exprimes (60%)
  And le syndic cloture le scrutin
  Then la resolution est "Rejected" (60% < 66.67% requis)
  And le PV mentionne le resultat detaille

Scenario: Voter avec unanimite — inclusion de TOUS les tantiemes (INV-4, Art. 3.88 §1, 3°)
  Given l'AG a 10000 dix-milliemes au total, quorum valide avec 6000 dix-milliemes presents
  And la resolution "Dissolution de la copropriete" requiert une majorite Unanimity
  When les votes "Pour" totalisent 6000 dix-milliemes (100% des presents)
  And le syndic cloture le scrutin
  Then la resolution est "Rejected" (6000 < 10000 = 100% de TOUS les tantiemes, y compris absents)

Scenario: Vote par procuration
  Given Sophie a donne procuration a Alice pour l'AG
  When Alice vote "Pour" la resolution au nom de Sophie
  Then le vote est enregistre avec proxy_owner_id = Sophie
  And le voting_power utilise est celui des milliemes de Sophie (45)

Scenario: Modifier un vote avant cloture du scrutin
  Given Sophie a vote "Contre" la resolution
  When Sophie change son vote en "Pour" avant la cloture
  Then l'ancien vote est mis a jour (pas de nouveau vote cree)
  And l'audit trail enregistre le changement (VoteChanged)
```

- **Dependances** : FR-003 (Convocation), FR-002 (UnitOwner pour milliemes)
- **Endpoints** : `POST /meetings/:id/resolutions`, `POST /resolutions/:id/vote`, `PUT /votes/:id`, `PUT /resolutions/:id/close`, `GET /meetings/:id/vote-summary`

---

#### FR-005 : AGE (Assemblee Generale Extraordinaire) par petition

- **Description** : Permettre aux coproprietaires de demander une AGE par petition lorsque 1/5 (20%) des quotes-parts est atteint. Workflow de cosignatures avec verification automatique du seuil, soumission au syndic avec delai de reponse de 15 jours.
- **Priorite** : SHOULD HAVE (Jalon 3)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux lancer une petition pour une AGE sur les travaux urgents de toiture, et KoproGo me dit automatiquement quand le seuil de 1/5 est atteint.
- **Entites DDD** : `AgeRequest`, `AgeRequestCosignatory` (brief section 9)
- **Invariants** (brief section 10, INV-8) :
  - Seuil AGE = 1/5 des quotes-parts (20%) — Art. 3.87 ss2 CC
  - Verification automatique a chaque cosignature
  - Delai syndic : 15 jours apres soumission
- **Principes SOLID** : SRP, OCP (nouveaux statuts sans modifier le Domain)
- **Scenarios BDD** :

```gherkin
Scenario: Seuil AGE atteint automatiquement (INV-8)
  Given l'immeuble a 10000 dix-milliemes au total (seuil 20% = 2000 dix-milliemes)
  And Sophie (450 dix-mill.) a cree une demande d'AGE "Travaux urgents toiture"
  And Pierre (800 dix-mill.) et Alice (550 dix-mill.) ont cosigne (total = 1800 dix-mill.)
  When Jean (600 dix-mill.) cosigne la demande
  Then le total atteint 2400 dix-milliemes (24% > 20%)
  And le statut passe automatiquement de "Open" a "Reached"
  And Sophie peut soumettre la demande au syndic

Scenario: Refuser la soumission d'une AGE sous le seuil (INV-8)
  Given la demande d'AGE totalise 1500 dix-milliemes (15% < 20%)
  When Sophie tente de soumettre la demande au syndic
  Then une erreur "Seuil non atteint : 15% < 20% requis" est retournee

Scenario: Delai de reponse du syndic (15 jours)
  Given la demande d'AGE est soumise au syndic le 01/03/2026
  Then le syndic_deadline_at est calcule au 16/03/2026
  And si Marc n'a pas repondu avant le 16/03/2026, une convocation automatique est declenchee
```

- **Dependances** : FR-002 (UnitOwner pour quotes-parts), FR-003 (Convocation pour auto-convocation)
- **Endpoints** : `POST /buildings/:id/age-requests`, `POST /age-requests/:id/cosign`, `POST /age-requests/:id/submit`, `POST /age-requests/:id/accept`, `POST /age-requests/:id/reject`

---

### Module : Accounting (brief section 9, bounded context 4)

#### FR-006 : Comptabilite PCMN belge

- **Description** : Plan Comptable Minimum Normalise belge (AR 12/07/2012) avec ~90 comptes pre-seedes (8 classes), ecritures journal double-entree (4 types : ACH, VEN, FIN, ODS), bilan et compte de resultats, variance budgetaire.
- **Priorite** : MUST HAVE (Jalon 2, acheve)
- **User Story** : En tant que Jean-Pierre (comptable, brief section 6.3), je veux saisir une ecriture journal double-entree conforme au PCMN belge, avec verification automatique de l'equilibre debit/credit. (Scenario typique brief section 6.3)
- **Entites DDD** : `Account`, `JournalEntry`, `JournalEntryLine`, `Budget` (brief section 9)
- **Invariants** (brief section 10, INV-5) :
  - Comptabilite double-entree equilibree (total debits = total credits par ecriture)
  - ~90 comptes PCMN pre-seedes (classes 1-8)
  - Validation codes comptables et hierarchie (classes, sous-classes, groupes, comptes)
- **Principes SOLID** :
  - SRP : `Account` gere le plan comptable, `JournalEntry` gere les ecritures
  - OCP : Ajout de nouveaux types de journaux sans modifier le Domain
  - DIP : `AccountRepository`, `JournalEntryRepository` traits
- **Scenarios BDD** :

```gherkin
Scenario: Saisir une ecriture journal equilibree
  Given Jean-Pierre est connecte en tant que comptable
  And le plan comptable PCMN est seede pour "Residence du Parc"
  When Jean-Pierre saisit une ecriture journal ACH :
    | Compte | Debit    | Credit   | Description           |
    | 6000   | 1000.00  | 0.00     | Charges d'entretien   |
    | 4400   | 0.00     | 1000.00  | Fournisseur a payer   |
  Then l'ecriture est creee avec succes
  And le total debits (1000.00) = total credits (1000.00)

Scenario: Refuser une ecriture journal desequilibree (INV-5)
  When Jean-Pierre saisit une ecriture avec 1000.00 EUR au debit du compte 6000 sans contrepartie au credit
  Then une erreur "Ecriture desequilibree : debits (1000.00) != credits (0.00)" est retournee
  And aucune ecriture n'est creee

Scenario: Generer le bilan d'une copropriete
  Given la copropriete "Residence du Parc" a des ecritures sur l'exercice 2025
  When Jean-Pierre genere le bilan au 31/12/2025
  Then le rapport affiche l'Actif (classes 2-3-4), le Passif (classes 1-4), et le Resultat
  And Actif = Passif + Resultat (equilibre du bilan)

Scenario: Seeder le PCMN belge complet
  When l'administrateur execute le seed PCMN pour une nouvelle organisation
  Then environ 90 comptes sont crees avec leur hierarchie (classe, sous-classe, groupe, compte)
  And les 8 classes comptables belges sont presentes (1:Actif a 8:Hors-bilan)
```

- **Dependances** : FR-001 (Building), FR-018 (Identity & Access / Organization)
- **Endpoints** : `GET/POST /accounts`, `POST /accounts/seed/belgian-pcmn`, `GET/POST /journal-entries`, `GET /reports/balance-sheet`, `GET /reports/income-statement`

---

#### FR-007 : Budget annuel et variance

- **Description** : Gestion du budget annuel par copropriete avec workflow d'approbation (Draft -> Submitted -> Approved -> Archived), suivi des depenses reelles, et analyse de variance (budget vs reel).
- **Priorite** : MUST HAVE (Jalon 2, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux soumettre le budget 2026 de "Residence du Parc" pour approbation en AG, puis suivre les ecarts entre budget et depenses reelles.
- **Entite DDD** : `Budget` (brief section 9)
- **Principes SOLID** : SRP (Budget ne gere que l'enveloppe budgetaire), OCP (ajout de nouveaux statuts sans modifier le Domain)
- **Scenarios BDD** :

```gherkin
Scenario: Approuver un budget en AG
  Given Marc a cree le budget 2026 pour "Residence du Parc" (120 000 EUR, statut "Draft")
  When Marc soumet le budget (Draft -> Submitted)
  And l'AG approuve le budget avec meeting_id de l'AG du 20/03/2026
  Then le budget passe en statut "Approved"
  And le meeting_id est enregistre pour tracabilite

Scenario: Analyser la variance budgetaire
  Given le budget 2026 approuve est de 120 000 EUR
  And les depenses reelles au 30/06/2026 sont de 75 000 EUR
  When Jean-Pierre (brief section 6.3) consulte la variance
  Then la variance affiche : budget 120 000 EUR, reel 75 000 EUR, ecart -45 000 EUR (37.5% sous-budget)
```

- **Dependances** : FR-006 (Accounting), FR-004 (AG pour approbation)
- **Endpoints** : `POST /budgets`, `PUT /budgets/:id/submit`, `PUT /budgets/:id/approve`, `GET /budgets/:id/variance`

---

#### FR-008 : Etat date pour ventes immobilieres

- **Description** : Generation du document legal "etat date" obligatoire lors de la vente d'un lot (Art. 577-11 ss2 CC), resumant la situation financiere du coproprietaire vendeur. Delai legal : 10 jours ouvrables apres demande. Validite : 3 mois.
- **Priorite** : MUST HAVE (Jalon 2, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux generer l'etat date du lot 12A pour la vente par Sophie, avec le solde debiteur et les charges en cours, dans le delai legal de 10 jours.
- **Entite DDD** : `EtatDate` (brief section 9)
- **Scenarios BDD** :

```gherkin
Scenario: Generer un etat date dans les delais
  Given Sophie vend le lot 12A et le notaire demande l'etat date le 01/03/2026
  When Marc genere l'etat date avec les donnees financieres (solde 1 250 EUR, charges 2026 payees)
  Then le document PDF est genere avec le numero de reference unique
  And le statut passe a "Generated"
  And la date de reference est le 01/03/2026

Scenario: Alerter sur les etats dates en retard (>10 jours)
  Given un etat date a ete demande le 01/03/2026 et n'est pas encore genere le 15/03/2026
  When le systeme verifie les retards
  Then l'etat date apparait dans la liste "overdue" (14 jours > 10 jours legaux)

Scenario: Detecter les etats dates expires (>3 mois)
  Given un etat date a ete genere le 01/01/2026
  When le systeme verifie les expirations le 15/04/2026
  Then l'etat date apparait dans la liste "expired" (>3 mois depuis la date de reference)
```

- **Dependances** : FR-001 (Building), FR-002 (Unit/UnitOwner)
- **Endpoints** : `POST /etats-dates`, `GET /etats-dates/overdue`, `GET /etats-dates/expired`, `PUT /etats-dates/:id/mark-generated`, `PUT /etats-dates/:id/mark-delivered`

---

### Module : Billing & Payments (brief section 9, bounded context 5)

#### FR-009 : Facturation avec TVA belge

- **Description** : Factures multi-lignes avec taux TVA belges (6% reduit pour renovations, 12% intermediaire, 21% standard), workflow d'approbation (Draft -> PendingApproval -> Approved/Rejected), et distribution automatique des charges selon les quotes-parts.
- **Priorite** : MUST HAVE (Jalon 2, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux creer une facture de travaux de renovation avec TVA 6% et la soumettre au conseil de copropriete pour approbation.
- **Entites DDD** : `Expense`, `InvoiceLineItem`, `ChargeDistribution` (brief section 9)
- **Principes SOLID** :
  - SRP : `Expense` = workflow facture, `InvoiceLineItem` = lignes detaillees, `ChargeDistribution` = repartition
  - OCP : Ajout de nouveaux taux TVA sans modifier le moteur de calcul
- **Scenarios BDD** :

```gherkin
Scenario: Creer une facture multi-lignes avec TVA belge
  Given Marc cree une facture pour "Residence du Parc"
  When il ajoute les lignes suivantes :
    | Description        | Quantite | Prix unitaire | TVA  |
    | Renovation hall    | 1        | 5 000 EUR     | 6%   |
    | Remplacement porte | 1        | 2 000 EUR     | 21%  |
  Then le total HTVA est 7 000 EUR
  And le total TVA est 720 EUR (300 + 420)
  And le total TTC est 7 720 EUR

Scenario: Distribuer les charges selon les quotes-parts
  Given la facture de 3 000 EUR est approuvee pour "Residence du Parc"
  And le lot 3B de Sophie (brief section 6.2) a 4.5% de quotes-parts
  When la distribution des charges est calculee
  Then Sophie doit payer 135 EUR (3 000 x 4.5%)
  And la somme de toutes les distributions = 3 000 EUR

Scenario: Refuser la modification d'une facture approuvee
  Given la facture est en statut "Approved"
  When Marc tente de modifier le montant
  Then une erreur "Modification interdite apres approbation" est retournee
```

- **Dependances** : FR-002 (UnitOwner pour quotes-parts), FR-006 (Accounting)
- **Endpoints** : `GET/POST /expenses`, `PUT /expenses/:id/submit-for-approval`, `PUT /expenses/:id/approve`, `POST /invoices/:id/calculate-distribution`

---

#### FR-010 : Paiements Stripe + SEPA

- **Description** : Integration paiements avec Stripe Payment Intents et SEPA Direct Debit. Lifecycle complet : Pending -> Processing -> RequiresAction -> Succeeded/Failed/Cancelled/Refunded. Idempotency keys pour prevenir les doubles charges. Support remboursements partiels/complets.
- **Priorite** : MUST HAVE (Jalon 2-3, acheve)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux payer mes charges trimestrielles par SEPA depuis mon compte bancaire belge, avec la certitude qu'un retry reseau ne creera pas de double charge.
- **Entites DDD** : `Payment`, `PaymentMethod` (brief section 9)
- **Invariants** (brief section 10, INV-7) :
  - Idempotency des paiements — cle unique >= 16 caracteres
  - Pas de stockage de donnees carte raw (PCI-DSS : uniquement Stripe tokens)
  - Anti-over-refund : remboursement <= montant paye
- **Principes SOLID** :
  - SRP : `Payment` gere la transaction, `PaymentMethod` gere le moyen de paiement
  - DIP : Abstraction Payment adapter (hexagonal) pour Stripe, avec fallback possible
- **Scenarios BDD** :

```gherkin
Scenario: Paiement SEPA reussi avec idempotency (INV-7)
  Given Sophie a un mandat SEPA actif (BE68 5390 0754 7034)
  And la facture trimestrielle est de 450 EUR
  When Sophie initie le paiement avec idempotency_key "pay_2026q1_sophie_3b"
  Then le paiement est cree en statut "Pending"
  And Stripe retourne payment_intent_id
  And apres confirmation, le statut passe a "Succeeded"

Scenario: Prevenir la double charge par idempotency (INV-7)
  Given un paiement avec idempotency_key "pay_2026q1_sophie_3b" existe deja en statut "Succeeded"
  When un retry reseau envoie le meme paiement avec la meme idempotency_key
  Then le paiement existant est retourne (pas de nouveau paiement cree)
  And aucune double charge n'est effectuee

Scenario: Remboursement partiel
  Given un paiement de 450 EUR en statut "Succeeded"
  When Marc effectue un remboursement partiel de 150 EUR
  Then le refunded_amount_cents passe a 15000
  And le montant net est 300 EUR (450 - 150)

Scenario: Refuser un remboursement superieur au montant paye
  Given un paiement de 450 EUR avec 150 EUR deja rembourses
  When Marc tente de rembourser 400 EUR
  Then une erreur "Remboursement impossible : 400 > 300 EUR restants" est retournee
```

- **Dependances** : FR-009 (Expense), FR-002 (Owner)
- **Endpoints** : `POST /payments`, `PUT /payments/:id/succeeded`, `POST /payments/:id/refund`, `POST /payment-methods`, `PUT /payment-methods/:id/set-default`

---

#### FR-011 : Recouvrement automatise 4 niveaux

- **Description** : Workflow d'escalade automatique pour les impayes : Gentle (J+15), Formal (J+30), FinalNotice (J+45), LegalAction (J+60). Calcul automatique des penalites au taux legal belge (8% annuel). Tracabilite complete (sent_date, tracking_number, notes).
- **Priorite** : MUST HAVE (Jalon 3, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux que KoproGo envoie automatiquement des relances d'impayes avec escalade progressive, pour atteindre un taux de recouvrement > 85% avant action legale. (Objectif O7)
- **Entite DDD** : `PaymentReminder` (brief section 9)
- **Scenarios BDD** :

```gherkin
Scenario: Escalade automatique d'un impaye
  Given la facture de Sophie (135 EUR) est impayee depuis 15 jours
  When le systeme genere la relance de niveau 1
  Then une relance "Gentle" est creee avec ton courtois
  And la penalite de retard est calculee : 135 x 8% / 365 x 15 = 0.44 EUR

Scenario: Escalade jusqu'au niveau LegalAction
  Given Sophie n'a pas paye malgre 3 relances (Gentle, Formal, FinalNotice)
  And 60 jours se sont ecoules depuis l'echeance
  When le systeme escalade au niveau 4
  Then une relance "LegalAction" est creee
  And la penalite totale est calculee : 135 x 8% / 365 x 60 = 1.78 EUR
  And le tracking_number est enregistre pour le recommande

Scenario: Statistiques de recouvrement
  Given 50 factures impayees ce trimestre pour "Residence du Parc"
  When Marc consulte les statistiques de recouvrement
  Then le dashboard affiche : 35 recouvrees avant Gentle, 10 au niveau Formal, 3 au FinalNotice, 2 en LegalAction
  And le taux de recouvrement avant LegalAction est 96% (48/50)
```

- **Dependances** : FR-009 (Expense), FR-010 (Payment)
- **Endpoints** : `GET/POST /payment-reminders`, `PUT /payment-reminders/:id/escalate`, `GET /payment-reminders/stats`

---

#### FR-012 : Appels de fonds et contributions

- **Description** : Creation d'appels de fonds collectifs distribues automatiquement selon les quotes-parts, avec suivi des paiements individuels par coproprietaire.
- **Priorite** : MUST HAVE (Jalon 2, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux lancer l'appel de fonds trimestriel de 25 000 EUR pour "Residence du Parc", avec distribution automatique selon les milliemes de chaque coproprietaire. (Scenario typique brief section 6.1)
- **Entites DDD** : `CallForFunds`, `OwnerContribution` (brief section 9)
- **Scenarios BDD** :

```gherkin
Scenario: Creer et envoyer un appel de fonds
  Given Marc cree l'appel de fonds Q1 2026 de 25 000 EUR pour "Residence du Parc"
  When Marc envoie l'appel de fonds
  Then une contribution individuelle est creee pour chaque coproprietaire actif
  And Sophie (4.5%) recoit une contribution de 1 125 EUR (25 000 x 4.5%)
  And la somme de toutes les contributions = 25 000 EUR

Scenario: Suivi des paiements sur contributions
  Given Sophie a une contribution de 1 125 EUR en statut "Pending"
  When Sophie paie sa contribution
  Then le statut passe a "Paid" avec payment_date = aujourd'hui
  And le dashboard de Marc affiche le taux de paiement global
```

- **Dependances** : FR-002 (UnitOwner pour quotes-parts), FR-010 (Payment)
- **Endpoints** : `POST /call-for-funds`, `POST /call-for-funds/:id/send`, `PUT /owner-contributions/:id/mark-paid`

---

### Module : Maintenance (brief section 9, bounded context 6)

#### FR-013 : Ticketing maintenance avec SLA

- **Description** : Gestion des demandes de maintenance avec 6 etats (Open -> Assigned -> InProgress -> Resolved -> Closed/Cancelled), 5 niveaux de priorite avec SLA automatiques (Critical: 1h, Urgent: 4h, High: 24h, Medium: 3j, Low: 7j), assignation a des prestataires.
- **Priorite** : SHOULD HAVE (Jalon 3, acheve)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux signaler une fuite dans le parking via KoproGo, et suivre l'avancement de la reparation en temps reel. (Scenario typique brief section 6.2)
- **Entite DDD** : `Ticket` (brief section 9)
- **Principes SOLID** :
  - SRP : `Ticket` gere le workflow de maintenance, pas les devis ni les rapports
  - OCP : Ajout de nouvelles categories (Elevator, Roof) sans modifier le Domain
- **Scenarios BDD** :

```gherkin
Scenario: Creer un ticket maintenance avec SLA automatique
  Given Sophie est connectee en tant que coproprietaire
  When Sophie cree un ticket "Fuite d'eau parking P2" avec priorite "High"
  Then le ticket est cree en statut "Open"
  And la due_date est calculee automatiquement a maintenant + 24h (SLA High)
  And Marc (syndic) recoit une notification

Scenario: Workflow complet d'un ticket (multi-role)
  Given Sophie a cree le ticket "Fuite d'eau parking P2"
  When Marc assigne le ticket a Ahmed (plombier, brief section 6.4)
  Then le statut passe a "Assigned"
  When Ahmed commence l'intervention
  Then le statut passe a "InProgress"
  When Ahmed marque le ticket comme resolu
  Then le statut passe a "Resolved"
  When Marc valide et ferme le ticket
  Then le statut passe a "Closed"

Scenario: Detecter les tickets en retard
  Given le ticket "Fuite parking" a une due_date depassee de 2h
  When le systeme verifie les retards
  Then le ticket apparait dans la liste "overdue"
  And une alerte est envoyee a Marc
```

- **Dependances** : FR-001 (Building), FR-018 (Identity & Access pour assignation)
- **Endpoints** : `POST /tickets`, `PUT /tickets/:id/assign`, `PUT /tickets/:id/start`, `PUT /tickets/:id/resolve`, `PUT /tickets/:id/close`, `GET /tickets/overdue`

---

#### FR-014 : Devis multi-entrepreneurs avec scoring belge

- **Description** : Comparaison de devis entrepreneurs avec scoring automatique (prix 40%, delai 30%, garantie 20%, reputation 10%). Conformite belge : 3 devis obligatoires pour travaux > 5 000 EUR. Support des taux TVA belges et des garanties decennales. Rapports de travaux via magic link PWA pour prestataires.
- **Priorite** : SHOULD HAVE (Jalon 3, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux comparer 3 devis pour les travaux de toiture (45 000 EUR) et obtenir un classement objectif base sur le scoring belge.
- **Entite DDD** : `Quote`, `ContractorReport` (brief section 9)
- **Invariants** (brief section 10, INV-10) :
  - 3 devis minimum pour travaux > 5 000 EUR
  - Taux TVA belges : 6% (renovation), 21% (neuf)
  - Garantie decennale : 10 ans (structurel), 2 ans (apparent)
- **Principes SOLID** :
  - SRP : `Quote` gere le devis, `ContractorReport` gere le rapport d'intervention
  - OCP : Ajout de nouveaux criteres de scoring sans modifier l'algorithme de base
- **Scenarios BDD** :

```gherkin
Scenario: Comparer 3 devis avec scoring automatique (INV-10)
  Given Marc a recu 3 devis pour "Travaux toiture" :
    | Entrepreneur | Prix HTVA  | Delai  | Garantie | Reputation |
    | Toitures SA  | 42 000 EUR | 30j    | 10 ans   | 85/100     |
    | BelgiBat     | 45 000 EUR | 20j    | 10 ans   | 90/100     |
    | RoofPro      | 38 000 EUR | 45j    | 5 ans    | 70/100     |
  When Marc lance la comparaison
  Then le scoring est calcule avec poids : prix 40%, delai 30%, garantie 20%, reputation 10%
  And BelgiBat est classe premier avec le score le plus eleve

Scenario: Avertissement avec moins de 3 devis pour travaux > 5 000 EUR (INV-10)
  Given Marc a recu seulement 2 devis pour "Travaux toiture" a 45 000 EUR
  When Marc tente d'accepter un devis
  Then un avertissement "La loi belge requiert 3 devis minimum pour travaux > 5 000 EUR" est affiche

Scenario: Rapport de travaux via magic link prestataire
  Given Ahmed (brief section 6.4) a termine l'intervention sur le ticket "Fuite parking"
  When Marc genere un magic link pour Ahmed (validite 72h)
  Then Ahmed recoit un email avec le lien
  And Ahmed peut remplir le rapport (photos avant/apres, pieces remplacees, compte-rendu) sans creer de compte
  And apres soumission, le conseil de copropriete valide ou demande des corrections
```

- **Dependances** : FR-013 (Ticket), FR-009 (Expense pour facturation)
- **Endpoints** : `POST /quotes`, `POST /quotes/:id/submit`, `POST /quotes/compare`, `POST /contractor-reports/:id/generate-magic-link`, `GET /contractor-reports/magic/:token`

---

### Module : GDPR & Compliance (brief section 9, bounded context 8)

#### FR-015 : Conformite GDPR complete (Articles 15-21 + 30)

- **Description** : Implementation des 6 articles GDPR : droit d'acces (Art. 15), rectification (Art. 16), effacement/anonymisation (Art. 17), restriction de traitement (Art. 18), opposition marketing (Art. 21), registre des activites de traitement (Art. 30). Audit trail complet avec IP et user-agent.
- **Priorite** : MUST HAVE (Jalon 1, acheve)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux pouvoir exporter toutes mes donnees personnelles (Art. 15), demander la correction de mon email (Art. 16), ou exercer mon droit a l'oubli (Art. 17).
- **Entites DDD** : Operations sur `User` (export, erasure, rectify, restrict, marketing opt-out) (brief section 9)
- **Principes SOLID** :
  - SRP : Chaque article GDPR = une methode use case distincte
  - OCP : Ajout de nouveaux droits GDPR sans modifier les handlers existants
  - DIP : `GdprRepository` + `UserRepository` traits
- **Scenarios BDD** :

```gherkin
Scenario: Article 15 — Droit d'acces (export donnees personnelles)
  Given Sophie est connectee en tant que coproprietaire
  When Sophie demande l'export de ses donnees personnelles
  Then un fichier JSON est genere contenant :
    | Categorie | Donnees |
    | Identite  | nom, prenom, email, telephone |
    | Propriete | lots, quotes-parts, historique |
    | Finances  | paiements, contributions, relances |
    | Votes     | historique des votes (anonymise si anonyme) |
  And l'audit trail enregistre "GdprDataExported" avec IP et user-agent

Scenario: Article 16 — Droit de rectification
  Given Sophie a l'email "sophei@mail.com" (typo)
  When Sophie demande la rectification avec email "sophie@mail.com"
  Then l'email est corrige dans la base de donnees
  And l'audit trail enregistre "GdprDataRectified"

Scenario: Article 17 — Droit a l'effacement avec verification legale
  Given Sophie demande la suppression de son compte
  When le systeme verifie l'eligibilite a l'effacement
  And Sophie n'a aucune obligation legale en cours (pas de litige, pas de dette)
  Then les donnees personnelles sont anonymisees (nom -> "Anonyme_xxxxx", email -> null)
  And les donnees financieres sont conservees (obligation legale 10 ans)

Scenario: Article 17 — Refus d'effacement pour obligation legale
  Given Sophie a un litige en cours avec la copropriete
  When Sophie demande la suppression de son compte
  Then la demande est refusee : "Effacement impossible : obligation legale en cours"
  And l'audit trail enregistre "GdprErasureRefused"

Scenario: Article 18 — Restriction de traitement
  Given Sophie est en litige et demande la restriction de traitement
  When le systeme applique la restriction
  Then processing_restricted = true et processing_restricted_at = maintenant
  And les donnees de Sophie ne sont plus utilisees pour les traitements automatises
  And les relances automatiques sont suspendues pour Sophie

Scenario: Article 21 — Opposition au marketing
  Given Sophie ne souhaite plus recevoir de communications marketing
  When Sophie active l'opt-out marketing
  Then marketing_opt_out = true et marketing_opt_out_at = maintenant
  And Sophie ne recoit plus que les notifications legalement obligatoires (convocations AG, appels de fonds)
```

- **Dependances** : FR-018 (Identity & Access / User)
- **Endpoints** : `GET /gdpr/export`, `DELETE /gdpr/erase`, `GET /gdpr/can-erase`, `PUT /gdpr/rectify`, `PUT /gdpr/restrict-processing`, `PUT /gdpr/marketing-preference`

---

### Module : Community (brief section 9, bounded context 9)

#### FR-016 : SEL — Systeme d'Echange Local

- **Description** : Systeme d'echange local a base de monnaie temps (1 heure = 1 credit). 3 types d'echanges : Service, ObjectLoan, SharedPurchase. Workflow 5 etats (Offered -> Requested -> InProgress -> Completed -> Cancelled). Systeme de notation mutuelle (1-5 etoiles). Leaderboard et statistiques communautaires.
- **Priorite** : SHOULD HAVE (Jalon 3, acheve)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux offrir 2h de babysitting sur le SEL de mon immeuble, et quand un voisin complete l'echange, mes credits sont mis a jour automatiquement.
- **Entites DDD** : `LocalExchange`, `OwnerCreditBalance` (brief section 9)
- **Principes SOLID** :
  - SRP : `LocalExchange` gere l'echange, `OwnerCreditBalance` gere la monnaie
  - OCP : Ajout de nouveaux types d'echange sans modifier le Domain
- **Scenarios BDD** :

```gherkin
Scenario: Offrir un service et completer l'echange
  Given Sophie offre "Babysitting 2h" (2 credits) sur le SEL de "Residence du Parc"
  And le statut est "Offered"
  When Pierre demande l'echange
  Then le statut passe a "Requested" et requester_id = Pierre
  When Sophie accepte et demarre l'echange
  Then le statut passe a "InProgress"
  When Sophie marque l'echange comme complete
  Then le statut passe a "Completed"
  And le solde de Sophie augmente de 2 credits (provider)
  And le solde de Pierre diminue de 2 credits (requester)

Scenario: Notation mutuelle apres echange
  Given l'echange entre Sophie (provider) et Pierre (requester) est "Completed"
  When Pierre note Sophie 5/5 etoiles
  And Sophie note Pierre 4/5 etoiles
  Then la note moyenne de Sophie est mise a jour
  And la note moyenne de Pierre est mise a jour

Scenario: Leaderboard communautaire
  Given 10 coproprietaires participent au SEL de "Residence du Parc"
  When Sophie consulte le leaderboard
  Then les 10 participants sont classes par solde de credits decroissant
  And le participant avec le plus de credits est en tete
```

- **Dependances** : FR-001 (Building), FR-002 (Owner)
- **Endpoints** : `POST /exchanges`, `POST /exchanges/:id/request`, `POST /exchanges/:id/complete`, `PUT /exchanges/:id/rate-provider`, `GET /buildings/:id/leaderboard`, `GET /buildings/:id/sel-statistics`

---

#### FR-017 : Sondages (Polls) entre assemblees

- **Description** : Systeme de sondages pour consultations rapides entre AG (Art. 577-8/4 ss4 Code Civil). 4 types : YesNo, MultipleChoice, Rating, OpenEnded. Support vote anonyme. Prevention doublons (UNIQUE constraint). Resultats avec taux de participation.
- **Priorite** : SHOULD HAVE (Jalon 3, acheve)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux lancer un sondage "Faut-il repeindre le hall ?" aupres des coproprietaires, et voir les resultats avec le taux de participation.
- **Entites DDD** : `Poll`, `PollOption`, `PollVote` (brief section 9)
- **Scenarios BDD** :

```gherkin
Scenario: Creer et publier un sondage YesNo
  Given Marc cree le sondage "Repeindre le hall en bleu ?" de type YesNo
  When Marc publie le sondage (Draft -> Active)
  Then le sondage est visible par tous les coproprietaires de "Residence du Parc"
  And les options "Oui" et "Non" sont creees automatiquement

Scenario: Voter et calculer les resultats
  Given le sondage "Repeindre le hall ?" est actif avec 30 coproprietaires eligibles
  When 20 coproprietaires votent (15 "Oui", 5 "Non")
  And Marc cloture le sondage
  Then le resultat est : "Oui" 75%, "Non" 25%
  And le taux de participation est 66.7% (20/30)
  And le gagnant est "Oui"

Scenario: Prevenir le double vote
  Given Sophie a deja vote "Oui" au sondage
  When Sophie tente de voter a nouveau
  Then une erreur "Vous avez deja vote sur ce sondage" est retournee

Scenario: Vote anonyme
  Given le sondage est configure avec is_anonymous = true
  When Sophie vote "Oui"
  Then le vote est enregistre sans owner_id (anonymisation)
  And seule l'adresse IP est conservee pour l'audit
```

- **Dependances** : FR-001 (Building), FR-002 (UnitOwner pour eligibilite)
- **Endpoints** : `POST /polls`, `PUT /polls/:id/publish`, `POST /polls/:id/vote`, `PUT /polls/:id/close`, `GET /polls/:id/results`

---

### Module : Identity & Access (brief section 9, bounded context 2)

#### FR-018 : Authentification multi-role avec 2FA

- **Description** : Systeme d'authentification JWT avec support multi-role (SuperAdmin, Syndic, Owner, Comptable), switch de role actif, 2FA TOTP avec backup codes, refresh token rotation, rate limiting (5 tentatives/15 min).
- **Priorite** : MUST HAVE (Jalon 1, acheve)
- **User Story** : En tant que Marc (syndic qui est aussi coproprietaire, brief section 6.1), je veux basculer entre mon role "syndic" et mon role "coproprietaire" sans me reconnecter, et securiser mon compte avec 2FA.
- **Entites DDD** : `User`, `UserRoleAssignment`, `RefreshToken`, `TwoFactorSecret` (brief section 9)
- **Principes SOLID** :
  - SRP : `User` gere l'identite, `UserRoleAssignment` gere les roles, `TwoFactorSecret` gere le 2FA
  - DIP : `UserRepository`, `UserRoleRepository` traits
- **Scenarios BDD** :

```gherkin
Scenario: Login avec switch de role
  Given Marc a les roles "syndic" et "coproprietaire"
  When Marc se connecte avec ses identifiants
  Then le JWT contient le role actif "syndic" (role principal)
  When Marc switch vers le role "coproprietaire"
  Then un nouveau JWT est emis avec le role actif "coproprietaire"
  And Marc voit le dashboard coproprietaire

Scenario: Activation 2FA TOTP
  Given Marc est connecte sans 2FA
  When Marc active le 2FA
  Then un QR code et 8 backup codes sont generes
  When Marc scanne le QR code et saisit le code TOTP
  Then le 2FA est active avec succes

Scenario: Rate limiting sur le login
  Given 5 tentatives de connexion echouees pour l'IP 192.168.1.1 en 15 minutes
  When une 6eme tentative est effectuee
  Then la tentative est rejetee : "Trop de tentatives, reessayez dans 15 minutes"
```

- **Dependances** : Aucune (module fondation)
- **Endpoints** : `POST /auth/login`, `POST /auth/switch-role`, `GET /auth/me`, `POST /2fa/setup`, `POST /2fa/enable`, `POST /2fa/verify`

---

### Module : Notifications (brief section 9, bounded context 7)

#### FR-019 : Notifications multi-canal

- **Description** : Systeme de notifications sur 4 canaux (Email, SMS, Push, In-App) avec 22 types d'evenements, preferences utilisateur granulaires par type et canal, tracking de livraison et lecture.
- **Priorite** : SHOULD HAVE (Jalon 3, acheve)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux recevoir les convocations AG par email et les alertes maintenance urgentes par SMS, mais ne pas etre derangee par push pour les annonces communautaires.
- **Entites DDD** : `Notification`, `NotificationPreference` (brief section 9)
- **Scenarios BDD** :

```gherkin
Scenario: Notification selon preferences utilisateur
  Given Sophie a configure ses preferences :
    | Type              | Email | SMS | Push | InApp |
    | MeetingReminder   | oui   | non | oui  | oui   |
    | TicketUpdate      | oui   | oui | non  | oui   |
    | CommunityNotice   | non   | non | non  | oui   |
  When un ticket est mis a jour concernant Sophie
  Then Sophie recoit une notification Email et SMS (pas de Push)
  And la notification InApp est visible dans son dashboard

Scenario: Marquer toutes les notifications comme lues
  Given Sophie a 15 notifications non lues
  When Sophie clique "Tout marquer comme lu"
  Then toutes les 15 notifications passent en is_read = true
```

- **Dependances** : Tous les modules (generent des evenements)
- **Endpoints** : `GET /notifications/my`, `GET /notifications/unread`, `PUT /notifications/:id/read`, `PUT /notifications/read-all`, `GET/PUT /notification-preferences/:user_id/:type`

---

## 7. Exigences non-fonctionnelles

> Derivees du brief section 13 (contraintes) et section 16 (metriques).

### 7.1 Performance (brief section 16)

| Metrique | Cible | Mesure |
|----------|-------|--------|
| Latence P99 | < 5ms | Prometheus endpoint `/metrics` + Grafana |
| Throughput | > 100k req/s | Load tests Criterion/k6 |
| Memoire par instance | < 128 MB | `process_resident_memory_bytes` |
| Connection pool PostgreSQL | Max 10 connexions | Configuration sqlx |
| Compilation release | LTO + `codegen-units=1` + `opt-level=3` | `Cargo.toml` profile |

### 7.2 Securite (brief section 13, 15)

- **Chiffrement at-rest** : LUKS AES-XTS-512 (PostgreSQL data + uploads)
- **GDPR** : Articles 15, 16, 17, 18, 21, 30 implementes (cf. FR-015)
- **2FA** : TOTP avec backup codes (cf. FR-018)
- **Rate limiting** : 5 tentatives login / 15 min / IP
- **IDS** : Suricata avec regles custom (SQL injection, XSS, path traversal)
- **WAF** : CrowdSec community threat intelligence
- **fail2ban** : Jails custom (SSH, Traefik, API abuse, PostgreSQL brute-force)
- **SSH Hardening** : Key-only, modern ciphers, reduced attack surface
- **Kernel Hardening** : SYN cookies, IP spoofing protection, ASLR
- **Headers** : HSTS (1 an), CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- **Audit** : Lynis hebdomadaire (score > 80/100), rkhunter quotidien, AIDE file integrity
- **Backups** : GPG-encrypted, S3 off-site (7j local)
- **JWT** : Secret minimum 32 chars, CORS sans wildcards

### 7.3 Scalabilite organisationnelle (brief section 13)

| Phase | Organisation | Taille equipe | Trigger |
|-------|-------------|---------------|---------|
| Phase 1 | Scrum | 1-7 personnes | Initial |
| Phase 2 | Nexus | 3-9 equipes | > 7 devs ou > 3 bounded contexts en parallele |
| Phase 3 | SAFe | 50+ agents | > 9 equipes, alignement portfolio strategique |

### 7.4 Exploitabilite ITIL

- **Incident Management** : Alertmanager -> Grafana -> resolution
- **Change Management** : PR review obligatoire, CI/CD GitHub Actions
- **Release Management** : Tags semantiques, progression par jalons de capacite
- **Monitoring** : Prometheus (30j metriques), Loki (7j logs), Grafana dashboards
- **Uptime cible** : 99.9%
- **Incidents P1/mois** : 0

### 7.5 Testabilite (brief section 13)

| Niveau | Outil | Couverture |
|--------|-------|------------|
| Unit (domain) | `#[cfg(test)]` in-module | 100% cible |
| Integration | testcontainers PostgreSQL | Tous les repositories |
| BDD | Cucumber/Gherkin | 819 scenarios, 69 features |
| E2E | Playwright | 49 smoke tests, 12 Doc Vivante |
| Benchmarks | Criterion | Performance critique |
| Securite | Lynis, rkhunter, AIDE | Hebdomadaire/quotidien |

### 7.6 Accessibilite

- WCAG 2.1 AA (cible)
- Frontend Astro + Svelte avec Tailwind CSS semantique

### 7.7 Observabilite

| Outil | Role | Retention |
|-------|------|-----------|
| Prometheus | Metriques applicatives + infra | 30 jours |
| Grafana | Dashboards + alertes visuelles | — |
| Loki | Agregation logs | 7 jours |
| Alertmanager | Routage alertes | — |
| Backend `/metrics` | Endpoint Prometheus scrape | — |

### 7.8 Ecologie / Green IT (brief section 13)

- Cible : < 0.5g CO2/requete (actuel mesure : 0.12g)
- Anti-bloatware : zero dependance superflue, zero JavaScript inutile
- Astro Islands : JavaScript minimal (hydratation partielle)
- Compilation Rust optimisee : LTO, codegen-units=1
- Connection pooling strict : max 10 PostgreSQL

### 7.9 Internationalisation (brief section 13)

- 4 langues : FR, NL, EN, DE
- ~2 000 cles par locale
- Couverture actuelle : 73%
- Convocations AG generees dans la langue du destinataire

---

## 8. Modele de donnees (entites DDD -> tables PostgreSQL)

> 59 entites domaine -> 80 migrations PostgreSQL. Toutes les entites utilisent UUID pour les IDs et incluent `created_at`/`updated_at` timestamps.

### 8.1 Building Management

```
buildings (id, organization_id, name, address, city, postal_code, country, total_units,
           construction_year, slug, syndic_name, syndic_email, syndic_phone,
           syndic_address, syndic_office_hours, syndic_emergency_contact,
           created_at, updated_at)

units (id, building_id, unit_number, floor, area, unit_type, created_at, updated_at)

unit_owners (id, unit_id, owner_id, percentage, start_date, end_date,
             is_primary_contact, created_at, updated_at)
  -- Trigger: validate_unit_ownership_total (somme <= 100%, tolerance +/-0.01%)
```

### 8.2 Identity & Access

```
users (id, organization_id, email, password_hash, first_name, last_name,
       phone, is_active, processing_restricted, processing_restricted_at,
       marketing_opt_out, marketing_opt_out_at, created_at, updated_at)

organizations (id, name, slug, contact_email, subscription_plan, max_buildings,
               max_users, is_active, created_at, updated_at)

user_roles (id, user_id, organization_id, role, is_primary, created_at, updated_at)

refresh_tokens (id, user_id, token_hash, expires_at, is_revoked, created_at)

two_factor_secrets (id, user_id, organization_id, secret, is_enabled,
                    backup_codes, verified_at, last_used_at, created_at, updated_at)
```

### 8.3 General Assembly

```
meetings (id, building_id, organization_id, meeting_date, meeting_type, agenda,
          minutes, quorum_validated, quorum_percentage, total_quotas,
          present_quotas, created_at, updated_at)

convocations (id, meeting_id, building_id, organization_id, meeting_type,
              meeting_date, minimum_send_date, status, pdf_file_path, language,
              total_recipients, opened_count, will_attend_count,
              respects_legal_deadline, created_at, updated_at)

convocation_recipients (id, convocation_id, owner_id, email_sent_at,
                        email_opened_at, email_failed, reminder_sent_at,
                        attendance_status, proxy_owner_id, needs_reminder,
                        created_at, updated_at)

resolutions (id, meeting_id, title, description, resolution_type,
             majority_required, majority_threshold, votes_for, votes_against,
             votes_abstention, total_voting_power, status, created_at, updated_at)

votes (id, resolution_id, owner_id, choice, voting_power, proxy_owner_id,
       created_at, updated_at)

ag_sessions (id, meeting_id, platform, video_url, host_url, status,
             remote_attendees_count, remote_voting_power,
             quorum_remote_contribution, access_password,
             waiting_room_enabled, recording_enabled, created_at, updated_at)

age_requests (id, building_id, organization_id, initiator_owner_id, title,
              description, status, total_shares_pct, threshold_pct,
              submitted_to_syndic_at, syndic_deadline_at,
              auto_convocation_triggered, concertation_poll_id,
              created_at, updated_at)

age_request_cosignatories (id, age_request_id, owner_id, shares_pct, signed_at)
```

### 8.4 Accounting

```
accounts (id, organization_id, code, name, account_type, parent_code,
          is_active, created_at, updated_at)
  -- ~90 comptes PCMN pre-seedes (8 classes)

journal_entries (id, organization_id, building_id, journal_type, entry_date,
                 description, document_ref, created_at, updated_at)
  -- journal_type: ACH/VEN/FIN/ODS

journal_entry_lines (id, journal_entry_id, account_code, debit, credit,
                     description, created_at, updated_at)

budgets (id, building_id, organization_id, fiscal_year, status,
         total_budget_amount, actual_expenses, variance, meeting_id,
         rejection_reason, created_at, updated_at)

etats_dates (id, unit_id, building_id, organization_id, reference_number,
             status, language, financial_data, additional_data,
             pdf_file_path, created_at, updated_at)
```

### 8.5 Billing & Payments

```
expenses (id, building_id, organization_id, description, amount, category,
          status, approved_by, approved_at, rejected_reason, created_at, updated_at)

invoice_line_items (id, expense_id, description, quantity, unit_price,
                    vat_rate, total_excl_vat, total_vat, total_incl_vat,
                    created_at, updated_at)

payments (id, organization_id, owner_id, expense_id, building_id,
          amount_cents, currency, status, payment_method_type,
          stripe_payment_intent_id, idempotency_key,
          refunded_amount_cents, created_at, updated_at)

payment_methods (id, owner_id, organization_id, method_type,
                 stripe_payment_method_id, display_label, is_default,
                 is_active, expires_at, created_at, updated_at)

payment_reminders (id, expense_id, owner_id, organization_id, level, status,
                   sent_date, tracking_number, penalty_amount, notes,
                   created_at, updated_at)

owner_contributions (id, organization_id, owner_id, unit_id,
                     call_for_funds_id, description, amount,
                     contribution_type, payment_status, payment_date,
                     account_code, created_at, updated_at)

call_for_funds (id, organization_id, building_id, title, total_amount,
                contribution_type, status, call_date, due_date,
                account_code, created_at, updated_at)

charge_distributions (id, expense_id, unit_id, owner_id, percentage,
                      amount_cents, created_at, updated_at)
```

### 8.6 Maintenance

```
tickets (id, building_id, organization_id, title, description, priority,
         status, category, requester_id, assigned_contractor_id,
         due_date, resolved_at, created_at, updated_at)

quotes (id, building_id, organization_id, contractor_id, project_title,
        description, amount_excl_vat, vat_rate, amount_incl_vat,
        validity_date, estimated_duration_days, warranty_years,
        contractor_rating, status, decision_at, decision_by,
        decision_notes, created_at, updated_at)

work_reports (id, building_id, organization_id, work_type, title,
              description, contractor_name, start_date, end_date,
              warranty_years, warranty_type, photos, documents,
              created_at, updated_at)

technical_inspections (id, building_id, organization_id, inspection_type,
                       status, inspector_name, inspection_date,
                       next_inspection_date, reports, photos, certificates,
                       created_at, updated_at)

contractor_reports (id, ticket_id, quote_id, organization_id, contractor_name,
                    work_date, compte_rendu, photos_before, photos_after,
                    parts_replaced, status, magic_token_hash,
                    magic_token_expires_at, created_at, updated_at)
```

### 8.7 Community & Gamification

```
local_exchanges (id, building_id, organization_id, provider_id, requester_id,
                 exchange_type, title, description, credits, status,
                 provider_rating, requester_rating, cancellation_reason,
                 created_at, updated_at)

owner_credit_balances (id, owner_id, building_id, credits_earned,
                       credits_spent, balance, total_exchanges,
                       average_rating, participation_level, created_at, updated_at)

polls (id, building_id, organization_id, poll_type, question, description,
       status, starts_at, ends_at, is_anonymous, total_eligible_voters,
       total_votes_cast, allow_multiple_votes, min_rating, max_rating,
       created_at, updated_at)

poll_options (id, poll_id, option_text, option_value, display_order,
              vote_count, created_at, updated_at)

poll_votes (id, poll_id, owner_id, option_id, vote_value, vote_text,
            ip_address, is_anonymous, created_at, updated_at)
  -- UNIQUE(poll_id, owner_id) pour prevention doublons

achievements (id, organization_id, category, tier, name, description, icon,
              points_value, requirements, is_secret, is_repeatable,
              display_order, created_at, updated_at)

user_achievements (id, user_id, achievement_id, earned_at, progress_data,
                   times_earned, created_at, updated_at)

challenges (id, organization_id, building_id, challenge_type, status, title,
            description, icon, start_date, end_date, target_metric,
            target_value, reward_points, created_at, updated_at)

challenge_progress (id, challenge_id, user_id, current_value, completed,
                    completed_at, created_at, updated_at)
```

---

## 9. Integrations externes

| Systeme | Usage | Module | Priorite |
|---------|-------|--------|----------|
| **Stripe Payment Intents** | Paiements carte bancaire | Billing & Payments | MUST (Jalon 2) |
| **SEPA Direct Debit** (via Stripe) | Prelevements IBAN belge | Billing & Payments | MUST (Jalon 2) |
| **SMTP** (email) | Notifications, convocations, relances | Notifications | MUST (Jalon 1) |
| **SMS Gateway** (Twilio/OVH) | Alertes urgentes maintenance | Notifications | SHOULD (Jalon 3) |
| **Zoom/Teams/Meet/Jitsi/Whereby** | Visioconference AG | General Assembly | SHOULD (Jalon 3) |
| **Linky/Ores API** | Consommation energetique | Energy & IoT | COULD (Jalon 4) |
| **itsme/eID** (OpenID Connect) | Authentification forte belge | Identity & Access | COULD (Jalon 4) |
| **Winbooks** (export comptable) | Export ecritures vers logiciel comptable | Accounting | COULD (Jalon 4) |
| **PSD2 Banking API** | Reconciliation bancaire automatique | Billing & Payments | WON'T v0.1.0 (Jalon 6) |
| **Polygon blockchain** | Votes AG immutables | General Assembly | WON'T v0.1.0 (Jalon 7) |

---

## 10. Contraintes et hypotheses

### 10.1 Contraintes (brief section 13)

1. **Stack imposee** : Rust 1.75+ / Actix-web 4.9, Astro 4.x / Svelte 4.x, PostgreSQL 15, Tailwind CSS 3.x
2. **Architecture** : Hexagonale stricte (Ports & Adapters) + DDD + SOLID
3. **Tests** : BDD obligatoire pour chaque FR (Gherkin), TDD pour le Domain (100% couverture), pyramide complete (unit -> integration -> BDD -> E2E -> benchmarks)
4. **Securite** : RGPD des la conception, chiffrement at-rest LUKS, defense en profondeur
5. **Infrastructure** : IaC (Terraform + Ansible), CI/CD GitHub Actions, monitoring Prometheus/Grafana/Loki
6. **Performance** : P99 < 5ms, >100k req/s, <128 MB RAM/instance, pool max 10 PostgreSQL
7. **Licence** : AGPL-3.0
8. **i18n** : 4 langues (FR/NL/EN/DE)
9. **Developpement** : Agents IA supervises (Methode Maury), code review humain obligatoire
10. **Base existante** : 559 endpoints, 59 entites, 80 migrations, 137k+ LOC — toute modification doit etre retrocompatible

### 10.2 Hypotheses

1. Le marche cible est exclusivement belge pour v0.1.0 (extension Benelux possible post-v1.0)
2. Les syndics professionnels sont le premier segment d'adoption (15-50 coproprietes par syndic)
3. Le modele ASBL avec cotisation 5 EUR/copro/mois est viable pour le financement durable
4. Les prestataires acceptent l'interface magic link PWA sans creation de compte
5. La legislation belge Art. 577 CC ne changera pas de maniere incompatible pendant le developpement du MVP
6. Un seul developpeur (10-15h/semaine) avec assistance IA peut delivrer le MVP en mode jalon par capacite
7. PostgreSQL 15 est suffisant pour les volumes MVP (< 1 000 coproprietes, < 100k utilisateurs)
8. Stripe couvre les besoins de paiement belges (cartes + SEPA)

---

## 11. Criteres de succes MVP (Jalons 0-3)

> Repris du brief section 16 et enrichis avec des seuils mesurables.

| # | Critere | Seuil | Comment mesurer | Jalon |
|---|---------|-------|-----------------|-------|
| S1 | Latence P99 | < 5ms | Prometheus + Grafana | Continu |
| S2 | Throughput | > 100k req/s | k6 load tests | Continu |
| S3 | Memoire | < 128 MB/instance | Prometheus | Continu |
| S4 | CO2/requete | < 0.5g (actuel: 0.12g) | Green Metrics Tool | Continu |
| S5 | Tests domain | 100% couverture | Tarpaulin | Continu |
| S6 | Scenarios BDD | >= 819 scenarios | `cargo test --test bdd` | Continu |
| S7 | E2E tests | >= 49 smoke + 12 Doc Vivante | Playwright | Continu |
| S8 | Uptime | 99.9% | Grafana + Alertmanager | Production |
| S9 | Incidents P1 | 0/mois | ITIL Incident Log | Production |
| S10 | GDPR | 6/6 articles implementes | Tests automatises + audit checklist | Jalon 1 (acheve) |
| S11 | Conformite belge | >= 80% checklist Art. 577 | Audit juridique | Jalon 2 |
| S12 | Coproprietes hebergees | >= 500 | Compteur organisations actives | Jalon 3 |
| S13 | Temps convocation AG | < 5 min | Metriques application | Jalon 2 |
| S14 | Taux recouvrement | > 85% avant LegalAction | Dashboard comptable | Jalon 3 |
| S15 | Score Lynis securite | > 80/100 | Audit hebdomadaire automatise | Continu |

---

## Tracabilite Brief -> PRD

| Section Brief | Section PRD | FR associees |
|---------------|-------------|-------------|
| section 6 Personas (Marc, Sophie, Jean-Pierre, Ahmed) | section 6 User Stories | Toutes |
| section 8 Glossaire | section 4 Glossaire enrichi | Toutes |
| section 9 Bounded Contexts (13 contextes) | section 5 Modules Rust | Toutes |
| section 10 Invariant 1 (quotes-parts 100%) | FR-002 | INV-1 |
| section 10 Invariant 2 (quorum 50%) | FR-004 | INV-2 |
| section 10 Invariant 3 (delai 15j) | FR-003 | INV-3 |
| section 10 Invariant 4 (majorites) | FR-004 | INV-4 |
| section 10 Invariant 5 (double-entree) | FR-006 | INV-5 |
| section 10 Invariant 6 (quote-part 0<p<=1) | FR-002 | INV-6 |
| section 10 Invariant 7 (idempotency) | FR-010 | INV-7 |
| section 10 Invariant 8 (seuil AGE 1/5) | FR-005 | INV-8 |
| section 10 Invariant 9 (building.name) | FR-001 | INV-9 |
| section 10 Invariant 10 (3 devis >5000 EUR) | FR-014 | INV-10 |
| section 11 Fonctionnalites cles | section 3.1 Perimetre MVP | FR-001 a FR-019 |
| section 12 Fonctionnalites secondaires | section 3.2 Post-MVP | — |
| section 13 Contraintes | section 10 Contraintes | NFR section 7 |
| section 14 Risques | section 10.2 Hypotheses (mitigations) | — |
| section 16 Metriques | section 11 Criteres de succes | S1-S15 |

---

## Pipeline suivant

Ce PRD sera consomme par :
- **Etape 3** : Architecte (architecture hexagonale SOLID) -> `architecture.md`
- **Etape 4** : Scrum Master (stories TDD pour agents IA) -> `epics-and-stories.md`
- **Etape 5** : Validation croisee -> `validation-report.md`

---

*Document genere par John (Product Manager BMAD) — Methode Maury Phase TOGAF B-C*
*Pipeline : product-brief.md (Mary, Phase A) -> PRD.md (John, Phase B-C)*
*Prochaine etape : Architecture (Phase C-D) par l'Architecte BMAD*

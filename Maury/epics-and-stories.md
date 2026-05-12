# Epics & User Stories — KoproGo

## Methode Maury — Phase TOGAF E (Solutions)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : Bob (Scrum Master)
**Date** : 29/03/2026
**Version** : 2.0
**Brief source** : `Maury/product-brief.md` v1.0 (29/03/2026, Mary — Analyste)
**PRD source** : `Maury/PRD.md` v1.0 (29/03/2026, John — Product Manager)
**Architecture source** : `Maury/architecture.md` v2.0 (29/03/2026, Winston — Architecte)

**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Execution** : Scrum -> Nexus -> SAFe
**Production** : ITIL + IaC ISO 27001

**Contexte** : KoproGo est une plateforme SaaS belge de gestion de copropriete. Architecture hexagonale Rust + Actix-web, frontend Astro + Svelte, PostgreSQL 15. 559 endpoints API, 60 entites domaine, 82 migrations, 137k+ LOC Rust. Conformite loi belge copropriete (Code Civil Art. 577 et suivants) et RGPD.

**Tracabilite** : Ce document consomme et transforme le brief (Mary, Phase TOGAF A), le PRD (John, Phase TOGAF B-C) et l'architecture (Winston, Phase TOGAF C-D) en stories executables pour agents IA. Chaque story trace vers un FR du PRD, un invariant du brief, et des chemins de fichiers reels de l'architecture.

---

## Sprint 0 — Fondations (COMPLETE — Jalon 0)

| # | ID | Titre | Taille | Statut |
|---|-----|-------|--------|--------|
| 1 | STORY-T01 | Setup projet Rust + architecture dossiers SOLID/Hexa | S | DONE |
| 2 | STORY-T02 | Setup framework tests (BDD Cucumber + testcontainers) | M | DONE |
| 3 | STORY-T03 | Docker Compose (PostgreSQL + backend + frontend) | M | DONE |
| 4 | STORY-T04 | CI GitHub Actions (test + clippy + fmt + audit + BDD) | S | DONE |
| 5 | STORY-T05 | Setup Astro + Svelte (Islands Architecture) | S | DONE |
| 6 | STORY-T06 | IaC Terraform : provisioning VPS OVH (prepare ITIL) | M | DONE |
| 7 | STORY-T07 | IaC Ansible : security hardening ISO 27001 (LUKS, fail2ban, Suricata, CrowdSec) | M | DONE |
| 8 | STORY-T08 | Monitoring : Prometheus + Grafana + Loki + Alertmanager (prepare ITIL) | M | DONE |

---

## Epic 1 : Building Management | MUST HAVE

> Bounded Context DDD : Building Management (brief section 9, architecture section 2.1 #1)
> FRs PRD : FR-001, FR-002
> Jalon : 0 (acheve)

### Story 1.1 : Gestion des immeubles (CRUD)

- **ID** : STORY-001 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Building Management
- **Entite(s) DDD** : `Building` (brief section 9)
- **Principes SOLID** : SRP (Building ne gere que les donnees immobilieres), OCP (ajout champs syndic sans modifier use cases), DIP (BuildingRepository trait)
- **Invariants** : INV-9 (Building.name non vide)
- **FR PRD** : FR-001
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux enregistrer la "Residence du Parc" avec ses 45 lots pour gerer la copropriete de maniere centralisee.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/building.rs` (#[cfg(test)] — valider INV-9 : name non vide, total_units > 0, slug generation)
  2. [ ] GREEN : Implementer entite `Building::new()` avec invariants -> `Result<Self, String>`
  3. [ ] RED : Ecrire fichier BDD `backend/tests/features/building.feature` (3 scenarios ci-dessus)
  4. [ ] Definir port `backend/src/application/ports/building_repository.rs` (trait BuildingRepository : 7 methodes)
  5. [ ] Implementer use case `backend/src/application/use_cases/building_use_cases.rs` (CRUD + find_by_slug)
  6. [ ] Creer DTO `backend/src/application/dto/building_dto.rs` (CreateBuildingDto, BuildingResponseDto, BuildingFilters, PageRequest)
  7. [ ] RED : Ecrire tests integration (testcontainers PostgreSQL)
  8. [ ] GREEN : Implementer repository `backend/src/infrastructure/database/repositories/building_repository_impl.rs` (PostgresBuildingRepository — 19 colonnes)
  9. [ ] Creer migration `backend/migrations/[timestamp]_create_buildings.sql`
  10. [ ] Implementer handlers `backend/src/infrastructure/web/handlers/building_handlers.rs` + `public_handlers.rs`
  11. [ ] Ecrire tests E2E (endpoints GET/POST /buildings, GET /public/buildings/:slug/syndic)
  12. [ ] REFACTOR si necessaire
  13. [ ] Verifier : `cargo test` (all)
- **Dependances** : Aucune (module fondation)
- **Endpoints** : `GET/POST /buildings`, `GET/PUT/DELETE /buildings/:id`, `GET /public/buildings/:slug/syndic`

---

### Story 1.2 : Gestion des lots et quotes-parts

- **ID** : STORY-002 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Building Management
- **Entite(s) DDD** : `Unit`, `UnitOwner` (brief section 9)
- **Principes SOLID** : SRP (UnitOwner gere uniquement la relation lot-proprietaire), LSP (tout UnitOwnerRepository substituable), ISP (traits separes UnitRepository / UnitOwnerRepository)
- **Invariants** : INV-1 (somme quotes-parts actives = 100%, tolerance +/-0.01%), INV-6 (0.0 < p <= 1.0)
- **FR PRD** : FR-002
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux attribuer a Sophie (brief section 6.2) sa quote-part de 450/10000 dix-milliemes pour le lot 3B, afin que ses charges et son pouvoir de vote soient correctement calcules.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/unit.rs` et `backend/src/domain/entities/unit_owner.rs` (valider INV-1, INV-6, transfert propriete)
  2. [ ] GREEN : Implementer entites `Unit::new()`, `UnitOwner::new()` avec validation 0.0 < p <= 1.0
  3. [ ] RED : Ecrire fichier BDD `backend/tests/features/unit_owner_validation.feature`
  4. [ ] Definir ports `backend/src/application/ports/unit_repository.rs` et `backend/src/application/ports/unit_owner_repository.rs`
  5. [ ] Implementer use cases `backend/src/application/use_cases/unit_use_cases.rs` et `backend/src/application/use_cases/unit_owner_use_cases.rs` (somme quotes-parts <= 100%, transfert, historique)
  6. [ ] RED : Ecrire tests integration (testcontainers — verifier trigger PostgreSQL `validate_unit_ownership_total`)
  7. [ ] GREEN : Implementer repositories `backend/src/infrastructure/database/repositories/unit_repository_impl.rs` et `unit_owner_repository_impl.rs`
  8. [ ] Creer migration `backend/migrations/[timestamp]_create_units.sql` + `[timestamp]_add_unit_ownership_validation.sql` (trigger PostgreSQL INV-1)
  9. [ ] Implementer handlers `backend/src/infrastructure/web/handlers/unit_handlers.rs` et `unit_owner_handlers.rs`
  10. [ ] Ecrire tests E2E (POST /units/:id/owners, POST /units/:id/owners/transfer, GET /units/:id/owners/total-percentage)
  11. [ ] REFACTOR
  12. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-001
- **Endpoints** : `GET/POST /units`, `GET /buildings/:id/units`, `POST /units/:id/owners`, `PUT /unit-owners/:id`, `POST /units/:id/owners/transfer`, `GET /units/:id/owners/total-percentage`

---

## Epic 2 : Identity & Access | MUST HAVE

> Bounded Context DDD : Identity & Access (brief section 9, architecture section 2.1 #2)
> FR PRD : FR-018
> Jalon : 1 (acheve)

### Story 2.1 : Authentification multi-role avec JWT et 2FA

- **ID** : STORY-003 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Identity & Access
- **Entite(s) DDD** : `User`, `UserRoleAssignment`, `RefreshToken`, `TwoFactorSecret` (brief section 9)
- **Principes SOLID** : SRP (User = identite, UserRoleAssignment = roles, TwoFactorSecret = 2FA), DIP (UserRepository, UserRoleRepository traits)
- **FR PRD** : FR-018
- **User Story** : En tant que Marc (syndic qui est aussi coproprietaire, brief section 6.1), je veux basculer entre mon role "syndic" et mon role "coproprietaire" sans me reconnecter, et securiser mon compte avec 2FA.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/user.rs`, `backend/src/domain/entities/user_role_assignment.rs`, `backend/src/domain/entities/two_factor_secret.rs`
  2. [ ] GREEN : Implementer entites User, UserRoleAssignment, RefreshToken, TwoFactorSecret
  3. [ ] RED : Ecrire fichier BDD `backend/tests/features/auth.feature` et `backend/tests/features/two_factor.feature`
  4. [ ] Definir ports `backend/src/application/ports/user_repository.rs`, `user_role_repository.rs`, `two_factor_repository.rs`, `organization_repository.rs`, `refresh_token_repository.rs`
  5. [ ] Implementer use cases `backend/src/application/use_cases/auth_use_cases.rs` (login, switch_role, refresh_token) et `two_factor_use_cases.rs`
  6. [ ] RED : Ecrire tests integration (testcontainers)
  7. [ ] GREEN : Implementer repositories `backend/src/infrastructure/database/repositories/user_repository_impl.rs`, `user_role_repository_impl.rs`, `two_factor_repository_impl.rs`
  8. [ ] Creer migrations (users, user_roles, refresh_tokens, two_factor_secrets, organizations)
  9. [ ] Implementer handlers `backend/src/infrastructure/web/handlers/auth_handlers.rs`, `two_factor_handlers.rs`, `user_handlers.rs`, `organization_handlers.rs`
  10. [ ] Ecrire tests E2E (POST /auth/login, POST /auth/switch-role, POST /2fa/setup, POST /2fa/enable, POST /2fa/verify)
  11. [ ] REFACTOR
  12. [ ] Verifier : `cargo test` (all)
- **Dependances** : Aucune (module fondation transversal)
- **Endpoints** : `POST /auth/login`, `POST /auth/switch-role`, `GET /auth/me`, `POST /2fa/setup`, `POST /2fa/enable`, `POST /2fa/verify`, `POST /2fa/disable`, `GET /2fa/status`

---

## Epic 3 : General Assembly | MUST HAVE

> Bounded Context DDD : General Assembly (brief section 9, architecture section 2.1 #3)
> FRs PRD : FR-003, FR-004, FR-005
> Jalon : 1-3

### Story 3.1 : Convocations legales avec delai 15 jours

- **ID** : STORY-004 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : General Assembly
- **Entite(s) DDD** : `Convocation`, `ConvocationRecipient` (brief section 9)
- **Principes SOLID** : SRP (Convocation = invitation, pas vote ni quorum), OCP (ajout canaux SMS/Push sans modifier Domain), DIP (ConvocationRepository + ConvocationRecipientRepository traits)
- **Invariants** : INV-3 (delai legal >= 15 jours — Art. 3.87 ss3 CC)
- **FR PRD** : FR-003
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux que KoproGo calcule automatiquement la date limite d'envoi des convocations pour l'AG du 20 mars, afin de ne jamais risquer l'annulation pour vice de forme.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/convocation.rs` (440 lignes) et `backend/src/domain/entities/convocation_recipient.rs` (260 lignes) — valider INV-3, workflow Draft->Scheduled->Sent->Cancelled, calcul minimum_send_date
  2. [ ] GREEN : Implementer entites avec validation delai legal, multi-langue (FR/NL/DE/EN)
  3. [ ] RED : Ecrire fichier BDD `backend/tests/features/convocations.feature` et `backend/tests/features/second_convocation.feature`
  4. [ ] Definir ports `backend/src/application/ports/convocation_repository.rs` (13 methodes) et `backend/src/application/ports/convocation_recipient_repository.rs` (18 methodes)
  5. [ ] Implementer use case `backend/src/application/use_cases/convocation_use_cases.rs` (21 methodes, multi-repo orchestration)
  6. [ ] Creer DTOs `backend/src/application/dto/convocation_dto.rs` et `convocation_recipient_dto.rs`
  7. [ ] RED : Ecrire tests integration (testcontainers)
  8. [ ] GREEN : Implementer repositories `backend/src/infrastructure/database/repositories/convocation_repository_impl.rs` (600 lignes) et `convocation_recipient_repository_impl.rs` (750 lignes)
  9. [ ] Creer migration `backend/migrations/[timestamp]_create_convocations.sql` (2 tables, 3 ENUMs custom, 14 indexes, 10 constraints)
  10. [ ] Implementer handler `backend/src/infrastructure/web/handlers/convocation_handlers.rs` (14 endpoints)
  11. [ ] Ecrire tests E2E
  12. [ ] REFACTOR
  13. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-001 (Building), STORY-003 (Identity & Access)
- **Endpoints** : `POST /convocations`, `PUT /convocations/:id/schedule`, `POST /convocations/:id/send`, `POST /convocations/:id/reminders`, `PUT /convocation-recipients/:id/proxy`, `PUT /convocation-recipients/:id/attendance`, `GET /convocations/:id/tracking-summary`

---

### Story 3.2 : Vote numerique avec majorites legales belges

- **ID** : STORY-005 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : General Assembly
- **Entite(s) DDD** : `Resolution`, `Vote` (brief section 9)
- **Principes SOLID** : SRP (Resolution = proposition, Vote = suffrage individuel), OCP (nouveaux types majorite sans modifier moteur calcul), LSP (ResolutionRepository et VoteRepository substituables)
- **Invariants** : INV-2 (quorum AG >= 50%), INV-4 (majorites legales : simple, absolue, qualifiee 2/3, 4/5, unanimite)
- **FR PRD** : FR-004
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux voter "Pour" la resolution de travaux de toiture lors de l'AG, avec mon pouvoir de vote de 45 milliemes, et voir le resultat en temps reel.
- **Scenarios BDD** :

```gherkin
Scenario: Verifier le quorum avant ouverture des votes (INV-2)
  Given l'AG de "Residence du Parc Royal" a 10000 dix-milliemes au total
  And les coproprietaires presents et representes totalisent 4500 dix-milliemes
  When le syndic tente d'ouvrir les votes
  Then une erreur "Quorum non atteint : 45% < 50% requis" est retournee
  And aucun vote ne peut etre enregistre

Scenario: Voter avec majorite Absolute — resolution adoptee (INV-4, Art. 3.88 §1)
  Given l'AG a le quorum valide (5200 dix-milliemes sur 10000)
  And la resolution "Changer de fournisseur d'electricite" requiert une majorite Absolute
  When Sophie vote "Pour" avec 450 dix-milliemes
  And Alice vote "Pour" avec 800 dix-milliemes (dont 300 par procuration de Pierre)
  And Jean vote "Contre" avec 600 dix-milliemes
  And le syndic cloture le scrutin
  Then les votes "Pour" totalisent 1250 dix-milliemes (67.6% des exprimes, abstentions exclues)
  And les votes "Contre" totalisent 600 dix-milliemes (32.4% des exprimes)
  And la resolution est "Adopted" (1250 > 50% de 1850 exprimes)

Scenario: Voter avec majorite TwoThirds — resolution rejetee (INV-4, Art. 3.88 §1, 1°)
  Given l'AG a le quorum valide
  And la resolution "Travaux structurels de toiture pour 45 000 EUR" requiert une majorite TwoThirds (66.67%)
  When les votes "Pour" totalisent 3000 dix-milliemes sur 5000 exprimes (60%)
  And le syndic cloture le scrutin
  Then la resolution est "Rejected" (60% < 66.67% requis)

Scenario: Voter avec Unanimity — calcul sur TOUS les tantiemes (INV-4, Art. 3.88 §1, 3°)
  Given l'AG a 10000 dix-milliemes au total, quorum valide avec 6000 dix-milliemes presents
  And la resolution "Dissolution de la copropriete" requiert Unanimity
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/resolution.rs` et `backend/src/domain/entities/vote.rs` — valider INV-2 (quorum), INV-4 (majorites Absolute/TwoThirds/FourFifths/Unanimity), procuration
  2. [ ] GREEN : Implementer entites Resolution (MajorityType enum: Absolute/TwoThirds/FourFifths/Unanimity), Vote (choice: Pour/Contre/Abstention, voting_power 0-10000 dix-milliemes)
  3. [ ] RED : Ecrire fichiers BDD `backend/tests/features/resolutions.feature`, `backend/tests/features/vote_ag_workflow.feature`
  4. [ ] Definir ports `backend/src/application/ports/resolution_repository.rs` et `backend/src/application/ports/vote_repository.rs`
  5. [ ] Implementer use case `backend/src/application/use_cases/resolution_use_cases.rs` (14 methodes, calcul majorites)
  6. [ ] Creer DTOs `backend/src/application/dto/resolution_dto.rs` et `vote_dto.rs`
  7. [ ] RED : Ecrire tests integration (testcontainers)
  8. [ ] GREEN : Implementer repositories `backend/src/infrastructure/database/repositories/resolution_repository_impl.rs` et `vote_repository_impl.rs`
  9. [ ] Creer migration `backend/migrations/[timestamp]_create_resolutions_and_votes.sql` (10 contraintes + 8 index)
  10. [ ] Implementer handler `backend/src/infrastructure/web/handlers/resolution_handlers.rs` (9 endpoints)
  11. [ ] Ecrire tests E2E
  12. [ ] REFACTOR
  13. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-004 (Convocation), STORY-002 (UnitOwner pour milliemes)
- **Endpoints** : `POST /meetings/:id/resolutions`, `POST /resolutions/:id/vote`, `PUT /votes/:id`, `PUT /resolutions/:id/close`, `GET /meetings/:id/vote-summary`

---

### Story 3.3 : AGE (Assemblee Generale Extraordinaire) par petition

- **ID** : STORY-006 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : General Assembly
- **Entite(s) DDD** : `AgeRequest`, `AgeRequestCosignatory` (brief section 9)
- **Principes SOLID** : SRP, OCP (nouveaux statuts sans modifier Domain)
- **Invariants** : INV-8 (seuil AGE 1/5 des quotes-parts = 20% — Art. 3.87 ss2 CC)
- **FR PRD** : FR-005
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux lancer une petition pour une AGE sur les travaux urgents de toiture, et KoproGo me dit automatiquement quand le seuil de 1/5 est atteint.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/age_request.rs` — valider INV-8 (seuil 20%), workflow Draft->Open->Reached->Submitted->Accepted/Rejected
  2. [ ] GREEN : Implementer entites AgeRequest, AgeRequestCosignatory
  3. [ ] RED : Ecrire fichier BDD `backend/tests/features/age_requests.feature`
  4. [ ] Definir ports `backend/src/application/ports/age_request_repository.rs`
  5. [ ] Implementer use case `backend/src/application/use_cases/age_request_use_cases.rs`
  6. [ ] RED : Ecrire tests integration
  7. [ ] GREEN : Implementer repository `backend/src/infrastructure/database/repositories/age_request_repository_impl.rs`
  8. [ ] Creer migration
  9. [ ] Implementer handler `backend/src/infrastructure/web/handlers/age_request_handlers.rs`
  10. [ ] Ecrire tests E2E
  11. [ ] REFACTOR
  12. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-002 (UnitOwner pour quotes-parts), STORY-004 (Convocation pour auto-convocation)
- **Endpoints** : `POST /buildings/:id/age-requests`, `POST /age-requests/:id/cosign`, `POST /age-requests/:id/submit`, `POST /age-requests/:id/accept`, `POST /age-requests/:id/reject`

---

### Story 3.4 : Visioconference AG et AG Session

- **ID** : STORY-007 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : General Assembly
- **Entite(s) DDD** : `AgSession` (Art. 3.87 ss1 CC)
- **Principes SOLID** : SRP (AgSession gere la visio, pas le vote), OCP (ajout plateformes sans modifier Domain)
- **FR PRD** : FR-004 (composante visioconference)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux creer une session visioconference Zoom/Teams pour l'AG afin que les coproprietaires puissent participer a distance.
- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/ag_session.rs`
  2. [ ] GREEN : Entite AgSession (platform enum, workflow Scheduled->Live->Ended->Cancelled, quorum combine physique+remote)
  3. [ ] RED : BDD `backend/tests/features/ag_sessions.feature`
  4. [ ] Port + Use Case + Repository + Handler + Migration
  5. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-005
- **Endpoints** : `POST /meetings/:meeting_id/ag-session`, `PUT /ag-sessions/:id/start`, `PUT /ag-sessions/:id/end`, `GET /ag-sessions/:id/combined-quorum`

---

## Epic 4 : Comptabilite PCMN | MUST HAVE

> Bounded Context DDD : Accounting (brief section 9, architecture section 2.1 #4)
> FRs PRD : FR-006, FR-007, FR-008
> Jalon : 2 (acheve)

### Story 4.1 : Comptabilite PCMN belge et ecritures journal

- **ID** : STORY-008 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Accounting
- **Entite(s) DDD** : `Account`, `JournalEntry`, `JournalEntryLine` (brief section 9)
- **Principes SOLID** : SRP (Account = plan comptable, JournalEntry = ecritures), OCP (nouveaux types journaux sans modifier Domain), DIP (AccountRepository, JournalEntryRepository traits)
- **Invariants** : INV-5 (comptabilite double-entree equilibree : total debits = total credits)
- **FR PRD** : FR-006
- **User Story** : En tant que Jean-Pierre (comptable, brief section 6.3), je veux saisir une ecriture journal double-entree conforme au PCMN belge, avec verification automatique de l'equilibre debit/credit.
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

- **Taches techniques TDD** :
  1. [ ] RED : Ecrire tests unitaires domain `backend/src/domain/entities/account.rs` et `backend/src/domain/entities/journal_entry.rs` — valider INV-5, hierarchie PCMN, types journaux ACH/VEN/FIN/ODS
  2. [ ] GREEN : Implementer entites Account, JournalEntry, JournalEntryLine
  3. [ ] RED : Ecrire fichiers BDD `backend/tests/features/accounts.feature` et `backend/tests/features/journal_entries.feature`
  4. [ ] Definir ports `backend/src/application/ports/account_repository.rs` et `backend/src/application/ports/journal_entry_repository.rs`
  5. [ ] Implementer use cases `backend/src/application/use_cases/account_use_cases.rs`, `journal_entry_use_cases.rs`, `financial_report_use_cases.rs` (bilan, compte de resultats)
  6. [ ] RED : Ecrire tests integration (testcontainers)
  7. [ ] GREEN : Implementer repositories
  8. [ ] Creer migration (tables accounts, journal_entries, journal_entry_lines)
  9. [ ] Implementer handlers `backend/src/infrastructure/web/handlers/account_handlers.rs`, `financial_report_handlers.rs`
  10. [ ] Ecrire tests E2E
  11. [ ] REFACTOR
  12. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-001 (Building), STORY-003 (Identity/Organization)
- **Endpoints** : `GET/POST /accounts`, `POST /accounts/seed/belgian-pcmn`, `GET/POST /journal-entries`, `GET /reports/balance-sheet`, `GET /reports/income-statement`

---

### Story 4.2 : Budget annuel et variance

- **ID** : STORY-009 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Accounting
- **Entite(s) DDD** : `Budget` (brief section 9)
- **Principes SOLID** : SRP (Budget = enveloppe budgetaire), OCP (ajout statuts sans modifier Domain)
- **FR PRD** : FR-007
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux soumettre le budget 2026 de "Residence du Parc" pour approbation en AG, puis suivre les ecarts entre budget et depenses reelles.
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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/budget.rs` — workflow Draft->Submitted->Approved->Archived
  2. [ ] GREEN : Entite Budget avec validation
  3. [ ] RED : BDD `backend/tests/features/budget.feature`
  4. [ ] Port `backend/src/application/ports/budget_repository.rs` + Use Case `budget_use_cases.rs`
  5. [ ] Integration + Repository + Migration + Handler + E2E + REFACTOR
  6. [ ] Verifier : `cargo test` (all)
- **Dependances** : STORY-008 (Accounting), STORY-005 (AG pour approbation)
- **Endpoints** : `POST /budgets`, `PUT /budgets/:id/submit`, `PUT /budgets/:id/approve`, `GET /budgets/:id/variance`

---

### Story 4.3 : Etat date pour ventes immobilieres

- **ID** : STORY-010 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Accounting
- **Entite(s) DDD** : `EtatDate` (Art. 577-11 ss2 CC)
- **Principes SOLID** : SRP
- **FR PRD** : FR-008
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux generer l'etat date du lot 12A pour la vente par Sophie, avec le solde debiteur et les charges en cours, dans le delai legal de 10 jours.
- **Scenarios BDD** :

```gherkin
Scenario: Generer un etat date dans les delais
  Given Sophie vend le lot 12A et le notaire demande l'etat date le 01/03/2026
  When Marc genere l'etat date avec les donnees financieres (solde 1 250 EUR, charges 2026 payees)
  Then le document PDF est genere avec le numero de reference unique
  And le statut passe a "Generated"

Scenario: Alerter sur les etats dates en retard (>10 jours)
  Given un etat date a ete demande le 01/03/2026 et n'est pas encore genere le 15/03/2026
  When le systeme verifie les retards
  Then l'etat date apparait dans la liste "overdue" (14 jours > 10 jours legaux)

Scenario: Detecter les etats dates expires (>3 mois)
  Given un etat date a ete genere le 01/01/2026
  When le systeme verifie les expirations le 15/04/2026
  Then l'etat date apparait dans la liste "expired" (>3 mois depuis la date de reference)
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/etat_date.rs` — workflow Requested->InProgress->Generated->Delivered->Expired, delai 10 jours, validite 3 mois
  2. [ ] GREEN : Entite EtatDate
  3. [ ] RED : BDD `backend/tests/features/etat_date.feature`
  4. [ ] Port + Use Case `backend/src/application/use_cases/etat_date_use_cases.rs` + Repository + Migration + Handler + E2E
  5. [ ] REFACTOR + Verifier
- **Dependances** : STORY-001 (Building), STORY-002 (Unit/UnitOwner)
- **Endpoints** : `POST /etats-dates`, `GET /etats-dates/overdue`, `GET /etats-dates/expired`, `PUT /etats-dates/:id/mark-generated`, `PUT /etats-dates/:id/mark-delivered`

---

## Epic 5 : Billing & Payments | MUST HAVE

> Bounded Context DDD : Billing & Payments (brief section 9, architecture section 2.1 #5)
> FRs PRD : FR-009, FR-010, FR-011, FR-012
> Jalon : 2-3

### Story 5.1 : Facturation avec TVA belge

- **ID** : STORY-011 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Billing & Payments
- **Entite(s) DDD** : `Expense`, `InvoiceLineItem`, `ChargeDistribution` (brief section 9)
- **Principes SOLID** : SRP (Expense = workflow facture, InvoiceLineItem = lignes, ChargeDistribution = repartition), OCP (nouveaux taux TVA sans modifier moteur calcul)
- **FR PRD** : FR-009
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux creer une facture de travaux de renovation avec TVA 6% et la soumettre au conseil de copropriete pour approbation.
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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/expense.rs` et `backend/src/domain/entities/invoice_line_item.rs` — workflow Draft->PendingApproval->Approved/Rejected, TVA 6/12/21%, immutabilite apres approbation
  2. [ ] GREEN : Implementer entites
  3. [ ] RED : BDD `backend/tests/features/expenses.feature`, `invoices.feature`, `charge_distribution.feature`
  4. [ ] Ports + Use Cases `backend/src/application/use_cases/expense_use_cases.rs`, `charge_distribution_use_cases.rs`
  5. [ ] Integration + Repositories + Migrations + Handlers
  6. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-002 (UnitOwner pour quotes-parts), STORY-008 (Accounting)
- **Endpoints** : `GET/POST /expenses`, `PUT /expenses/:id/submit-for-approval`, `PUT /expenses/:id/approve`, `POST /invoices/:id/calculate-distribution`

---

### Story 5.2 : Paiements Stripe + SEPA

- **ID** : STORY-012 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Billing & Payments
- **Entite(s) DDD** : `Payment`, `PaymentMethod` (brief section 9)
- **Principes SOLID** : SRP (Payment = transaction, PaymentMethod = moyen), DIP (abstraction Payment adapter hexagonal pour Stripe)
- **Invariants** : INV-7 (idempotency key >= 16 chars, pas de double charge, anti-over-refund)
- **FR PRD** : FR-010
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux payer mes charges trimestrielles par SEPA depuis mon compte bancaire belge, avec la certitude qu'un retry reseau ne creera pas de double charge.
- **Scenarios BDD** :

```gherkin
Scenario: Paiement SEPA reussi avec idempotency (INV-7)
  Given Sophie a un mandat SEPA actif (BE68 5390 0754 7034)
  And la facture trimestrielle est de 450 EUR
  When Sophie initie le paiement avec idempotency_key "pay_2026q1_sophie_3b"
  Then le paiement est cree en statut "Pending"
  And apres confirmation, le statut passe a "Succeeded"

Scenario: Prevenir la double charge par idempotency (INV-7)
  Given un paiement avec idempotency_key "pay_2026q1_sophie_3b" existe deja en statut "Succeeded"
  When un retry reseau envoie le meme paiement avec la meme idempotency_key
  Then le paiement existant est retourne (pas de nouveau paiement cree)

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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/payment.rs` (530 lignes) et `backend/src/domain/entities/payment_method.rs` (273 lignes) — INV-7, lifecycle Pending->Succeeded/Failed, anti-over-refund
  2. [ ] GREEN : Implementer entites Payment, PaymentMethod
  3. [ ] RED : BDD `backend/tests/features/payments.feature`, `payment_methods.feature`
  4. [ ] Ports `backend/src/application/ports/payment_repository.rs` (21 methodes), `payment_method_repository.rs` (13 methodes)
  5. [ ] Use Cases `backend/src/application/use_cases/payment_use_cases.rs` (26 methodes), `payment_method_use_cases.rs` (14 methodes)
  6. [ ] DTOs `backend/src/application/dto/payment_dto.rs`, `payment_method_dto.rs`
  7. [ ] Integration + Repositories + Migration `[timestamp]_create_payments.sql` (2 tables, custom ENUMs, 10 indexes, UNIQUE idempotency_key)
  8. [ ] Handlers `backend/src/infrastructure/web/handlers/payment_handlers.rs` (22 endpoints), `payment_method_handlers.rs` (16 endpoints)
  9. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-011 (Expense), STORY-002 (Owner)
- **Endpoints** : `POST /payments`, `PUT /payments/:id/succeeded`, `POST /payments/:id/refund`, `POST /payment-methods`, `PUT /payment-methods/:id/set-default`

---

### Story 5.3 : Recouvrement automatise 4 niveaux

- **ID** : STORY-013 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Billing & Payments
- **Entite(s) DDD** : `PaymentReminder` (brief section 9)
- **Principes SOLID** : SRP, OCP (ajout niveaux d'escalade)
- **FR PRD** : FR-011
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux que KoproGo envoie automatiquement des relances d'impayes avec escalade progressive, pour atteindre un taux de recouvrement > 85% avant action legale.
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

Scenario: Statistiques de recouvrement
  Given 50 factures impayees ce trimestre pour "Residence du Parc"
  When Marc consulte les statistiques de recouvrement
  Then le dashboard affiche : 35 recouvrees avant Gentle, 10 au niveau Formal, 3 au FinalNotice, 2 en LegalAction
  And le taux de recouvrement avant LegalAction est 96% (48/50)
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/payment_reminder.rs` — 4 niveaux Gentle->Formal->FinalNotice->LegalAction, penalites taux legal belge 8%
  2. [ ] GREEN : Entite PaymentReminder
  3. [ ] RED : BDD `backend/tests/features/payment_recovery.feature`
  4. [ ] Port + Use Case `backend/src/application/use_cases/payment_reminder_use_cases.rs` + Repository + Migration + Handler
  5. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-011 (Expense), STORY-012 (Payment)
- **Endpoints** : `GET/POST /payment-reminders`, `PUT /payment-reminders/:id/escalate`, `GET /payment-reminders/stats`

---

### Story 5.4 : Appels de fonds et contributions

- **ID** : STORY-014 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Billing & Payments
- **Entite(s) DDD** : `CallForFunds`, `OwnerContribution` (brief section 9)
- **Principes SOLID** : SRP
- **FR PRD** : FR-012
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux lancer l'appel de fonds trimestriel de 25 000 EUR pour "Residence du Parc", avec distribution automatique selon les milliemes de chaque coproprietaire.
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
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain + BDD `backend/tests/features/call_for_funds.feature`, `owner_contributions.feature`
  2. [ ] GREEN : Entites CallForFunds, OwnerContribution
  3. [ ] Ports + Use Cases `backend/src/application/use_cases/call_for_funds_use_cases.rs`, `owner_contribution_use_cases.rs`
  4. [ ] Integration + Repositories + Migration + Handlers
  5. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-002 (UnitOwner pour quotes-parts), STORY-012 (Payment)
- **Endpoints** : `POST /call-for-funds`, `POST /call-for-funds/:id/send`, `PUT /owner-contributions/:id/mark-paid`

---

## Epic 6 : Maintenance | SHOULD HAVE

> Bounded Context DDD : Maintenance (brief section 9, architecture section 2.1 #6)
> FRs PRD : FR-013, FR-014
> Jalon : 3

### Story 6.1 : Ticketing maintenance avec SLA

- **ID** : STORY-015 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Maintenance
- **Entite(s) DDD** : `Ticket` (brief section 9)
- **Principes SOLID** : SRP (Ticket gere le workflow maintenance, pas les devis), OCP (nouvelles categories sans modifier Domain)
- **FR PRD** : FR-013
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux signaler une fuite dans le parking via KoproGo, et suivre l'avancement de la reparation en temps reel.
- **Scenarios BDD** :

```gherkin
Scenario: Creer un ticket maintenance avec SLA automatique
  Given Sophie est connectee en tant que coproprietaire
  When Sophie cree un ticket "Fuite d'eau parking P2" avec priorite "High"
  Then le ticket est cree en statut "Open"
  And la due_date est calculee automatiquement a maintenant + 24h (SLA High)

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
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/ticket.rs` (310 lignes) — 6 etats Open->Assigned->InProgress->Resolved->Closed/Cancelled, 5 priorites SLA, due_date auto
  2. [ ] GREEN : Entite Ticket
  3. [ ] RED : BDD `backend/tests/features/tickets.feature`, `ticket_workflow.feature`
  4. [ ] Port `backend/src/application/ports/ticket_repository.rs` (18 methodes) + Use Case `ticket_use_cases.rs` (18 methodes)
  5. [ ] Integration + Repository + Migration `[timestamp]_create_tickets.sql` (custom ENUMs, 8 indexes) + Handler `ticket_handlers.rs` (17 endpoints)
  6. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building), STORY-003 (Identity pour assignation)
- **Endpoints** : `POST /tickets`, `PUT /tickets/:id/assign`, `PUT /tickets/:id/start`, `PUT /tickets/:id/resolve`, `PUT /tickets/:id/close`, `GET /tickets/overdue`, `GET /tickets/statistics`

---

### Story 6.2 : Devis multi-entrepreneurs avec scoring belge

- **ID** : STORY-016 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Maintenance
- **Entite(s) DDD** : `Quote`, `ContractorReport` (brief section 9)
- **Principes SOLID** : SRP (Quote = devis, ContractorReport = rapport), OCP (nouveaux criteres scoring)
- **Invariants** : INV-10 (3 devis minimum pour travaux > 5 000 EUR)
- **FR PRD** : FR-014
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux comparer 3 devis pour les travaux de toiture (45 000 EUR) et obtenir un classement objectif base sur le scoring belge.
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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/quote.rs` (661 lignes) — 7 transitions etat, scoring (prix 40%, delai 30%, garantie 20%, reputation 10%), TVA belge 6%/21%, garantie decennale
  2. [ ] GREEN : Entites Quote, ContractorReport (magic link JWT 72h)
  3. [ ] RED : BDD `backend/tests/features/quotes.feature`, `contractor_reports.feature`
  4. [ ] Ports + Use Cases `backend/src/application/use_cases/quote_use_cases.rs` (20 methodes), `contractor_report_use_cases.rs`
  5. [ ] Integration + Repositories + Migrations + Handlers `quote_handlers.rs` (15 endpoints), `contractor_report_handlers.rs`
  6. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-015 (Ticket), STORY-011 (Expense pour facturation)
- **Endpoints** : `POST /quotes`, `POST /quotes/compare`, `POST /contractor-reports/:id/generate-magic-link`, `GET /contractor-reports/magic/:token`

---

## Epic 7 : GDPR & Compliance | MUST HAVE

> Bounded Context DDD : GDPR & Compliance (brief section 9, architecture section 2.1 #8)
> FR PRD : FR-015
> Jalon : 1 (acheve)

### Story 7.1 : Conformite GDPR complete (Articles 15-21 + 30)

- **ID** : STORY-017 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : GDPR & Compliance
- **Entite(s) DDD** : Operations sur `User` (export, erasure, rectify, restrict, marketing opt-out)
- **Principes SOLID** : SRP (chaque article GDPR = methode use case distincte), OCP (ajout nouveaux droits sans modifier handlers), DIP (GdprRepository + UserRepository traits)
- **FR PRD** : FR-015
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux pouvoir exporter toutes mes donnees personnelles (Art. 15), demander la correction de mon email (Art. 16), ou exercer mon droit a l'oubli (Art. 17).
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
  And Sophie n'a aucune obligation legale en cours
  Then les donnees personnelles sont anonymisees (nom -> "Anonyme_xxxxx", email -> null)
  And les donnees financieres sont conservees (obligation legale 10 ans)

Scenario: Article 17 — Refus d'effacement pour obligation legale
  Given Sophie a un litige en cours avec la copropriete
  When Sophie demande la suppression de son compte
  Then la demande est refusee : "Effacement impossible : obligation legale en cours"

Scenario: Article 18 — Restriction de traitement
  Given Sophie est en litige et demande la restriction de traitement
  When le systeme applique la restriction
  Then processing_restricted = true et processing_restricted_at = maintenant
  And les relances automatiques sont suspendues pour Sophie

Scenario: Article 21 — Opposition au marketing
  Given Sophie ne souhaite plus recevoir de communications marketing
  When Sophie active l'opt-out marketing
  Then marketing_opt_out = true et marketing_opt_out_at = maintenant
  And Sophie ne recoit plus que les notifications legalement obligatoires
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests unitaires domain `backend/src/domain/entities/user.rs` — 11 tests (3 Art.16, 4 Art.18, 3 Art.21, 1 defaults) : rectify_data(), restrict_processing(), set_marketing_opt_out()
  2. [ ] GREEN : Implementer methodes GDPR sur User
  3. [ ] RED : BDD `backend/tests/features/gdpr.feature`, `gdpr_art30.feature`, `consent.feature`
  4. [ ] Port `backend/src/application/ports/gdpr_repository.rs` + Use Case `backend/src/application/use_cases/gdpr_use_cases.rs` (Articles 15-21, audit trail)
  5. [ ] DTOs `backend/src/application/dto/gdpr_dto.rs` (GdprRectifyRequest, GdprRestrictProcessingRequest, GdprMarketingPreferenceRequest, GdprActionResponse)
  6. [ ] Migration `[timestamp]_add_gdpr_complementary_fields.sql` (processing_restricted, marketing_opt_out + partial indexes)
  7. [ ] Handlers `backend/src/infrastructure/web/handlers/gdpr_handlers.rs` (GET /gdpr/export, DELETE /gdpr/erase, PUT /gdpr/rectify, PUT /gdpr/restrict-processing, PUT /gdpr/marketing-preference)
  8. [ ] 7 audit event types (GdprDataRectified, GdprProcessingRestricted, GdprMarketingOptOut, etc.)
  9. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-003 (Identity & Access / User)
- **Endpoints** : `GET /gdpr/export`, `DELETE /gdpr/erase`, `GET /gdpr/can-erase`, `PUT /gdpr/rectify`, `PUT /gdpr/restrict-processing`, `PUT /gdpr/marketing-preference`

---

## Epic 8 : Community | SHOULD HAVE

> Bounded Context DDD : Community (brief section 9, architecture section 2.1 #9)
> FRs PRD : FR-016, FR-017
> Jalon : 3

### Story 8.1 : SEL — Systeme d'Echange Local

- **ID** : STORY-018 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Community
- **Entite(s) DDD** : `LocalExchange`, `OwnerCreditBalance` (brief section 9)
- **Principes SOLID** : SRP (LocalExchange = echange, OwnerCreditBalance = monnaie), OCP (nouveaux types echange sans modifier Domain)
- **FR PRD** : FR-016
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux offrir 2h de babysitting sur le SEL de mon immeuble, et quand un voisin complete l'echange, mes credits sont mis a jour automatiquement.
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
```

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/local_exchange.rs` (661 lignes) et `backend/src/domain/entities/owner_credit_balance.rs` (273 lignes) — 5 etats, 3 types echange (Service/ObjectLoan/SharedPurchase), ratings 1-5, credits 1-100, provider != requester
  2. [ ] GREEN : Implementer entites
  3. [ ] RED : BDD `backend/tests/features/local_exchange.feature`, `sel_workflow.feature`
  4. [ ] Ports `backend/src/application/ports/local_exchange_repository.rs` (18 methodes), `owner_credit_balance_repository.rs` (11 methodes)
  5. [ ] Use Case `backend/src/application/use_cases/local_exchange_use_cases.rs` (20 methodes, multi-repo)
  6. [ ] Integration + Repositories + Migration `[timestamp]_create_local_exchanges.sql` (2 tables, 2 ENUMs, 11 indexes, 6 constraints)
  7. [ ] Handler (17 endpoints)
  8. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building), STORY-002 (Owner)
- **Endpoints** : `POST /exchanges`, `POST /exchanges/:id/request`, `POST /exchanges/:id/complete`, `PUT /exchanges/:id/rate-provider`, `GET /buildings/:id/leaderboard`, `GET /buildings/:id/sel-statistics`

---

### Story 8.2 : Sondages (Polls) entre assemblees

- **ID** : STORY-019 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Community
- **Entite(s) DDD** : `Poll`, `PollOption`, `PollVote` (brief section 9)
- **Principes SOLID** : SRP, OCP (nouveaux types sondage), DIP (PollRepository, PollOptionRepository, PollVoteRepository)
- **FR PRD** : FR-017
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux lancer un sondage "Faut-il repeindre le hall ?" aupres des coproprietaires, et voir les resultats avec le taux de participation.
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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/poll.rs` (572 lignes), `backend/src/domain/entities/poll_option.rs` (188 lignes), `backend/src/domain/entities/poll_vote.rs` (214 lignes) — 4 types (YesNo/MultipleChoice/Rating/OpenEnded), lifecycle Draft->Active->Closed, UNIQUE(poll_id, owner_id)
  2. [ ] GREEN : Implementer entites
  3. [ ] RED : BDD `backend/tests/features/polls.feature`, `poll_workflow.feature` (20 scenarios)
  4. [ ] Ports (PollRepository 16 methodes, PollOptionRepository 12 methodes, PollVoteRepository 10 methodes)
  5. [ ] Use Case `backend/src/application/use_cases/poll_use_cases.rs` (18 methodes)
  6. [ ] Integration + Repositories (PostgresPollRepository 511 lignes) + Migration `[timestamp]_create_polls.sql` (3 tables, 2 ENUMs, 14 indexes, 10 constraints)
  7. [ ] Handler `backend/src/infrastructure/web/handlers/poll_handlers.rs` (12 endpoints)
  8. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building), STORY-002 (UnitOwner pour eligibilite)
- **Endpoints** : `POST /polls`, `PUT /polls/:id/publish`, `POST /polls/:id/vote`, `PUT /polls/:id/close`, `GET /polls/:id/results`

---

## Epic 9 : Notifications | SHOULD HAVE

> Bounded Context DDD : Notifications (brief section 9, architecture section 2.1 #7)
> FR PRD : FR-019
> Jalon : 3

### Story 9.1 : Notifications multi-canal

- **ID** : STORY-020 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Notifications
- **Entite(s) DDD** : `Notification`, `NotificationPreference` (brief section 9)
- **Principes SOLID** : SRP, OCP (ajout canaux sans modifier Domain), DIP (NotificationRepository, NotificationPreferenceRepository)
- **FR PRD** : FR-019
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux recevoir les convocations AG par email et les alertes maintenance urgentes par SMS, mais ne pas etre derangee par push pour les annonces communautaires.
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

- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/notification.rs` et `notification_preference.rs` — 22 types, 4 canaux (Email/SMS/Push/InApp), preferences granulaires
  2. [ ] GREEN : Entites Notification, NotificationPreference
  3. [ ] RED : BDD (scenarios ci-dessus)
  4. [ ] Ports + Use Case `backend/src/application/use_cases/notification_use_cases.rs` (13 methodes)
  5. [ ] Integration + Repositories + Migration `[timestamp]_create_notifications.sql` (2 tables, custom ENUMs, 9 indexes) + Handler `notification_handlers.rs` (11 endpoints)
  6. [ ] E2E + REFACTOR + Verifier
- **Dependances** : Tous les modules (generent des evenements)
- **Endpoints** : `GET /notifications/my`, `GET /notifications/unread`, `PUT /notifications/:id/read`, `PUT /notifications/read-all`, `GET/PUT /notification-preferences/:user_id/:type`

---

## Epic 10 : Gamification | SHOULD HAVE

> Bounded Context DDD : Gamification (brief section 9, architecture section 2.1 #10)
> Jalon : 3

### Story 10.1 : Achievements et Challenges

- **ID** : STORY-021 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Gamification
- **Entite(s) DDD** : `Achievement`, `UserAchievement`, `Challenge`, `ChallengeProgress` (brief section 9)
- **Principes SOLID** : SRP (Achievement = definition, Challenge = defi temporel), OCP (nouvelles categories/tiers)
- **User Story** : En tant que Sophie (coproprietaire, brief section 6.2), je veux gagner des badges et points en participant aux echanges SEL, sondages et reservations, et voir mon classement sur le leaderboard de l'immeuble.
- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/achievement.rs`, `challenge.rs`, `user_achievement.rs`, `challenge_progress.rs` — 8 categories, 5 tiers, state machine Challenge (Draft->Active->Completed/Cancelled), auto-completion
  2. [ ] GREEN : Entites
  3. [ ] RED : BDD `backend/tests/features/gamification.feature`
  4. [ ] Ports (4 traits, 35 methodes total) + Use Cases (3 classes, 28 methodes)
  5. [ ] Integration + Repositories + Migration `[timestamp]_create_gamification.sql` (4 tables, 4 ENUMs, 17 indexes, 3 triggers)
  6. [ ] Handler `backend/src/infrastructure/web/handlers/gamification_handlers.rs` (22 endpoints)
  7. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-018 (SEL), STORY-019 (Polls)
- **Endpoints** : `POST /achievements`, `POST /users/achievements`, `POST /challenges`, `POST /challenges/:id/progress/increment`, `GET /users/:id/gamification/stats`, `GET /organizations/:id/gamification/leaderboard`

---

## Epic 11 : Board Management | SHOULD HAVE

> Bounded Context DDD : Board Management (architecture section 2.1 #13)
> Jalon : 3

### Story 11.1 : Conseil de copropriete et decisions post-AG

- **ID** : STORY-022 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Board Management
- **Entite(s) DDD** : `BoardMember`, `BoardDecision`
- **Principes SOLID** : SRP (BoardMember = mandat, BoardDecision = suivi decisions)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux enregistrer les membres du conseil de copropriete elus en AG et suivre l'avancement des decisions votees.
- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/board_member.rs`, `board_decision.rs` — positions (President, VicePresident, Treasurer, Secretary, Member), statuts decisions (Pending/InProgress/Completed/Cancelled/Overdue)
  2. [ ] GREEN : Entites
  3. [ ] RED : BDD `backend/tests/features/board.feature`, `board_members.feature`, `board_decisions.feature`, `board_dashboard.feature`
  4. [ ] Ports + Use Cases + Repositories + Handlers + Migration
  5. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building), STORY-005 (AG)
- **Endpoints** : `POST /board-members`, `GET /buildings/:id/board-members/active`, `POST /board-decisions`, `GET /buildings/:id/board-decisions/overdue`, `GET /buildings/:id/board-decisions/stats`

---

## Epic 12 : Documents | SHOULD HAVE

> Bounded Context DDD : Documents (architecture section 2.1 #11)

### Story 12.1 : Gestion documentaire

- **ID** : STORY-023 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Documents
- **Entite(s) DDD** : `Document`
- **Principes SOLID** : SRP, OCP (ajout types documents), DIP (DocumentRepository)
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux uploader les PV d'AG, factures et rapports d'inspection dans KoproGo pour les partager avec les coproprietaires.
- **Taches techniques TDD** :
  1. [ ] RED : Tests domain `backend/src/domain/entities/document.rs`
  2. [ ] GREEN : Entite Document (title, file_path, document_type, max 50MB multipart)
  3. [ ] RED : BDD `backend/tests/features/documents.feature`, `documents_delete.feature`, `documents_expenses.feature`, `documents_linking.feature`
  4. [ ] Port + Use Case + Repository + Handler + Migration
  5. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building)
- **Endpoints** : `POST /documents` (multipart), `GET /documents/:id/download`, `PUT /documents/:id/link-meeting`, `PUT /documents/:id/link-expense`

---

## Epic 13 : Energy & IoT | COULD HAVE

> Bounded Context DDD : Energy & IoT (architecture section 2.1 #12)
> Jalon : 4

### Story 13.1 : Achats groupes d'energie

- **ID** : STORY-024 | **Type** : Feature | **Taille** : L
- **Bounded Context DDD** : Energy & IoT
- **Entite(s) DDD** : `EnergyCampaign`, `ProviderOffer`, `EnergyBillUpload`
- **Principes SOLID** : SRP, DIP
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux lancer une campagne d'achat groupe d'energie pour "Residence du Parc" afin de negocier des tarifs reduits pour les coproprietaires.
- **Taches techniques TDD** :
  1. [ ] Domain (entities + tests) + BDD `backend/tests/features/energy_campaigns.feature`
  2. [ ] Ports + Use Cases + Repositories + Handlers + Migration
  3. [ ] GDPR compliance : k-anonymite >= 5 participants, consent GDPR Art. 7.3, chiffrement bills
  4. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001, STORY-017 (GDPR)
- **Endpoints** : `POST /energy-campaigns`, `POST /energy-bills/upload`, `POST /energy-campaigns/:id/select-offer`, `GET /energy-campaigns/:id/stats`

---

### Story 13.2 : IoT Smart Meters et Linky

- **ID** : STORY-025 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Energy & IoT
- **Entite(s) DDD** : `IoTReading`, `LinkyDevice`
- **User Story** : En tant que Marc (syndic, brief section 6.1), je veux configurer le compteur Linky de l'immeuble et visualiser les statistiques de consommation electrique pour detecter les anomalies.
- **Taches techniques TDD** :
  1. [ ] Domain (IoTReading, LinkyDevice) + BDD `backend/tests/features/iot.feature`
  2. [ ] Ports + Use Cases + Repositories + Handlers + Migration
  3. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-001 (Building)
- **Endpoints** : `POST /iot/readings`, `POST /iot/readings/bulk`, `POST /iot/linky/devices`, `GET /iot/buildings/:id/consumption/stats`, `GET /iot/buildings/:id/consumption/anomalies`

---

## Epic 14 : SuperAdmin & Dashboard | MUST HAVE

> Bounded Context DDD : Identity & Access (administration)
> Jalon : 2

### Story 14.1 : Gestion organisations et utilisateurs (SuperAdmin)

- **ID** : STORY-026 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Identity & Access
- **Entite(s) DDD** : `Organization`, `User` (administration)
- **Principes SOLID** : SRP
- **FR PRD** : FR-018 (composante administration)
- **User Story** : En tant que SuperAdmin, je veux creer et gerer les organisations (syndics) et leurs utilisateurs pour administrer la plateforme multi-tenant.
- **Taches techniques TDD** :
  1. [ ] Domain entities (Organization avec plans: free/starter/professional/enterprise)
  2. [ ] BDD `backend/tests/features/organizations.feature`, `backend/tests/features/multitenancy.feature`
  3. [ ] Handlers `backend/src/infrastructure/web/handlers/organization_handlers.rs`, `user_handlers.rs`
  4. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-003
- **Endpoints** : `GET/POST /organizations`, `PUT /organizations/:id/activate`, `PUT /organizations/:id/suspend`, `GET/POST /users`, `PUT /users/:id/activate`

---

### Story 14.2 : Dashboard comptable

- **ID** : STORY-027 | **Type** : Feature | **Taille** : S
- **Bounded Context DDD** : Accounting (vue dashboard)
- **FR PRD** : FR-006 (composante reporting)
- **User Story** : En tant que Jean-Pierre (comptable, brief section 6.3), je veux un dashboard synthetique avec les statistiques financieres et les transactions recentes.
- **Taches techniques TDD** :
  1. [ ] Use Case `backend/src/application/use_cases/dashboard_use_cases.rs`
  2. [ ] BDD `backend/tests/features/dashboard.feature`
  3. [ ] Handler `backend/src/infrastructure/web/handlers/dashboard_handlers.rs`
  4. [ ] E2E + REFACTOR + Verifier
- **Dependances** : STORY-008 (Accounting)
- **Endpoints** : `GET /dashboard/accountant/stats`, `GET /dashboard/accountant/transactions`

---

## Suggestion de planification Sprints

### Sprint 1 — Core Domain (TDD)

| Priorite | Story | Epic | Taille | FR PRD |
|----------|-------|------|--------|--------|
| 1 | STORY-001 | Epic 1 (Building) | M | FR-001 |
| 2 | STORY-002 | Epic 1 (Building) | L | FR-002 |
| 3 | STORY-003 | Epic 2 (Identity) | L | FR-018 |

### Sprint 2 — Assembly & Accounting (TDD + BDD)

| Priorite | Story | Epic | Taille | FR PRD |
|----------|-------|------|--------|--------|
| 4 | STORY-004 | Epic 3 (AG) | L | FR-003 |
| 5 | STORY-005 | Epic 3 (AG) | L | FR-004 |
| 6 | STORY-008 | Epic 4 (Comptabilite) | L | FR-006 |
| 7 | STORY-009 | Epic 4 (Comptabilite) | M | FR-007 |

### Sprint 3 — Billing & Payments (TDD + BDD)

| Priorite | Story | Epic | Taille | FR PRD |
|----------|-------|------|--------|--------|
| 8 | STORY-011 | Epic 5 (Billing) | L | FR-009 |
| 9 | STORY-012 | Epic 5 (Billing) | L | FR-010 |
| 10 | STORY-013 | Epic 5 (Billing) | M | FR-011 |
| 11 | STORY-014 | Epic 5 (Billing) | M | FR-012 |

### Sprint 4 — GDPR + Maintenance (TDD + BDD)

| Priorite | Story | Epic | Taille | FR PRD |
|----------|-------|------|--------|--------|
| 12 | STORY-017 | Epic 7 (GDPR) | L | FR-015 |
| 13 | STORY-015 | Epic 6 (Maintenance) | L | FR-013 |
| 14 | STORY-016 | Epic 6 (Maintenance) | L | FR-014 |

### Sprint 5 — Community + Notifications (TDD + BDD + E2E)

| Priorite | Story | Epic | Taille | FR PRD |
|----------|-------|------|--------|--------|
| 15 | STORY-018 | Epic 8 (Community) | L | FR-016 |
| 16 | STORY-019 | Epic 8 (Community) | L | FR-017 |
| 17 | STORY-020 | Epic 9 (Notifications) | M | FR-019 |
| 18 | STORY-021 | Epic 10 (Gamification) | L | — |

### Sprint 6 — Complement (Frontend + E2E)

| Priorite | Story | Epic | Taille |
|----------|-------|------|--------|
| 19 | STORY-006 | Epic 3 (AG - AGE) | M |
| 20 | STORY-007 | Epic 3 (AG - Visio) | M |
| 21 | STORY-010 | Epic 4 (Etat date) | M |
| 22 | STORY-022 | Epic 11 (Board) | M |
| 23 | STORY-023 | Epic 12 (Documents) | M |

### Sprint 7+ — Post-MVP (COULD HAVE)

| Priorite | Story | Epic | Taille |
|----------|-------|------|--------|
| 24 | STORY-024 | Epic 13 (Energy) | L |
| 25 | STORY-025 | Epic 13 (IoT) | M |
| 26 | STORY-026 | Epic 14 (SuperAdmin) | M |
| 27 | STORY-027 | Epic 14 (Dashboard) | S |

---

## Matrice de tracabilite FR -> Stories

| FR PRD | Description | Stories | Epic | Invariants |
|--------|-------------|---------|------|------------|
| FR-001 | Gestion immeubles (CRUD) | STORY-001 | Epic 1 | INV-9 |
| FR-002 | Lots et quotes-parts | STORY-002 | Epic 1 | INV-1, INV-6 |
| FR-003 | Convocations legales 15j | STORY-004 | Epic 3 | INV-3 |
| FR-004 | Vote numerique majorites | STORY-005, STORY-007 | Epic 3 | INV-2, INV-4 |
| FR-005 | AGE par petition | STORY-006 | Epic 3 | INV-8 |
| FR-006 | Comptabilite PCMN | STORY-008 | Epic 4 | INV-5 |
| FR-007 | Budget annuel et variance | STORY-009 | Epic 4 | — |
| FR-008 | Etat date ventes | STORY-010 | Epic 4 | — |
| FR-009 | Facturation TVA belge | STORY-011 | Epic 5 | — |
| FR-010 | Paiements Stripe + SEPA | STORY-012 | Epic 5 | INV-7 |
| FR-011 | Recouvrement 4 niveaux | STORY-013 | Epic 5 | — |
| FR-012 | Appels de fonds | STORY-014 | Epic 5 | — |
| FR-013 | Ticketing SLA | STORY-015 | Epic 6 | — |
| FR-014 | Devis scoring belge | STORY-016 | Epic 6 | INV-10 |
| FR-015 | GDPR Articles 15-21+30 | STORY-017 | Epic 7 | — |
| FR-016 | SEL echanges locaux | STORY-018 | Epic 8 | — |
| FR-017 | Sondages (Polls) | STORY-019 | Epic 8 | — |
| FR-018 | Auth multi-role + 2FA | STORY-003, STORY-026 | Epic 2, Epic 14 | — |
| FR-019 | Notifications multi-canal | STORY-020 | Epic 9 | — |

---

## Stories de scaling (activees quand necessaire)

### Nexus (declencheur : 3+ flux paralleles)

| ID | Titre | Declencheur |
|----|-------|-------------|
| STORY-N01 | Setup Nexus : backlog unifie + Daily Nexus | 3+ flux paralleles sur bounded contexts differents |
| STORY-N02 | Cross-team refinement process | Dependances inter-flux (ex: Epic 5 depend d'Epic 4) |
| STORY-N03 | Nexus Sprint Review unifiee | Plusieurs equipes livrent en parallele |

### SAFe (declencheur : 50+ agents)

| ID | Titre | Declencheur |
|----|-------|-------------|
| STORY-S01 | Setup SAFe : PI Planning + ART (Agile Release Train) | 50+ agents IA actifs |
| STORY-S02 | Architectural Runway + Enabler Stories | Features cross-cutting (ex: migration ScyllaDB) |
| STORY-S03 | Portfolio Kanban + Value Stream Mapping | Alignement strategique multi-epics |

---

## Stories ITIL (activees en pre-production)

| ID | Titre | Phase ITIL | Declencheur |
|----|-------|-----------|-------------|
| STORY-I01 | Runbooks incidents (niveaux 1-2-3) | Incident Management | Premiere mise en production |
| STORY-I02 | Change management : PR + terraform plan + review | Change Management | CI/CD operationnel |
| STORY-I03 | Release management : semantic versioning + ArgoCD | Release Management | Premier tag de release |
| STORY-I04 | Backup restore test automatise (GPG + S3) | Continuity Management | Donnees production critiques |
| STORY-I05 | Capacity planning (Prometheus + alertes seuils) | Capacity Management | 100+ coproprietes hebergees |
| STORY-I06 | Security audit continu (Lynis > 80/100, rkhunter, AIDE) | Security Management | Pre-production |
| STORY-I07 | SLA monitoring (uptime 99.9%, P1 incidents = 0/mois) | Service Level Management | Production |

---

*Methode Maury — Phase TOGAF E (Solutions) — Par Gilles Maury & Farah Maury*
*Agent BMAD : Bob (Scrum Master) — 29/03/2026*

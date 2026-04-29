# PRD — KoproGo Bugfix UI v0.1.0

## Methode Maury — Phase TOGAF B-C (Business + SI)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : John (Product Manager)
**Date** : 01/04/2026
**Version** : 1.0
**Source** : `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` (01/04/2026, Claude — Revue UI)
**PRD parent** : `Maury/PRD.md` v1.0 (29/03/2026, John — Product Manager)

**Stack** : Rust 1.75+ / Actix-web 4.9 + Astro 4.x / Svelte 4.x + PostgreSQL 15
**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Organisation** : Scrum → Nexus → SAFe → ITIL
**Dev** : Agents IA supervises par Gilles Maury

---

## 1. Resume executif

La revue UI humaine de KoproGo v0.1.0 (01/04/2026, 3h, Chrome/localhost) a revele **16 bugs** (4 critiques, 4 majeurs, 8 mineurs) empechant l'utilisation des workflows centraux (AG, tickets, votes) via l'interface utilisateur, alors que le backend fonctionne correctement lorsqu'appele directement via API.

**Verdict** : NO-GO pour release publique — GO conditionnel pour beta privee fermee.

Ce PRD cadre le sprint correctif necessaire pour passer du statut NO-GO a GO beta publique. Il couvre exclusivement les bugs d'integration frontend ↔ backend identifies lors de la revue, organises en 4 phases de correction avec dependances explicites.

**Tracabilite** : Chaque BFR (Bug Fix Requirement) trace vers un BUG-ID du rapport de revue, le FR du PRD parent concerne, et les fichiers source impactes.

---

## 2. Objectifs produit (mesurables)

| # | Objectif | Metrique cible | Echeance | Tracabilite PRD parent |
|---|----------|----------------|----------|----------------------|
| BO1 | Zero bug critique UI | 0/4 bugs critiques restants | Fin Phase 1 | O3 (beta publique) |
| BO2 | Zero bug majeur UI | 0/4 bugs majeurs restants | Fin Phase 2 | O3 (beta publique) |
| BO3 | Workflows AG testables via UI | WF1-6 passants en revue | Fin Phase 2 | O1 (conformite legale) |
| BO4 | i18n ICU resolu a 100% | 0 cle non interpolee | Fin Phase 3 | O3 (beta publique) |
| BO5 | Feedback erreurs API visible | Toast sur 100% des erreurs 4xx/5xx | Fin Phase 0 | O3 (UX beta) |
| BO6 | Isolation donnees par role | Owner ne voit que ses lots | Fin Phase 2 | O5 (GDPR) |

---

## 3. Perimetre

### 3.1 In scope (sprint correctif)

| Module | Bugs | Priorite | Phase |
|--------|------|----------|-------|
| Infrastructure toast erreurs API | Transversal | Prealable | 0 |
| Formulaire creation reunion | BUG-WF1-1 | CRITIQUE | 1 |
| Formulaire creation convocation | BUG-WF1-2 | CRITIQUE | 1 |
| Contrainte DB voting_power | BUG-WF2-1 | CRITIQUE | 1 |
| Formulaire creation ticket | BUG-WF7-1 | CRITIQUE | 1 |
| Liste convocations UI | BUG-WF1-3 | MAJEUR | 2 |
| Affichage NaN% votes | BUG-WF2-2 | MAJEUR | 2 |
| Redirection login SuperAdmin | BUG-WF14-1 | MAJEUR | 2 |
| Isolation donnees par role | BUG-WF14-2 | MAJEUR | 2 |
| i18n ICU MessageFormat | BUG-WF1-4, BUG-SEL-1 | MINEUR | 3 |
| Titres GDPR en anglais | BUG-GDPR-1 | MINEUR | 3 |
| Bouton Voter absent | BUG-WF2-3 | MINEUR | 3 |
| Copyright footer | BUG-WF14-4 | MINEUR | 3 |
| Counter immeubles incoherent | BUG-WF14-3 | MINEUR | 3 |
| Format date income-statement | BUG-WF10-1 | MINEUR | 3 |
| Message rate limiting login | BUG-RL-1 | MINEUR | 3 |

### 3.2 Hors scope

- Nouvelles fonctionnalites (pas de feature, uniquement des corrections)
- Refactoring backend (architecture hexagonale intacte)
- Tests de performance (P99, throughput)
- Deploiement production (correctifs valides en dev/localhost uniquement)
- Workflows WF3-6 (seront retestes apres correction WF1, dans un sprint suivant)

---

## 4. Glossaire metier (complement au PRD parent)

| Terme | Definition | Contexte bugfix |
|-------|------------|-----------------|
| **Toast** | Notification ephemere dans l'UI (3-5s) informant l'utilisateur du resultat d'une action | Systeme existant (`stores/toast.ts`) non branche sur les erreurs API |
| **ICU MessageFormat** | Standard de formatage de messages internationaux avec placeholders (`{count}`, `{hours}`) | Cles non interpolees dans le frontend Svelte |
| **Isolation par role** | Principe RBAC : un utilisateur ne voit que les donnees auxquelles son role lui donne acces | Bug : owner voit tous les immeubles au lieu de ses lots uniquement |
| **Tantieme / Dix-millieme** | (cf. PRD parent, section 4) Quote-part de copropriete, base du pouvoir de vote | Contrainte DB limitee a 1000 alors que certains lots ont >1000 tantiemes |

---

## 5. Bounded Contexts impactes

| Bounded Context | Bugs concernes | Couche impactee |
|-----------------|----------------|-----------------|
| **General Assembly** | BUG-WF1-1, WF1-2, WF1-3, WF2-1, WF2-2, WF2-3 | Frontend (5), Backend migration (1) |
| **Identity & Access** | BUG-WF14-1, WF14-2, BUG-RL-1 | Frontend (2), Backend filtrage (1) |
| **Maintenance** | BUG-WF7-1 | Frontend |
| **GDPR & Compliance** | BUG-GDPR-1 | Frontend (i18n) |
| **Community** | BUG-SEL-1 | Frontend (i18n) |
| **Accounting** | BUG-WF10-1 | Backend (parsing date) |
| **Transversal (UX)** | BUG-WF1-4, WF14-3, WF14-4, Toast | Frontend |

---

## 6. Exigences fonctionnelles (Bug Fix Requirements)

---

### Phase 0 : Infrastructure Transversale (prealable)

#### BFR-000 : Feedback erreurs API via toast

- **Description** : Brancher le systeme de toast existant (`frontend/src/stores/toast.ts`, `frontend/src/components/ui/ToastContainer.svelte`) sur les reponses HTTP 4xx/5xx de l'API afin qu'aucune erreur ne soit silencieuse.
- **Priorite** : PREALABLE (debloque la correction de tous les bugs "silencieux")
- **Source** : Observation transversale revue UI — toutes les erreurs 400 sont avalees sans feedback
- **FR PRD parent** : Transversal (UX)
- **User Story** : En tant que Marc (syndic), quand je soumets un formulaire et que l'API retourne une erreur 400/500, je veux voir un message d'erreur clair dans un toast rouge, afin de comprendre ce qui a echoue et agir en consequence.
- **Fichiers impactes** :
  - `frontend/src/stores/toast.ts` (existant, 54 lignes)
  - `frontend/src/components/ui/ToastContainer.svelte` (existant)
  - Tous les composants effectuant des appels API (fetch/POST/PUT)
- **Scenarios BDD** :

```gherkin
Scenario: Afficher un toast d'erreur sur reponse 400
  Given Marc est connecte en tant que syndic
  And le formulaire de creation de convocation est ouvert
  When Marc soumet le formulaire sans building_id (champ obligatoire)
  Then l'API retourne un code 400 avec message "building_id is required"
  And un toast rouge s'affiche avec le message "building_id is required"
  And le toast disparait apres 5 secondes

Scenario: Afficher un toast d'erreur sur reponse 500
  Given Marc est connecte en tant que syndic
  When une erreur serveur se produit
  Then un toast rouge s'affiche avec "Erreur serveur. Veuillez reessayer."
  And l'erreur est loguee dans la console navigateur

Scenario: Afficher un toast de succes apres action reussie
  Given Marc est connecte en tant que syndic
  When Marc cree un ticket avec succes (code 201)
  Then un toast vert s'affiche avec "Ticket cree avec succes"
```

- **Dependances** : Aucune (prealable a tout)
- **Critere d'acceptation** : Aucune reponse 4xx/5xx ne reste sans feedback visuel dans l'UI

---

### Phase 1 : Bugs Critiques (bloquants release)

#### BFR-001 : Bouton "Nouvelle reunion" manquant sur /meetings

- **Description** : Ajouter un bouton de creation de reunion (AG) sur la page `/meetings`, visible uniquement pour le role syndic. Ce bouton ouvre un modal de creation avec les champs obligatoires (titre, date, type AG, building_id).
- **Priorite** : CRITIQUE
- **Bug ID** : BUG-WF1-1
- **FR PRD parent** : FR-003 (Convocations legales)
- **Impact** : Impossible de creer une AG via l'UI → bloque en cascade WF1-6
- **User Story** : En tant que Marc (syndic), je veux un bouton "Nouvelle reunion" sur la page /meetings afin de creer une AG directement depuis l'interface, sans passer par l'API.
- **Fichiers impactes** :
  - `frontend/src/pages/meetings.astro` (l.1-28) — ajouter bouton
  - `frontend/src/components/MeetingList.svelte` (l.110-173) — ou integrer dans le composant
  - `frontend/src/components/MeetingCreateModal.svelte` (a creer si inexistant)
- **Scenarios BDD** :

```gherkin
Scenario: Bouton "Nouvelle reunion" visible pour le syndic
  Given Marc est connecte en tant que syndic
  When Marc accede a la page /meetings
  Then un bouton "Nouvelle reunion" est visible en haut de page

Scenario: Bouton "Nouvelle reunion" invisible pour le coproprietaire
  Given Sophie est connectee en tant que coproprietaire
  When Sophie accede a la page /meetings
  Then aucun bouton "Nouvelle reunion" n'est affiche

Scenario: Creer une AG via le modal
  Given Marc est connecte en tant que syndic
  And Marc a clique sur "Nouvelle reunion"
  When Marc remplit le titre "AG Annuelle 2026", la date "20/03/2026", le type "Ordinary", et selectionne "Residence du Parc"
  And Marc valide le formulaire
  Then l'AG "AG Annuelle 2026" est creee avec succes
  And un toast vert confirme la creation
  And la reunion apparait dans la liste
```

- **Dependances** : BFR-000 (toast pour feedback)
- **Debloque** : BFR-002 (convocations), puis en cascade WF3-6

---

#### BFR-002 : POST /convocations omet building_id

- **Description** : Le composant `ConvocationPanel.svelte` ne transmet pas le `building_id` dans le body du POST /convocations, provoquant une erreur 400 silencieuse du backend.
- **Priorite** : CRITIQUE
- **Bug ID** : BUG-WF1-2
- **FR PRD parent** : FR-003 (Convocations legales), INV-3 (delai legal 15 jours)
- **Impact** : Impossible de creer une convocation via l'UI
- **User Story** : En tant que Marc (syndic), quand je cree une convocation pour une AG, je veux que le building_id soit automatiquement inclus dans la requete, afin que la convocation soit correctement rattachee a l'immeuble.
- **Fichiers impactes** :
  - `frontend/src/components/convocations/ConvocationPanel.svelte` (l.49-62) — verifier passage building_id
  - `frontend/src/lib/api/convocations.ts` (l.67-73) — verifier payload POST
- **Scenarios BDD** :

```gherkin
Scenario: Creer une convocation avec building_id transmis
  Given Marc est connecte en tant que syndic
  And l'AG "AG Annuelle 2026" existe pour "Residence du Parc"
  When Marc cree la convocation pour cette AG
  Then le POST /convocations contient building_id dans le body
  And la convocation est creee avec succes (code 201)
  And un toast vert confirme la creation

Scenario: Erreur explicite si building_id manquant
  Given Marc est connecte en tant que syndic
  When le frontend envoie un POST /convocations sans building_id
  Then l'API retourne un code 400 avec message "building_id is required"
  And un toast rouge affiche l'erreur
```

- **Dependances** : BFR-000 (toast), BFR-001 (reunion doit exister)

---

#### BFR-003 : Contrainte DB voting_power <= 1000 vs tantiemes reels

- **Description** : La contrainte DB et le frontend limitent `voting_power` a 1000, mais le modele de donnees du seed comporte des lots avec >1000 tantiemes (Emmanuel = 1280 dix-milliemes). La limite doit etre relevee a 10000 (dix-milliemes) pour couvrir tous les cas legaux belges.
- **Priorite** : CRITIQUE
- **Bug ID** : BUG-WF2-1
- **FR PRD parent** : FR-004 (Vote numerique majorites legales), INV-4
- **Impact** : Certains coproprietaires ne peuvent pas voter — violation conformite legale belge
- **User Story** : En tant que Emmanuel (coproprietaire avec 1280 dix-milliemes), je veux pouvoir voter lors de l'AG avec mon pouvoir de vote reel, sans etre bloque par une limite technique.
- **Fichiers impactes** :
  - `backend/migrations/20251115120000_create_resolutions_and_votes.sql` — ALTER contrainte
  - `backend/src/domain/entities/vote.rs` — ajuster validation
  - `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.225) — `max="1000"` → `max="10000"`
- **Scenarios BDD** :

```gherkin
Scenario: Voter avec voting_power > 1000 dix-milliemes
  Given Emmanuel possede le lot penthouse avec 1280 dix-milliemes
  And la resolution "Travaux de toiture" requiert une majorite Absolute
  When Emmanuel vote "Pour" avec voting_power = 1280
  Then le vote est enregistre avec succes
  And le voting_power 1280 est comptabilise dans le calcul de majorite

Scenario: Refuser voting_power > 10000 dix-milliemes
  When un vote est soumis avec voting_power = 15000
  Then une erreur "Voting power must be between 0 and 10000" est retournee
```

- **Dependances** : Aucune (bug mixte backend + frontend, independant)
- **Note** : Necessite une migration SQL ALTER + modification du domain entity

---

#### BFR-004 : Formulaire ticket envoie body malformate

- **Description** : Le composant `TicketCreateModal.svelte` envoie un body JSON qui ne correspond pas au `CreateTicketDto` attendu par le backend, provoquant une erreur 400 silencieuse.
- **Priorite** : CRITIQUE
- **Bug ID** : BUG-WF7-1
- **FR PRD parent** : FR-013 (Ticketing SLA)
- **Impact** : Impossible de creer un ticket de maintenance via l'UI
- **User Story** : En tant que Sophie (coproprietaire), je veux signaler une fuite d'eau dans mon appartement en creant un ticket depuis l'interface, avec la priorite, la categorie et la description.
- **Fichiers impactes** :
  - `frontend/src/components/tickets/TicketCreateModal.svelte` (l.70-90) — corriger payload
- **Scenarios BDD** :

```gherkin
Scenario: Creer un ticket de maintenance via l'UI
  Given Sophie est connectee en tant que coproprietaire
  And Sophie a ouvert le formulaire de creation de ticket
  When Sophie remplit :
    | Champ       | Valeur                     |
    | title       | Fuite d'eau salle de bain  |
    | description | Fuite sous le lavabo       |
    | priority    | High                       |
    | category    | Plumbing                   |
    | building_id | (Residence du Parc)        |
  And Sophie soumet le formulaire
  Then le ticket est cree avec succes (code 201)
  And un toast vert confirme "Ticket cree avec succes"
  And le ticket apparait dans la liste /tickets

Scenario: Erreur claire si champ obligatoire manquant
  Given Sophie soumet le formulaire de ticket sans titre
  Then un toast rouge affiche "title is required"
  And le formulaire n'est pas soumis
```

- **Dependances** : BFR-000 (toast pour feedback)

---

### Phase 2 : Bugs Majeurs (avant beta)

#### BFR-005 : Convocations creees non listees dans l'UI

- **Description** : Les convocations creees (via API ou formulaire) n'apparaissent pas dans la liste du composant `ConvocationList.svelte`. Probablement un probleme de transmission du `building_id` au composant liste, ou de mapping reponse API.
- **Priorite** : MAJEUR
- **Bug ID** : BUG-WF1-3
- **FR PRD parent** : FR-003 (Convocations legales)
- **Impact** : Le syndic ne voit pas ses convocations creees
- **User Story** : En tant que Marc (syndic), apres avoir cree une convocation, je veux la voir apparaitre dans la liste des convocations de l'immeuble concerne.
- **Fichiers impactes** :
  - `frontend/src/pages/convocations.astro` (l.25-57) — BuildingSelector + mounting
  - `frontend/src/components/convocations/ConvocationList.svelte` (l.1-157) — fetch + mapping
- **Scenarios BDD** :

```gherkin
Scenario: Convocation visible apres creation
  Given Marc a cree une convocation pour l'AG du 20/03/2026 de "Residence du Parc"
  When Marc accede a la page /convocations
  And Marc selectionne "Residence du Parc" dans le building selector
  Then la convocation pour l'AG du 20/03/2026 apparait dans la liste
  And le statut "Draft" est affiche

Scenario: Liste vide si aucune convocation
  Given "Residence du Parc" n'a aucune convocation
  When Marc accede a la page /convocations et selectionne "Residence du Parc"
  Then un message "Aucune convocation" est affiche
```

- **Dependances** : BFR-001, BFR-002 (les convocations doivent pouvoir etre creees d'abord)

---

#### BFR-006 : NaN% dans les compteurs de vote

- **Description** : La fonction `getVotePercentage()` dans `ResolutionVotePanel.svelte` retourne NaN quand `totalVotes = 0` (division par zero). Les compteurs affichent "NaN%" au lieu de "0%".
- **Priorite** : MAJEUR
- **Bug ID** : BUG-WF2-2
- **FR PRD parent** : FR-004 (Vote numerique majorites legales)
- **Impact** : Interface de vote inutilisable — affichage corrompu
- **User Story** : En tant que Marc (syndic), quand je consulte une resolution sans votes encore, je veux voir "0%" affiche proprement, et non "NaN%".
- **Fichiers impactes** :
  - `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.44-47) — garde division par zero
  - Lignes d'affichage : l.141, l.151, l.161
- **Scenarios BDD** :

```gherkin
Scenario: Afficher 0% quand aucun vote n'est enregistre
  Given la resolution "Travaux de toiture" est en statut "Pending"
  And aucun vote n'a ete enregistre
  When Marc consulte les compteurs de vote
  Then les compteurs affichent "Pour: 0%", "Contre: 0%", "Abstention: 0%"
  And aucun "NaN" n'apparait dans l'interface

Scenario: Pourcentages corrects apres votes
  Given la resolution "Travaux de toiture" a recu :
    | Choix       | Voting Power |
    | Pour        | 450          |
    | Contre      | 600          |
    | Abstention  | 200          |
  When Marc consulte les compteurs
  Then les pourcentages affiches sont : "Pour: 36.0%", "Contre: 48.0%", "Abstention: 16.0%"
```

- **Dependances** : Aucune (correction isolee)

---

#### BFR-007 : SuperAdmin login ne redirige pas vers /admin

- **Description** : Apres login reussi en tant que SuperAdmin, l'utilisateur reste sur la page /login au lieu d'etre redirige vers /admin. Le mapping role → URL existe dans `LoginForm.svelte` (l.43-49) mais la redirection ne s'execute pas.
- **Priorite** : MAJEUR
- **Bug ID** : BUG-WF14-1
- **FR PRD parent** : FR-018 (Authentification multi-role)
- **Impact** : Experience admin degradee — navigation manuelle requise
- **User Story** : En tant que SuperAdmin, apres m'etre connecte, je veux etre redirige automatiquement vers le dashboard /admin.
- **Fichiers impactes** :
  - `frontend/src/components/LoginForm.svelte` (l.42-49) — debugger flux redirection
- **Scenarios BDD** :

```gherkin
Scenario: Redirection automatique apres login SuperAdmin
  Given le SuperAdmin saisit ses identifiants corrects
  When le login reussit avec role "superadmin"
  Then l'utilisateur est redirige vers /admin
  And le dashboard SuperAdmin est affiche

Scenario: Redirection automatique apres login Syndic
  Given Marc saisit ses identifiants corrects
  When le login reussit avec role "syndic"
  Then Marc est redirige vers /syndic

Scenario: Redirection automatique apres login Coproprietaire
  Given Sophie saisit ses identifiants corrects
  When le login reussit avec role "owner"
  Then Sophie est redirigee vers /buildings

Scenario: Redirection automatique apres login Comptable
  Given Gisele saisit ses identifiants corrects
  When le login reussit avec role "accountant"
  Then Gisele est redirigee vers /accountant
```

- **Dependances** : Aucune (correction isolee)

---

#### BFR-008 : Isolation donnees — Alice voit 3 immeubles au lieu de 1

- **Description** : Le composant `BuildingSelector.svelte` appelle `GET /buildings?per_page=100` sans filtre role/organisation. Un coproprietaire voit tous les immeubles de l'organisation au lieu de ceux ou il possede des lots. Ce bug est probablement backend (filtrage insuffisant dans le handler ou use case).
- **Priorite** : MAJEUR
- **Bug ID** : BUG-WF14-2 (+ BUG-WF14-3 counter incoherent, consequence directe)
- **FR PRD parent** : FR-018 (Identity & Access), FR-015 (GDPR — minimisation donnees)
- **Impact** : Fuite de donnees entre roles — violation GDPR (principe de minimisation)
- **User Story** : En tant que Sophie (coproprietaire du lot 2A), je ne veux voir que l'immeuble "Residence du Parc" ou je suis proprietaire, pas les autres immeubles geres par le syndic.
- **Fichiers impactes** :
  - `frontend/src/components/BuildingSelector.svelte` (l.32-57) — filtre cote frontend (optionnel)
  - `frontend/src/components/BuildingList.svelte` (l.34-50) — meme probleme
  - `backend/src/infrastructure/web/handlers/building_handlers.rs` — filtrage par role
  - `backend/src/application/use_cases/building_use_cases.rs` — logique filtrage owner
- **Scenarios BDD** :

```gherkin
Scenario: Coproprietaire ne voit que ses immeubles
  Given Sophie est coproprietaire du lot "2A" dans "Residence du Parc" uniquement
  When Sophie accede a la page /buildings
  Then seul l'immeuble "Residence du Parc" est affiche
  And le counter indique "1 immeuble"

Scenario: Syndic voit tous ses immeubles
  Given Marc est syndic de "Residence du Parc", "Les Tilleuls", et "Clos des Acacias"
  When Marc accede a la page /buildings
  Then les 3 immeubles sont affiches
  And le counter indique "3 immeubles"

Scenario: SuperAdmin voit tous les immeubles de la plateforme
  Given le SuperAdmin accede a la page /buildings
  Then tous les immeubles de toutes les organisations sont affiches
```

- **Dependances** : BFR-007 (les redirections doivent fonctionner pour tester chaque role)
- **Note** : Bug probablement backend (filtrage insuffisant dans le handler GET /buildings)

---

### Phase 3 : Bugs Mineurs (v0.2)

#### BFR-009 : Cles i18n ICU non resolues ({count}, {hours})

- **Description** : Les placeholders ICU MessageFormat (`{count}`, `{hours}`) s'affichent tels quels dans l'UI au lieu d'etre interpoles. Affecte les pages meetings, SEL, et potentiellement d'autres.
- **Priorite** : MINEUR
- **Bug IDs** : BUG-WF1-4, BUG-SEL-1
- **FR PRD parent** : NFR-7.9 (Internationalisation — 4 langues, ~2000 cles)
- **User Story** : En tant que Sophie (coproprietaire), quand je consulte la page SEL, je veux voir "42 heures d'entraide" et non "{hours} hours of mutual aid".
- **Fichiers impactes** :
  - `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json` — cles ICU
  - Composants utilisant ces cles (MeetingList, SelStatistics, etc.)
- **Scenarios BDD** :

```gherkin
Scenario: Interpolation correcte de {hours} dans les statistiques SEL
  Given le SEL de "Residence du Parc" a totalise 42 heures d'echanges
  When Sophie consulte les statistiques SEL
  Then le message affiche "42 heures d'entraide echangees dans votre communaute"
  And aucune accolade "{" n'apparait dans le texte

Scenario: Interpolation correcte de {count} dans la liste des reunions
  Given Marc a 3 reunions planifiees
  When Marc consulte la page /meetings
  Then le compteur affiche "3 reunions" (et non "{count}" brut)
```

- **Dependances** : Aucune

---

#### BFR-010 : Titres GDPR en anglais au lieu de francais

- **Description** : Les pages GDPR (`admin/gdpr.astro`, `settings/gdpr.astro`) contiennent des titres hardcodes en anglais ("GDPR") au lieu d'utiliser les cles i18n en francais ("RGPD").
- **Priorite** : MINEUR
- **Bug ID** : BUG-GDPR-1
- **FR PRD parent** : FR-015 (GDPR Articles 15-21+30)
- **Fichiers impactes** :
  - `frontend/src/pages/admin/gdpr.astro` (l.6, l.10-20)
  - `frontend/src/pages/settings/gdpr.astro` (l.6, l.16)
- **Scenarios BDD** :

```gherkin
Scenario: Page GDPR en francais pour locale FR
  Given Sophie a selectionne la langue "FR"
  When Sophie accede a la page /settings/gdpr
  Then le titre affiche "Protection des donnees (RGPD)"
  And toutes les sections sont en francais
```

- **Dependances** : Aucune

---

#### BFR-011 : Format date income-statement trop strict

- **Description** : Le handler `GET /reports/income-statement` exige le format ISO 8601 complet (`2026-01-01T00:00:00Z`) et rejette le format simplifie (`2026-01-01`) avec une erreur 400 peu claire.
- **Priorite** : MINEUR
- **Bug ID** : BUG-WF10-1
- **FR PRD parent** : FR-006 (Comptabilite PCMN belge)
- **Fichiers impactes** :
  - Backend handler de GET /reports/income-statement — parsing date
- **Scenarios BDD** :

```gherkin
Scenario: Accepter format date simplifie pour income-statement
  Given Jean-Pierre est connecte en tant que comptable
  When Jean-Pierre demande le compte de resultats avec period_start="2026-01-01" et period_end="2026-12-31"
  Then le rapport est genere avec succes
  And les dates sont interpretees comme debut de journee UTC (00:00:00Z)

Scenario: Accepter format date complet ISO 8601
  When Jean-Pierre demande le compte de resultats avec period_start="2026-01-01T00:00:00Z"
  Then le rapport est genere avec succes
```

- **Dependances** : Aucune
- **Note** : Bug backend (parsing flexible NaiveDate en fallback)

---

#### BFR-012 : Bouton "Voter" non visible pour le coproprietaire

- **Description** : Le bouton "Voter" dans `ResolutionVotePanel.svelte` est conditionne par `canVote` (l.31) qui ne reconnait pas correctement le role "owner", rendant le bouton invisible pour les coproprietaires.
- **Priorite** : MINEUR
- **Bug ID** : BUG-WF2-3
- **FR PRD parent** : FR-004 (Vote numerique majorites legales)
- **Fichiers impactes** :
  - `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.31, l.176, l.246-256)
- **Scenarios BDD** :

```gherkin
Scenario: Bouton "Voter" visible pour le coproprietaire sur resolution Pending
  Given Sophie est connectee en tant que coproprietaire
  And la resolution "Travaux de toiture" est en statut "Pending"
  When Sophie consulte la resolution
  Then le bouton "Voter" est visible
  And Sophie peut choisir "Pour", "Contre" ou "Abstention"

Scenario: Bouton "Voter" invisible apres cloture du scrutin
  Given la resolution "Travaux de toiture" est en statut "Adopted"
  When Sophie consulte la resolution
  Then aucun bouton "Voter" n'est affiche
  And le resultat final est visible
```

- **Dependances** : BFR-006 (NaN% corrige d'abord pour UX coherente)

---

#### BFR-013 : Copyright 2025 dans le footer

- **Description** : Le footer affiche "© 2025 KoproGo" en dur au lieu de l'annee courante.
- **Priorite** : MINEUR
- **Bug ID** : BUG-WF14-4
- **FR PRD parent** : Transversal (UX)
- **Fichiers impactes** :
  - `frontend/src/layouts/Layout.astro` (l.156)
- **Action** : Remplacer `2025` par `new Date().getFullYear()` ou `2024-${new Date().getFullYear()}`
- **Dependances** : Aucune

---

#### BFR-014 : Counter immeubles incoherent

- **Description** : Le counter affiche "1 immeuble" alors que la liste montre 3 immeubles.
- **Priorite** : MINEUR
- **Bug ID** : BUG-WF14-3
- **FR PRD parent** : Transversal (UX)
- **Note** : Ce bug se corrigera probablement automatiquement avec BFR-008 (isolation donnees). A verifier apres Phase 2.
- **Dependances** : BFR-008

---

#### BFR-015 : Message rate limiting sur le login

- **Description** : Le formulaire de login affiche un message generique d'erreur quand le compte est temporairement verrouille (HTTP 429). L'utilisateur ne comprend pas pourquoi son mot de passe "ne marche plus".
- **Priorite** : MINEUR
- **Bug ID** : BUG-RL-1
- **FR PRD parent** : FR-018 (Authentification — rate limiting 5/15min)
- **Fichiers impactes** :
  - `frontend/src/components/LoginForm.svelte` (l.19-60, l.52) — detecter status 429
- **Scenarios BDD** :

```gherkin
Scenario: Message specifique pour rate limiting
  Given Marc a echoue 5 tentatives de login en 15 minutes
  When Marc tente une 6eme connexion
  Then l'API retourne un code 429
  And le formulaire affiche "Trop de tentatives de connexion. Reessayez dans 15 minutes."
  And le message est different de "Identifiants incorrects"

Scenario: Message standard pour mauvais mot de passe
  Given Marc n'a pas atteint la limite de tentatives
  When Marc saisit un mot de passe incorrect
  Then le formulaire affiche "Identifiants incorrects"
```

- **Dependances** : Aucune

---

## 7. Exigences non-fonctionnelles (specifiques au sprint correctif)

### 7.1 Retrocompatibilite

- Toute modification frontend doit etre retrocompatible avec les endpoints API existants (559 endpoints)
- La migration SQL de BFR-003 (ALTER contrainte voting_power) doit etre idempotente et non-destructive

### 7.2 Testabilite

| Niveau | Action requise |
|--------|----------------|
| Unit (frontend) | Verifier `getVotePercentage(0, 0) === 0` (BFR-006) |
| BDD | Scenarios Gherkin ci-dessus pour chaque BFR |
| E2E | Relancer les workflows WF1, WF2, WF7, WF14 du plan de revue apres correction |
| Regression | Aucun test existant ne doit echouer apres correction |

### 7.3 i18n

- Toutes les nouvelles chaines UI (messages de toast, labels de boutons) doivent etre externalisees dans les 4 fichiers de locale (`fr.json`, `en.json`, `nl.json`, `de.json`)
- Priorite FR > EN > NL > DE

---

## 8. Criteres de succes sprint correctif

| # | Critere | Seuil | Comment mesurer | Phase |
|---|---------|-------|-----------------|-------|
| SC1 | Bugs critiques resolus | 4/4 (0 restant) | Retest workflow WF1, WF2, WF7 via UI | Phase 1 |
| SC2 | Bugs majeurs resolus | 4/4 (0 restant) | Retest WF14, isolation donnees, NaN% | Phase 2 |
| SC3 | Bugs mineurs resolus | 8/8 (0 restant) | Retest i18n, footer, GDPR, rate limit | Phase 3 |
| SC4 | Zero regression | 0 test casse | `cargo test` + `npm run build` | Continu |
| SC5 | Toast sur 100% erreurs API | 0 erreur silencieuse | Test manuel Chrome DevTools | Phase 0 |
| SC6 | WF1-6 passants en revue UI | 6/6 workflows OK | Relancer campagne revue post-correction | Phase 2 |

**GO beta publique** = SC1 + SC2 atteints, SC4 garanti.

---

## 9. Matrice de tracabilite BFR → Bug → FR parent

| BFR | Bug ID | FR PRD parent | Epic parent | Phase | Couche |
|-----|--------|---------------|-------------|-------|--------|
| BFR-000 | Transversal | UX | — | 0 | Frontend |
| BFR-001 | BUG-WF1-1 | FR-003 | Epic 3 (AG) | 1 | Frontend |
| BFR-002 | BUG-WF1-2 | FR-003 | Epic 3 (AG) | 1 | Frontend |
| BFR-003 | BUG-WF2-1 | FR-004 | Epic 3 (AG) | 1 | Backend + Frontend |
| BFR-004 | BUG-WF7-1 | FR-013 | Epic 6 (Maintenance) | 1 | Frontend |
| BFR-005 | BUG-WF1-3 | FR-003 | Epic 3 (AG) | 2 | Frontend |
| BFR-006 | BUG-WF2-2 | FR-004 | Epic 3 (AG) | 2 | Frontend |
| BFR-007 | BUG-WF14-1 | FR-018 | Epic 2 (Identity) | 2 | Frontend |
| BFR-008 | BUG-WF14-2 | FR-018 + FR-015 | Epic 2 (Identity) | 2 | Backend + Frontend |
| BFR-009 | BUG-WF1-4 + BUG-SEL-1 | NFR-7.9 | — | 3 | Frontend |
| BFR-010 | BUG-GDPR-1 | FR-015 | Epic 7 (GDPR) | 3 | Frontend |
| BFR-011 | BUG-WF10-1 | FR-006 | Epic 4 (Comptabilite) | 3 | Backend |
| BFR-012 | BUG-WF2-3 | FR-004 | Epic 3 (AG) | 3 | Frontend |
| BFR-013 | BUG-WF14-4 | UX | — | 3 | Frontend |
| BFR-014 | BUG-WF14-3 | UX | — | 3 | Frontend |
| BFR-015 | BUG-RL-1 | FR-018 | Epic 2 (Identity) | 3 | Frontend |

---

## 10. Contraintes et hypotheses

### 10.1 Contraintes

1. **Aucune nouvelle feature** : Ce sprint est exclusivement correctif
2. **Retrocompatibilite** : Les 559 endpoints API restent inchanges (sauf BFR-003 migration et BFR-011 parsing)
3. **Stack identique** : Astro 4.x, Svelte 4.x, Rust 1.75+, PostgreSQL 15
4. **Ordre des phases** : Phase 0 → 1 → 2 → 3 (dependances explicites dans le graphe)
5. **Validation** : Chaque phase validee par retest des workflows concernes avant passage a la suivante

### 10.2 Hypotheses

1. Les bugs identifies sont reproductibles en localhost/dev (pas de dependance a l'infra production)
2. Le systeme de toast existant est fonctionnel — il suffit de le brancher sur les erreurs API
3. Le bug d'isolation donnees (BFR-008) est backend (filtrage insuffisant) et non frontend
4. Le bug de redirection (BFR-007) est un probleme de timing ou de guard frontend, pas de backend
5. La correction de BFR-008 resoudra automatiquement BFR-014 (counter incoherent)

---

## Pipeline suivant

Ce PRD sera consomme par :
- **Etape 4** : Scrum Master (stories TDD pour agents IA) → `Maury/epics-and-stories-BUGFIX-UI.md`
- **Etape 5** : Execution sprint correctif (5 jours dev)
- **Etape 6** : Relance campagne de revue UI (WF1-6 + WF7 + WF14)

---

*Document genere par John (Product Manager BMAD) — Methode Maury Phase TOGAF B-C*
*Pipeline : HUMAN_REVIEW_REPORT_v0.1.0.md (Revue UI) → PRD-BUGFIX-UI.md (John, Phase B-C)*
*Prochaine etape : Epics & Stories Bugfix (Bob, Phase E) → epics-and-stories-BUGFIX-UI.md*

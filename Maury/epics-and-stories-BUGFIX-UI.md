# Epics & User Stories — KoproGo Bugfix UI v0.1.0

## Methode Maury — Phase TOGAF E (Solutions)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : Bob (Scrum Master)
**Date** : 01/04/2026
**Version** : 1.0
**Source revue** : `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` (01/04/2026, Claude — Revue UI)
**PRD source** : `Maury/PRD-BUGFIX-UI.md` v1.0 (01/04/2026, John — Product Manager)
**PRD parent** : `Maury/PRD.md` v1.0 (29/03/2026, John — Product Manager)
**Architecture source** : `Maury/architecture.md` v2.0 (29/03/2026, Winston — Architecte)

**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Execution** : Scrum → Nexus → SAFe
**Production** : ITIL + IaC ISO 27001

**Contexte** : Sprint correctif de 5 jours pour corriger les 16 bugs UI identifies lors de la revue humaine v0.1.0. Le backend est fonctionnel (559 endpoints), les bugs sont principalement des problemes d'integration frontend ↔ backend (payloads manquants, mapping incorrect, feedback absent, filtrage insuffisant).

**Tracabilite** : Chaque story trace vers un BFR du PRD-BUGFIX-UI, un Bug-ID du rapport de revue, et un FR du PRD parent. Les fichiers source sont ceux identifies dans le rapport de revue et confirmes par exploration du codebase.

---

## Vue d'ensemble Sprint Correctif

| Phase | Epic | Stories | Bugs | Effort | Debloque |
|-------|------|---------|------|--------|----------|
| 0 | Epic BF-0 : Infrastructure Toast | 1 story | Transversal | 0.5j | Feedback utilisateur partout |
| 1 | Epic BF-1 : Bugs Critiques | 4 stories | 4 critiques | 1.5j | Workflows AG + Tickets via UI |
| 2 | Epic BF-2 : Bugs Majeurs | 4 stories | 4 majeurs | 1.35j | UX coherente, securite donnees |
| 3 | Epic BF-3 : Bugs Mineurs | 7 stories | 8 mineurs | 1.85j | Polish pour beta publique |
| **Total** | **4 epics** | **16 stories** | **16 bugs** | **~5j** | **GO beta publique** |

---

## Epic BF-0 : Infrastructure Toast (Prealable) | MUST HAVE

> Debloque : Tous les bugs "silencieux" (BFR-001, BFR-002, BFR-004)
> BFR PRD : BFR-000
> Phase : 0

### Story BF-0.1 : Intercepteur toast pour erreurs API

- **ID** : STORY-BF-001 | **Type** : Bugfix | **Taille** : M
- **Bug IDs** : Transversal (toutes les erreurs 400/500 silencieuses)
- **BFR PRD** : BFR-000
- **FR PRD parent** : Transversal (UX)
- **User Story** : En tant que Marc (syndic), quand je soumets un formulaire et que l'API retourne une erreur 400/500, je veux voir un message d'erreur clair dans un toast rouge, afin de comprendre ce qui a echoue.
- **Scenarios BDD** :

```gherkin
Scenario: Afficher un toast d'erreur sur reponse 400
  Given Marc est connecte en tant que syndic
  When Marc soumet un formulaire avec des donnees invalides
  Then l'API retourne un code 400 avec un message d'erreur
  And un toast rouge s'affiche avec le message d'erreur de l'API
  And le toast disparait apres 5 secondes

Scenario: Afficher un toast d'erreur sur reponse 500
  Given Marc est connecte en tant que syndic
  When une erreur serveur se produit
  Then un toast rouge s'affiche avec "Erreur serveur. Veuillez reessayer."

Scenario: Afficher un toast de succes apres action reussie
  Given Marc cree un ticket avec succes (code 201)
  Then un toast vert s'affiche avec "Ticket cree avec succes"
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/stores/toast.ts` (54 lignes) et `frontend/src/components/ui/ToastContainer.svelte` pour comprendre l'API du store existant
  2. [ ] Creer un wrapper `frontend/src/lib/api/fetchWithToast.ts` ou modifier le client API existant pour intercepter les reponses HTTP 4xx/5xx
  3. [ ] Pour les erreurs 4xx : extraire le message du body JSON de la reponse et l'afficher via `toast.error(message)`
  4. [ ] Pour les erreurs 5xx : afficher un message generique "Erreur serveur. Veuillez reessayer."
  5. [ ] Pour les succes (201, 200 sur POST/PUT/DELETE) : afficher un toast vert avec un message contextuel
  6. [ ] Verifier que `ToastContainer.svelte` est monte dans `Layout.astro` (devrait deja l'etre)
  7. [ ] Brancher le wrapper sur les composants critiques : ConvocationPanel, TicketCreateModal, LoginForm
  8. [ ] Test manuel : verifier qu'un POST /convocations sans building_id affiche un toast rouge
- **Dependances** : Aucune
- **Fichiers** :
  - Existants : `frontend/src/stores/toast.ts`, `frontend/src/components/ui/ToastContainer.svelte`
  - A creer/modifier : `frontend/src/lib/api/fetchWithToast.ts` (ou modification du client API existant)
  - A modifier : Tous les composants effectuant des appels API (branchement progressif)

---

## Epic BF-1 : Bugs Critiques | MUST HAVE

> 4 bugs bloquant l'utilisation des workflows principaux via l'UI
> BFRs PRD : BFR-001, BFR-002, BFR-003, BFR-004
> Phase : 1
> Dependance : Epic BF-0 (toast pour feedback)

### Story BF-1.1 : Bouton "Nouvelle reunion" sur /meetings

- **ID** : STORY-BF-002 | **Type** : Bugfix | **Taille** : M
- **Bug ID** : BUG-WF1-1
- **BFR PRD** : BFR-001
- **FR PRD parent** : FR-003 (Convocations legales)
- **Bounded Context DDD** : General Assembly
- **User Story** : En tant que Marc (syndic), je veux un bouton "Nouvelle reunion" sur la page /meetings afin de creer une AG directement depuis l'interface.
- **Scenarios BDD** :

```gherkin
Scenario: Bouton "Nouvelle reunion" visible pour le syndic
  Given Marc est connecte en tant que syndic
  When Marc accede a la page /meetings
  Then un bouton "Nouvelle reunion" est visible en haut de page

Scenario: Bouton invisible pour le coproprietaire
  Given Sophie est connectee en tant que coproprietaire
  When Sophie accede a la page /meetings
  Then aucun bouton "Nouvelle reunion" n'est affiche

Scenario: Creer une AG via le modal
  Given Marc a clique sur "Nouvelle reunion"
  When Marc remplit le titre "AG Annuelle 2026", la date "20/03/2026", le type "Ordinary", et selectionne "Residence du Parc"
  And Marc valide le formulaire
  Then l'AG "AG Annuelle 2026" est creee avec succes
  And un toast vert confirme la creation
  And la reunion apparait dans la liste
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/pages/meetings.astro` (28 lignes) et `frontend/src/components/MeetingList.svelte` (173 lignes)
  2. [ ] Lire `frontend/src/lib/api/` pour identifier le client API meetings existant (ou le creer)
  3. [ ] Verifier si `MeetingCreateModal.svelte` existe deja dans `frontend/src/components/` ; sinon creer le composant
  4. [ ] Dans `meetings.astro` : ajouter un bouton "Nouvelle reunion" conditionne au role syndic (`{#if userRole === 'syndic'}`)
  5. [ ] Implementer `MeetingCreateModal.svelte` : champs titre (string), date (date picker), type (Ordinary/Extraordinary), building_id (select depuis BuildingSelector)
  6. [ ] Le POST doit appeler `POST /meetings` avec le body `{ title, meeting_date, meeting_type, building_id }`
  7. [ ] Utiliser le wrapper toast (STORY-BF-001) pour feedback succes/erreur
  8. [ ] Apres creation reussie : fermer le modal et recharger la liste des reunions
  9. [ ] Externaliser les labels dans les fichiers de locale (fr.json: "Nouvelle reunion", en.json: "New meeting", etc.)
  10. [ ] Test manuel : verifier le workflow complet syndic → bouton → modal → creation → toast → liste mise a jour
- **Dependances** : STORY-BF-001 (toast)
- **Debloque** : STORY-BF-003 (convocations), puis en cascade WF3-6
- **Fichiers** :
  - A modifier : `frontend/src/pages/meetings.astro`
  - A creer : `frontend/src/components/MeetingCreateModal.svelte` (si inexistant)
  - Reference API : `backend/src/infrastructure/web/handlers/meeting_handlers.rs`

---

### Story BF-1.2 : POST /convocations transmet building_id

- **ID** : STORY-BF-003 | **Type** : Bugfix | **Taille** : S
- **Bug ID** : BUG-WF1-2
- **BFR PRD** : BFR-002
- **FR PRD parent** : FR-003 (Convocations legales), INV-3 (delai legal 15 jours)
- **Bounded Context DDD** : General Assembly
- **User Story** : En tant que Marc (syndic), quand je cree une convocation pour une AG, je veux que le building_id soit automatiquement inclus dans la requete.
- **Scenarios BDD** :

```gherkin
Scenario: Convocation creee avec building_id transmis
  Given Marc est connecte en tant que syndic
  And l'AG "AG Annuelle 2026" existe pour "Residence du Parc"
  When Marc cree la convocation pour cette AG via l'UI
  Then le POST /convocations contient building_id dans le body
  And la convocation est creee avec succes (code 201)
  And un toast vert confirme la creation

Scenario: Erreur explicite si building_id manquant
  When le frontend envoie un POST /convocations sans building_id
  Then un toast rouge affiche "building_id is required"
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/convocations/ConvocationPanel.svelte` (lignes 49-62) — identifier ou building_id devrait etre recupere
  2. [ ] Lire `frontend/src/lib/api/convocations.ts` (lignes 67-73) — verifier le payload du POST
  3. [ ] Lire le backend DTO `backend/src/application/dto/convocation_dto.rs` pour connaitre les champs requis
  4. [ ] Identifier la source du building_id : prop du composant parent ? BuildingSelector ? Route param ?
  5. [ ] Corriger le payload POST pour inclure `building_id` dans le body JSON
  6. [ ] Si le building_id vient d'un BuildingSelector, verifier que le composant parent le transmet correctement en prop
  7. [ ] Utiliser le wrapper toast pour feedback succes/erreur
  8. [ ] Test manuel : creer une convocation via l'UI et verifier dans la console Network que building_id est present dans le body
- **Dependances** : STORY-BF-001 (toast), STORY-BF-002 (reunion doit exister)
- **Fichiers** :
  - A modifier : `frontend/src/components/convocations/ConvocationPanel.svelte`, `frontend/src/lib/api/convocations.ts`
  - Reference backend : `backend/src/application/dto/convocation_dto.rs`

---

### Story BF-1.3 : Relever contrainte voting_power a 10000

- **ID** : STORY-BF-004 | **Type** : Bugfix | **Taille** : M
- **Bug ID** : BUG-WF2-1
- **BFR PRD** : BFR-003
- **FR PRD parent** : FR-004 (Vote numerique majorites legales), INV-4
- **Bounded Context DDD** : General Assembly
- **Principes SOLID** : OCP (modifier la contrainte sans changer le moteur de calcul des majorites)
- **User Story** : En tant que Emmanuel (coproprietaire avec 1280 dix-milliemes), je veux pouvoir voter lors de l'AG avec mon pouvoir de vote reel.
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

Scenario: Accepter voting_power = 10000 (100% d'un immeuble)
  When un vote est soumis avec voting_power = 10000
  Then le vote est enregistre avec succes
```

- **Taches techniques** :
  1. [ ] Lire `backend/migrations/20251115120000_create_resolutions_and_votes.sql` — identifier la contrainte CHECK sur voting_power
  2. [ ] Lire `backend/src/domain/entities/vote.rs` — identifier la validation domain (0 < voting_power <= 1000)
  3. [ ] Creer une nouvelle migration `backend/migrations/[timestamp]_alter_voting_power_constraint.sql` :
     ```sql
     ALTER TABLE votes DROP CONSTRAINT IF EXISTS votes_max_voting_power;
     ALTER TABLE votes ADD CONSTRAINT votes_max_voting_power CHECK (voting_power >= 0 AND voting_power <= 10000);
     ```
  4. [ ] Modifier `backend/src/domain/entities/vote.rs` : changer la validation de `<= 1000` a `<= 10000`
  5. [ ] Modifier `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.225) : changer `max="1000"` en `max="10000"`
  6. [ ] Verifier la coherence avec le seed data : les tantiemes du seed ne doivent pas depasser 10000 par lot
  7. [ ] RED : Ecrire un test unitaire dans `vote.rs` : `test_vote_with_voting_power_1280_succeeds()`
  8. [ ] GREEN : Verifier que le test passe avec la nouvelle validation
  9. [ ] Verifier : `cargo test --lib` (aucune regression)
- **Dependances** : Aucune (independant des autres stories)
- **Fichiers** :
  - Backend migration : `backend/migrations/[timestamp]_alter_voting_power_constraint.sql` (a creer)
  - Backend domain : `backend/src/domain/entities/vote.rs` (modifier validation)
  - Frontend : `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.225)
  - Reference : `backend/migrations/20251115120000_create_resolutions_and_votes.sql`

---

### Story BF-1.4 : Corriger payload formulaire ticket

- **ID** : STORY-BF-005 | **Type** : Bugfix | **Taille** : S
- **Bug ID** : BUG-WF7-1
- **BFR PRD** : BFR-004
- **FR PRD parent** : FR-013 (Ticketing SLA)
- **Bounded Context DDD** : Maintenance
- **User Story** : En tant que Sophie (coproprietaire), je veux signaler une fuite d'eau en creant un ticket depuis l'interface.
- **Scenarios BDD** :

```gherkin
Scenario: Creer un ticket de maintenance via l'UI
  Given Sophie est connectee en tant que coproprietaire
  When Sophie remplit le formulaire de ticket :
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
  Then un toast rouge affiche l'erreur de validation
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/tickets/TicketCreateModal.svelte` (lignes 31-90) — identifier le body JSON envoye
  2. [ ] Lire le backend DTO `backend/src/application/dto/ticket_dto.rs` (CreateTicketDto) — comparer les champs attendus
  3. [ ] Ouvrir Chrome DevTools → Network tab → tenter de creer un ticket → capturer le body POST reellement envoye
  4. [ ] Identifier les divergences entre le body envoye et le DTO attendu (champs manquants, noms differents, types incorrects — UUID vs string, enum values)
  5. [ ] Corriger le payload dans TicketCreateModal.svelte pour correspondre exactement au CreateTicketDto
  6. [ ] Verifier les enum values : priority (Low/Medium/High/Urgent/Critical), category (Plumbing/Electrical/Heating/Cleaning/Security/General/Emergency)
  7. [ ] Utiliser le wrapper toast pour feedback succes/erreur
  8. [ ] Test manuel : creer un ticket via l'UI, verifier la reponse 201 dans la console Network
- **Dependances** : STORY-BF-001 (toast)
- **Fichiers** :
  - A modifier : `frontend/src/components/tickets/TicketCreateModal.svelte` (l.70-90)
  - Reference backend : `backend/src/application/dto/ticket_dto.rs`

---

## Epic BF-2 : Bugs Majeurs | MUST HAVE

> 4 bugs degradant significativement l'experience utilisateur
> BFRs PRD : BFR-005, BFR-006, BFR-007, BFR-008
> Phase : 2
> Dependance : Epic BF-1 (workflows doivent fonctionner pour tester)

### Story BF-2.1 : Afficher les convocations creees dans la liste UI

- **ID** : STORY-BF-006 | **Type** : Bugfix | **Taille** : M
- **Bug ID** : BUG-WF1-3
- **BFR PRD** : BFR-005
- **FR PRD parent** : FR-003 (Convocations legales)
- **Bounded Context DDD** : General Assembly
- **User Story** : En tant que Marc (syndic), apres avoir cree une convocation, je veux la voir apparaitre dans la liste des convocations.
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
  When Marc selectionne "Residence du Parc"
  Then un message "Aucune convocation" est affiche
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/pages/convocations.astro` (lignes 25-57) — comprendre le montage BuildingSelector
  2. [ ] Lire `frontend/src/components/convocations/ConvocationList.svelte` (lignes 1-157) — identifier l'appel API GET
  3. [ ] Lire `frontend/src/lib/api/convocations.ts` — verifier l'URL du GET et les parametres (building_id requis ?)
  4. [ ] Verifier que le BuildingSelector transmet bien le building_id selectionne au ConvocationList
  5. [ ] Verifier que le GET /convocations/{building_id} ou /buildings/{id}/convocations est appele avec le bon ID
  6. [ ] Verifier le mapping de la reponse API vers les props du composant (noms de champs, structure JSON)
  7. [ ] Verifier le rechargement apres creation : le composant doit refetch la liste apres un POST reussi
  8. [ ] Si le probleme est un evenement non propage entre ConvocationPanel (creation) et ConvocationList (affichage), ajouter un store Svelte ou un callback
  9. [ ] Test manuel : creer une convocation → verifier qu'elle apparait immediatement dans la liste sans refresh
- **Dependances** : STORY-BF-002 (bouton reunion), STORY-BF-003 (building_id)
- **Fichiers** :
  - A modifier : `frontend/src/pages/convocations.astro`, `frontend/src/components/convocations/ConvocationList.svelte`
  - Potentiellement : `frontend/src/lib/api/convocations.ts`

---

### Story BF-2.2 : Corriger NaN% dans les compteurs de vote

- **ID** : STORY-BF-007 | **Type** : Bugfix | **Taille** : XS
- **Bug ID** : BUG-WF2-2
- **BFR PRD** : BFR-006
- **FR PRD parent** : FR-004 (Vote numerique majorites legales)
- **Bounded Context DDD** : General Assembly
- **User Story** : En tant que Marc (syndic), quand je consulte une resolution sans votes, je veux voir "0%" et non "NaN%".
- **Scenarios BDD** :

```gherkin
Scenario: Afficher 0% quand aucun vote n'est enregistre
  Given la resolution "Travaux de toiture" est en statut "Pending"
  And aucun vote n'a ete enregistre (totalVotes = 0)
  When Marc consulte les compteurs de vote
  Then les compteurs affichent "Pour: 0.0%", "Contre: 0.0%", "Abstention: 0.0%"

Scenario: Pourcentages corrects apres votes
  Given la resolution a recu 450 Pour, 600 Contre, 200 Abstention
  When Marc consulte les compteurs
  Then les pourcentages sont "Pour: 36.0%", "Contre: 48.0%", "Abstention: 16.0%"
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (lignes 44-47) — fonction `getVotePercentage()`
  2. [ ] Ajouter une garde : `if (totalVotes === 0) return 0;` au debut de la fonction
  3. [ ] Verifier aussi que `totalVotes` n'est pas `undefined` ou `null` (ajouter `|| 0` en fallback)
  4. [ ] Verifier les 3 lignes d'affichage (l.141, l.151, l.161) qui appellent `.toFixed(1)`
  5. [ ] Test : ouvrir une resolution sans votes et verifier que "0.0%" est affiche partout
- **Dependances** : Aucune (correction isolee, 1 ligne de code)
- **Fichiers** :
  - A modifier : `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.44-47)

---

### Story BF-2.3 : Redirection post-login pour SuperAdmin

- **ID** : STORY-BF-008 | **Type** : Bugfix | **Taille** : S
- **Bug ID** : BUG-WF14-1
- **BFR PRD** : BFR-007
- **FR PRD parent** : FR-018 (Authentification multi-role)
- **Bounded Context DDD** : Identity & Access
- **User Story** : En tant que SuperAdmin, apres m'etre connecte, je veux etre redirige automatiquement vers /admin.
- **Scenarios BDD** :

```gherkin
Scenario: Redirection SuperAdmin → /admin
  Given le SuperAdmin saisit ses identifiants corrects
  When le login reussit avec role "superadmin"
  Then l'utilisateur est redirige vers /admin

Scenario: Redirection Syndic → /syndic
  When Marc se connecte avec role "syndic"
  Then Marc est redirige vers /syndic

Scenario: Redirection Coproprietaire → /buildings
  When Sophie se connecte avec role "owner"
  Then Sophie est redirigee vers /buildings

Scenario: Redirection Comptable → /accountant
  When Gisele se connecte avec role "accountant"
  Then Gisele est redirigee vers /accountant
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/LoginForm.svelte` (lignes 42-49) — comprendre le mapping role → URL
  2. [ ] Verifier le flux post-login : (a) la reponse API contient bien `active_role`, (b) le mapping associe "superadmin" → "/admin", (c) la redirection utilise `window.location.href` ou `goto()`
  3. [ ] Verifier s'il y a un guard/middleware Astro qui redirige vers /login (boucle de redirection possible)
  4. [ ] Verifier que la page `/admin` existe (`frontend/src/pages/admin/index.astro` ou `admin.astro`)
  5. [ ] Si le probleme est un timing (redirection avant que le cookie/token soit persiste), ajouter un `await` ou un `tick()` Svelte
  6. [ ] Test manuel : se connecter en SuperAdmin et verifier la redirection vers /admin
  7. [ ] Test : verifier les 4 roles (superadmin, syndic, owner, accountant) redirigent correctement
- **Dependances** : Aucune
- **Fichiers** :
  - A modifier : `frontend/src/components/LoginForm.svelte` (l.42-49)
  - Reference : `frontend/src/pages/admin/` (verifier existence)

---

### Story BF-2.4 : Isolation des donnees par role (buildings)

- **ID** : STORY-BF-009 | **Type** : Bugfix | **Taille** : M
- **Bug ID** : BUG-WF14-2 (+ BUG-WF14-3 counter incoherent, consequence directe)
- **BFR PRD** : BFR-008 (+ BFR-014)
- **FR PRD parent** : FR-018 (Identity & Access), FR-015 (GDPR — minimisation donnees)
- **Bounded Context DDD** : Identity & Access + Building Management
- **Principes SOLID** : SRP (le handler Building filtre selon le role), LSP (BuildingRepository substituable), DIP (filtrage dans le use case, pas dans l'infrastructure)
- **User Story** : En tant que Sophie (coproprietaire du lot 2A), je ne veux voir que l'immeuble "Residence du Parc" ou je suis proprietaire.
- **Scenarios BDD** :

```gherkin
Scenario: Coproprietaire ne voit que ses immeubles
  Given Sophie est coproprietaire du lot "2A" dans "Residence du Parc" uniquement
  When Sophie accede a la page /buildings
  Then seul l'immeuble "Residence du Parc" est affiche
  And le counter indique "1 immeuble"

Scenario: Syndic voit tous ses immeubles d'organisation
  Given Marc est syndic de "Residence du Parc", "Les Tilleuls", et "Clos des Acacias"
  When Marc accede a la page /buildings
  Then les 3 immeubles sont affiches

Scenario: SuperAdmin voit tous les immeubles
  Given le SuperAdmin accede a /buildings
  Then tous les immeubles de toutes les organisations sont affiches
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/BuildingSelector.svelte` (l.32-57) — identifier l'appel GET /buildings
  2. [ ] Lire `frontend/src/components/BuildingList.svelte` (l.34-50) — meme analyse
  3. [ ] Lire `backend/src/infrastructure/web/handlers/building_handlers.rs` — handler GET /buildings
  4. [ ] Lire `backend/src/application/use_cases/building_use_cases.rs` — logique find_all
  5. [ ] Diagnostic : le handler GET /buildings filtre-t-il par organization_id ET par role ?
     - Pour le syndic : retourner tous les buildings de son organization_id
     - Pour l'owner : retourner uniquement les buildings ou l'owner possede des lots (JOIN unit_owners → units → buildings)
     - Pour le superadmin : retourner tous les buildings
  6. [ ] Si le filtrage est absent cote backend (probable) :
     - Modifier `building_use_cases.rs` : ajouter une methode `find_by_owner(owner_id)` qui fait le JOIN
     - Modifier `building_handlers.rs` : selon le role du JWT, appeler `find_all` (syndic) ou `find_by_owner` (owner)
  7. [ ] Si le filtrage est present mais ignore cote frontend :
     - Verifier que le frontend transmet le token JWT dans les headers (Authorization: Bearer)
  8. [ ] Verifier que le counter dans BuildingList est calcule depuis la longueur du tableau retourne (pas hardcode)
  9. [ ] Test manuel : se connecter en tant que Sophie (owner) et verifier qu'un seul immeuble est affiche
  10. [ ] Test : verifier que BUG-WF14-3 (counter incoherent) est automatiquement resolu
- **Dependances** : STORY-BF-008 (redirections fonctionnelles pour tester chaque role)
- **Note** : Bug probablement backend (filtrage insuffisant dans le handler/use case)
- **Fichiers** :
  - Frontend : `frontend/src/components/BuildingSelector.svelte`, `frontend/src/components/BuildingList.svelte`
  - Backend : `backend/src/infrastructure/web/handlers/building_handlers.rs`, `backend/src/application/use_cases/building_use_cases.rs`
  - Backend port (possible) : `backend/src/application/ports/building_repository.rs` (nouvelle methode `find_by_owner_id`)

---

## Epic BF-3 : Bugs Mineurs | SHOULD HAVE

> 8 bugs cosmetiques et d'i18n pour polish beta publique
> BFRs PRD : BFR-009 a BFR-015
> Phase : 3
> Dependance : Epic BF-2 (UX de base fonctionnelle)

### Story BF-3.1 : Resoudre les cles i18n ICU ({count}, {hours})

- **ID** : STORY-BF-010 | **Type** : Bugfix | **Taille** : M
- **Bug IDs** : BUG-WF1-4, BUG-SEL-1
- **BFR PRD** : BFR-009
- **FR PRD parent** : NFR-7.9 (Internationalisation)
- **User Story** : En tant que Sophie, quand je consulte les statistiques SEL, je veux voir "42 heures d'entraide" et non "{hours} hours of mutual aid".
- **Scenarios BDD** :

```gherkin
Scenario: Interpolation de {hours} dans les statistiques SEL
  Given le SEL a totalise 42 heures d'echanges
  When Sophie consulte les statistiques
  Then le message affiche "42 heures d'entraide echangees dans votre communaute"

Scenario: Interpolation de {count} dans la liste des reunions
  Given Marc a 3 reunions planifiees
  When Marc consulte /meetings
  Then le compteur affiche "3 reunions"
```

- **Taches techniques** :
  1. [ ] Identifier la librairie i18n utilisee (svelte-i18n, paraglide, ou custom) dans `frontend/package.json`
  2. [ ] Verifier si la librairie supporte ICU MessageFormat ou si elle utilise un format different
  3. [ ] Lire les fichiers de locale `frontend/src/locales/fr.json` (lignes avec `{count}`, `{hours}`)
  4. [ ] Pour chaque cle non interpolee, verifier le composant Svelte qui l'utilise et s'assurer que les variables sont passees : `$t('key', { values: { count: n, hours: h } })`
  5. [ ] Si la librairie ne supporte pas ICU, convertir les cles au format supporte (ex: `$t('key', { count: n })`)
  6. [ ] Tester les 4 locales (fr, en, nl, de) pour les cles corrigees
  7. [ ] Grep dans le codebase pour trouver toutes les occurrences de `{count}` et `{hours}` non interpolees
- **Dependances** : Aucune
- **Fichiers** :
  - A modifier : `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json`
  - A modifier : Composants Svelte utilisant ces cles (MeetingList, SelStatistics, etc.)

---

### Story BF-3.2 : Traduire les titres GDPR en francais

- **ID** : STORY-BF-011 | **Type** : Bugfix | **Taille** : XS
- **Bug ID** : BUG-GDPR-1
- **BFR PRD** : BFR-010
- **FR PRD parent** : FR-015 (GDPR Articles 15-21+30)
- **User Story** : En tant que Sophie (locale FR), quand j'accede a la page GDPR, je veux voir "Protection des donnees (RGPD)" et non "GDPR".
- **Taches techniques** :
  1. [ ] Lire `frontend/src/pages/admin/gdpr.astro` (l.6, l.10-20) et `frontend/src/pages/settings/gdpr.astro` (l.6, l.16)
  2. [ ] Remplacer les titres hardcodes en anglais par des cles i18n
  3. [ ] Verifier que les traductions existent dans `fr.json` (ex: `gdpr.title` → "Protection des donnees (RGPD)")
  4. [ ] Ajouter les traductions manquantes dans les 4 locales
- **Dependances** : Aucune
- **Fichiers** :
  - A modifier : `frontend/src/pages/admin/gdpr.astro`, `frontend/src/pages/settings/gdpr.astro`
  - A modifier : `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json`

---

### Story BF-3.3 : Parsing flexible date income-statement

- **ID** : STORY-BF-012 | **Type** : Bugfix | **Taille** : XS
- **Bug ID** : BUG-WF10-1
- **BFR PRD** : BFR-011
- **FR PRD parent** : FR-006 (Comptabilite PCMN belge)
- **Bounded Context DDD** : Accounting
- **User Story** : En tant que Jean-Pierre (comptable), je veux pouvoir saisir "2026-01-01" comme date de debut de periode sans devoir ecrire "2026-01-01T00:00:00Z".
- **Scenarios BDD** :

```gherkin
Scenario: Accepter format date simplifie
  When Jean-Pierre demande le income-statement avec period_start="2026-01-01"
  Then le rapport est genere avec succes

Scenario: Accepter format ISO 8601 complet
  When Jean-Pierre demande le income-statement avec period_start="2026-01-01T00:00:00Z"
  Then le rapport est genere avec succes
```

- **Taches techniques** :
  1. [ ] Identifier le handler backend de `GET /reports/income-statement` dans `backend/src/infrastructure/web/handlers/`
  2. [ ] Lire le DTO ou query params utilise pour `period_start` et `period_end`
  3. [ ] Ajouter un parsing flexible : tenter `DateTime::parse_from_rfc3339()` d'abord, puis fallback sur `NaiveDate::parse_from_str()` avec conversion en debut de journee UTC
  4. [ ] RED : Ecrire un test unitaire pour le parsing flexible (les deux formats doivent fonctionner)
  5. [ ] GREEN : Implementer le parsing
  6. [ ] Verifier : `cargo test --lib`
- **Dependances** : Aucune
- **Note** : Bug backend uniquement
- **Fichiers** :
  - Backend handler : `backend/src/infrastructure/web/handlers/financial_report_handlers.rs` (probable)

---

### Story BF-3.4 : Rendre le bouton "Voter" visible pour les owners

- **ID** : STORY-BF-013 | **Type** : Bugfix | **Taille** : S
- **Bug ID** : BUG-WF2-3
- **BFR PRD** : BFR-012
- **FR PRD parent** : FR-004 (Vote numerique majorites legales)
- **Bounded Context DDD** : General Assembly
- **User Story** : En tant que Sophie (coproprietaire), quand une resolution est ouverte au vote, je veux voir le bouton "Voter" pour exprimer mon choix.
- **Scenarios BDD** :

```gherkin
Scenario: Bouton "Voter" visible pour owner sur resolution Pending
  Given Sophie est connectee en tant que coproprietaire
  And la resolution "Travaux de toiture" est en statut "Pending"
  When Sophie consulte la resolution
  Then le bouton "Voter" est visible

Scenario: Bouton "Voter" invisible apres cloture
  Given la resolution est en statut "Adopted"
  When Sophie consulte la resolution
  Then aucun bouton "Voter" n'est affiche
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.31) — comprendre la logique `canVote`
  2. [ ] Identifier pourquoi `canVote` retourne false pour le role "owner" :
     - Le role est-il correctement lu depuis le store auth / JWT ?
     - La condition verifie-t-elle `role === 'owner'` ou un autre identifiant ?
     - La resolution est-elle bien en statut "Pending" ?
  3. [ ] Corriger la condition `canVote` pour inclure le role "owner" quand la resolution est "Pending"
  4. [ ] Verifier que le formulaire de vote (l.176-257) fonctionne apres correction
  5. [ ] Test manuel : se connecter en tant que Sophie, naviguer vers une resolution Pending, verifier le bouton
- **Dependances** : STORY-BF-007 (NaN% corrige pour UX coherente)
- **Fichiers** :
  - A modifier : `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.31, l.176)

---

### Story BF-3.5 : Copyright dynamique dans le footer

- **ID** : STORY-BF-014 | **Type** : Bugfix | **Taille** : XS
- **Bug ID** : BUG-WF14-4
- **BFR PRD** : BFR-013
- **FR PRD parent** : Transversal (UX)
- **Taches techniques** :
  1. [ ] Lire `frontend/src/layouts/Layout.astro` (l.156)
  2. [ ] Remplacer `&copy; 2025` par `&copy; {new Date().getFullYear()}` (ou `2024-{new Date().getFullYear()}`)
  3. [ ] Verifier que le rendu Astro evalue bien l'expression JavaScript (dans un bloc `{}`)
- **Dependances** : Aucune
- **Fichiers** :
  - A modifier : `frontend/src/layouts/Layout.astro` (l.156)

---

### Story BF-3.6 : Counter immeubles coherent (verification post-BF-2.4)

- **ID** : STORY-BF-015 | **Type** : Bugfix | **Taille** : XS
- **Bug ID** : BUG-WF14-3
- **BFR PRD** : BFR-014
- **FR PRD parent** : Transversal (UX)
- **Note** : Ce bug est une consequence directe de BFR-008 (isolation donnees). Il se corrigera probablement automatiquement apres STORY-BF-009.
- **Taches techniques** :
  1. [ ] Apres correction de STORY-BF-009, verifier que le counter affiche le nombre correct d'immeubles
  2. [ ] Si toujours incoherent : verifier que le counter est calcule depuis `buildings.length` et non une valeur hardcodee ou un endpoint separé
  3. [ ] Corriger si necessaire
- **Dependances** : STORY-BF-009 (isolation donnees)
- **Fichiers** :
  - Reference : `frontend/src/components/BuildingList.svelte`

---

### Story BF-3.7 : Message rate limiting specifique sur login

- **ID** : STORY-BF-016 | **Type** : Bugfix | **Taille** : S
- **Bug ID** : BUG-RL-1
- **BFR PRD** : BFR-015
- **FR PRD parent** : FR-018 (Authentification — rate limiting)
- **Bounded Context DDD** : Identity & Access
- **User Story** : En tant que Marc, quand mon compte est temporairement verrouille apres 5 tentatives, je veux voir "Trop de tentatives, reessayez dans 15 minutes" et non "Identifiants incorrects".
- **Scenarios BDD** :

```gherkin
Scenario: Message specifique pour rate limiting (HTTP 429)
  Given Marc a echoue 5 tentatives de login en 15 minutes
  When Marc tente une 6eme connexion
  Then le formulaire affiche "Trop de tentatives de connexion. Reessayez dans 15 minutes."

Scenario: Message standard pour mauvais mot de passe (HTTP 401)
  Given Marc n'a pas atteint la limite
  When Marc saisit un mot de passe incorrect
  Then le formulaire affiche "Identifiants incorrects"
```

- **Taches techniques** :
  1. [ ] Lire `frontend/src/components/LoginForm.svelte` (l.19-60, specifiquement le handler de reponse l.52)
  2. [ ] Identifier comment la reponse HTTP est traitee apres le POST /auth/login
  3. [ ] Ajouter une condition : si `response.status === 429`, afficher le message rate limiting specifique
  4. [ ] Sinon si `response.status === 401`, afficher "Identifiants incorrects"
  5. [ ] Sinon (5xx), afficher "Erreur serveur"
  6. [ ] Externaliser les 3 messages dans les fichiers i18n
  7. [ ] Test manuel : verrouiller un compte (5 tentatives) et verifier le message affiche
- **Dependances** : Aucune
- **Fichiers** :
  - A modifier : `frontend/src/components/LoginForm.svelte` (l.52)
  - A modifier : `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json`

---

## Suggestion de planification Sprint Correctif

### Jour 1 — Phase 0 + Debut Phase 1

| Priorite | Story | Epic | Taille | BFR |
|----------|-------|------|--------|-----|
| 1 | STORY-BF-001 | BF-0 (Toast) | M | BFR-000 |
| 2 | STORY-BF-002 | BF-1 (Btn reunion) | M | BFR-001 |

### Jour 2 — Suite Phase 1

| Priorite | Story | Epic | Taille | BFR |
|----------|-------|------|--------|-----|
| 3 | STORY-BF-003 | BF-1 (building_id) | S | BFR-002 |
| 4 | STORY-BF-004 | BF-1 (voting_power) | M | BFR-003 |
| 5 | STORY-BF-005 | BF-1 (ticket form) | S | BFR-004 |

### Jour 3 — Phase 2 (debut)

| Priorite | Story | Epic | Taille | BFR |
|----------|-------|------|--------|-----|
| 6 | STORY-BF-006 | BF-2 (liste convoc.) | M | BFR-005 |
| 7 | STORY-BF-007 | BF-2 (NaN%) | XS | BFR-006 |
| 8 | STORY-BF-008 | BF-2 (redirect admin) | S | BFR-007 |

### Jour 4 — Phase 2 (fin) + Phase 3 (debut)

| Priorite | Story | Epic | Taille | BFR |
|----------|-------|------|--------|-----|
| 9 | STORY-BF-009 | BF-2 (isolation) | M | BFR-008 |
| 10 | STORY-BF-010 | BF-3 (i18n ICU) | M | BFR-009 |
| 11 | STORY-BF-011 | BF-3 (GDPR FR) | XS | BFR-010 |

### Jour 5 — Phase 3 (fin) + Validation

| Priorite | Story | Epic | Taille | BFR |
|----------|-------|------|--------|-----|
| 12 | STORY-BF-012 | BF-3 (date parsing) | XS | BFR-011 |
| 13 | STORY-BF-013 | BF-3 (btn Voter) | S | BFR-012 |
| 14 | STORY-BF-014 | BF-3 (footer) | XS | BFR-013 |
| 15 | STORY-BF-015 | BF-3 (counter) | XS | BFR-014 |
| 16 | STORY-BF-016 | BF-3 (rate limit) | S | BFR-015 |

---

## Matrice de tracabilite BFR → Stories → Bug → FR parent

| BFR PRD | Story | Bug ID | FR parent | Epic parent | Phase |
|---------|-------|--------|-----------|-------------|-------|
| BFR-000 | STORY-BF-001 | Transversal | UX | BF-0 | 0 |
| BFR-001 | STORY-BF-002 | BUG-WF1-1 | FR-003 | BF-1 | 1 |
| BFR-002 | STORY-BF-003 | BUG-WF1-2 | FR-003 | BF-1 | 1 |
| BFR-003 | STORY-BF-004 | BUG-WF2-1 | FR-004 | BF-1 | 1 |
| BFR-004 | STORY-BF-005 | BUG-WF7-1 | FR-013 | BF-1 | 1 |
| BFR-005 | STORY-BF-006 | BUG-WF1-3 | FR-003 | BF-2 | 2 |
| BFR-006 | STORY-BF-007 | BUG-WF2-2 | FR-004 | BF-2 | 2 |
| BFR-007 | STORY-BF-008 | BUG-WF14-1 | FR-018 | BF-2 | 2 |
| BFR-008 | STORY-BF-009 | BUG-WF14-2 | FR-018+FR-015 | BF-2 | 2 |
| BFR-009 | STORY-BF-010 | BUG-WF1-4+SEL-1 | NFR-7.9 | BF-3 | 3 |
| BFR-010 | STORY-BF-011 | BUG-GDPR-1 | FR-015 | BF-3 | 3 |
| BFR-011 | STORY-BF-012 | BUG-WF10-1 | FR-006 | BF-3 | 3 |
| BFR-012 | STORY-BF-013 | BUG-WF2-3 | FR-004 | BF-3 | 3 |
| BFR-013 | STORY-BF-014 | BUG-WF14-4 | UX | BF-3 | 3 |
| BFR-014 | STORY-BF-015 | BUG-WF14-3 | UX | BF-3 | 3 |
| BFR-015 | STORY-BF-016 | BUG-RL-1 | FR-018 | BF-3 | 3 |

---

## Criteres de validation par phase

| Phase | Critere | Test de validation |
|-------|---------|-------------------|
| Phase 0 | Toast sur 100% erreurs API | POST invalide → toast rouge visible |
| Phase 1 | WF1 (AG+convocations) via UI | Syndic cree reunion → convocation → visible |
| Phase 1 | WF2 (votes) via UI | Owner vote avec >1000 tantiemes → succes |
| Phase 1 | WF7 (tickets) via UI | Owner cree ticket → code 201 → toast vert |
| Phase 2 | Convocations listees | Creer → apparait dans liste sans refresh |
| Phase 2 | NaN% corrige | Resolution sans votes → "0.0%" affiche |
| Phase 2 | Redirections login | 4 roles → 4 dashboards corrects |
| Phase 2 | Isolation donnees | Sophie (owner) → 1 immeuble visible |
| Phase 3 | i18n ICU | "{hours}" → "42 heures" (pas d'accolades) |
| Phase 3 | GDPR en FR | "RGPD" affiche, pas "GDPR" |
| Phase 3 | Rate limiting | 6eme tentative → message specifique 429 |
| **GO** | **0 bug critique + 0 bug majeur** | **Relance campagne revue UI** |

---

*Methode Maury — Phase TOGAF E (Solutions) — Par Gilles Maury & Farah Maury*
*Agent BMAD : Bob (Scrum Master) — 01/04/2026*
*Pipeline : HUMAN_REVIEW_REPORT_v0.1.0.md → PRD-BUGFIX-UI.md (John) → epics-and-stories-BUGFIX-UI.md (Bob)*

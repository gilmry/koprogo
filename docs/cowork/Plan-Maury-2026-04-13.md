# Post-Audit Epics — Session 2026-04-12/13

## Methode Maury — Phase TOGAF E (Solutions)

**Auteur** : Gilles Maury (avec Claude Opus 4.6 comme pair-programmer)
**Date** : 2026-04-13
**Version** : 1.0
**Branche** : `feature/dev`
**Source** : Rapports d'audit cowork v1 → v7 (`docs/cowork/Prompt2026-04-12-18-00.md` → `Prompt2026-04-13-02-00.md`)
**Brief source** : `Maury/product-brief.md` v1.0
**PRD source** : `Maury/PRD.md` v1.0
**Architecture source** : `Maury/architecture.md` v2.0

**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Execution** : Scrum (sprint single-dev)
**Contexte** : ce document documente le reliquat de la session d'audit 2026-04-12/13 et le transforme en epics exécutables par agents IA avec traçabilité vers les FR du PRD et les invariants du domaine.

**Traçabilité** : Ce document consomme et transforme les 7 rapports cowork en stories avec ID, Bounded Context DDD, Invariants, FR PRD, scénarios BDD Gherkin et checklist TDD RED/GREEN/REFACTOR. Chaque story trace vers un bug résiduel non traité par les 9 commits thématiques déjà posés sur `feature/dev`.

---

## Retrospective — Commits déjà posés sur `feature/dev`

9 commits thématiques couvrent le reliquat **déjà corrigé** durant la session d'audit 2026-04-12/13. Ces commits ne sont **pas** des stories Maury (pas de traçabilité BDD pré-existante) mais des réparations réactives. Ils sont listés ici pour que le plan post-audit sache ce qui est acquis.

| # | Commit | Scope | Bugs audit fermés |
|---|--------|-------|-------------------|
| 1 | `00e156b` feat(tickets): enrich ticket responses with requester/assignee names | Backend | v1 #P1 (requester Inconnu), v1 #P1 (stats empty) |
| 2 | `4c0540e` docs(cowork): audit trail v1→v7 | Docs | — |
| 3 | `241a336` feat(tickets): sticky modal, aligned enums, syndic workflow, i18n badges | Frontend | v1 #P1, v4 (Urgent), v5 (category 9), v6 #B5 (workflow gated) |
| 4 | `d04ea54` feat(meetings): location field + i18n meeting_type | Frontend | v1 #P2 (location), v7 #B7 (Ordinary) |
| 5 | `ea646c4` feat(resolutions): enum snake_case + vote panel owner/unit | Frontend | v6 #B1, v6 #B2, v7 #B1 |
| 6 | `841e47c` feat(expenses): building selector + i18n category/status | Frontend | v6 #B3, v7 #B8 |
| 7 | `268a24a` feat(api): session-expired dedup + mask raw DB errors | Frontend | v1 #P10, v7 #B13 |
| 8 | `e00b721` fix(ui): SEL formatDate + owner dashboard units + accents | Frontend | v1 (Lot 1A), v7 #B11-B12 |
| 9 | `1d01d06` i18n: align FR/NL/DE/EN + polls.status | Locales | v5 #B1, v6 #B1, v7 #B15 |

**Bugs v7 encore ouverts** (traités par les Epics ci-dessous) :
- **B4** UI validation quorum (déféré depuis v6)
- **B14** Contractor seed + dropdown assignation ticket
- **B16** Pattern Svelte 5 bind programmatique (outil de test)
- **B17** Toasts erreurs API restant en anglais brut
- **B18** Permissions UI non-gated sur meeting-detail
- **B19** "Back to Tickets" non traduit
- **B21** Toasts sans auto-close ni dedup

**Chantiers structurels** recommandés par v7 conclusion :
- Chaîne type-gen auto frontend ↔ backend (utoipa → openapi-typescript)
- Documentation des patterns de test programmatique Svelte 5

---

## Epic P7-1 : Contrat frontend ↔ backend type-safe | MUST HAVE

**Motivation** : 9 des 12 bugs critiques recensés v1→v7 étaient des désalignements d'enums/champs frontend↔backend qui auraient dû se détecter à la compilation. L'infrastructure existe déjà (utoipa 5.3 + SwaggerUI + openapi-typescript 7.13 + `frontend/src/types/api.d.ts` auto-généré) mais elle n'est pas **consommée** par les composants qui utilisent encore des wrappers hand-written dans `frontend/src/lib/api/*.ts`.

**Objectif** : faire de l'OpenAPI la source de vérité, régénérée à chaque PR, et migrer les 3 wrappers les plus "buggy" vers des re-exports depuis `api.d.ts`.

### Story P7-1.1 : Export OpenAPI déterministe depuis un binaire dédié

- **ID** : STORY-P7-101 | **Type** : Infra | **Taille** : S
- **Bounded Context DDD** : Infrastructure / OpenAPI
- **Entité(s) DDD** : aucun (technique)
- **Principes SOLID** : SRP (un binaire = un export), DIP (consomme `ApiDoc::openapi()` qui est déjà abstrait)
- **Invariants** : aucun business (infrastructure pure)
- **FR PRD** : transverse — support pour FR-* (API stable)
- **User Story** : En tant que développeur (frontend ou backend), je veux qu'une commande unique `make openapi-export` régénère `docs/api/openapi.json` pour que les types TypeScript restent synchronisés avec les DTO Rust sans étape manuelle.
- **Scénarios BDD** :

```gherkin
Scenario: Exporter la spec OpenAPI sans erreur
  Given le code backend compile sans erreur
  When le développeur execute "make openapi-export"
  Then le fichier "docs/api/openapi.json" est créé ou mis à jour
  And la sortie contient "Routes: <N>" et "Schemas: <N>" avec N > 0
  And le code de sortie est 0

Scenario: Détecter une divergence avec la spec commitée
  Given la spec "docs/api/openapi.json" est commitée
  When un DTO annoté ToSchema est modifié dans le backend
  And le développeur execute "make openapi-check"
  Then la commande retourne un code non-zéro
  And le message indique "openapi.json has diverged — run 'make openapi-export'"
```

- **Taches techniques TDD** :
  1. [ ] RED : Créer `backend/src/bin/export_openapi.rs` qui appelle `koprogo_api::infrastructure::openapi::ApiDoc::openapi().to_pretty_json()` et écrit dans `docs/api/openapi.json`.
  2. [ ] GREEN : `cargo run --bin export_openapi` écrit le fichier et exit 0.
  3. [ ] Ajouter cibles Makefile racine :
     - `openapi-export` : chaîne `cargo run --bin export_openapi` + check que `docs/api/openapi.json` existe.
     - `openapi-check` : `openapi-export` puis `git diff --exit-code docs/api/openapi.json`.
  4. [ ] Supprimer `docs/api/openapi.yaml` et mettre à jour `frontend/package.json` script `types:generate` pour consommer `openapi.json` au lieu de `openapi.yaml` (openapi-typescript accepte les deux).
  5. [ ] Ajouter `openapi-export` au pré-commit hook via `make pre-commit` (optionnel — peut alourdir, à faire seulement si `docs/api/` est modifié dans le diff).
  6. [ ] Verifier : `make openapi-export && git status` montre `docs/api/openapi.json` comme modifié si des DTO ont changé.
- **Dependances** : aucune (`openapi.rs` existe déjà avec `ApiDoc`).
- **Endpoints** : aucun (tooling).
- **Fichiers** : `backend/src/bin/export_openapi.rs` (nouveau, ~30 LOC), `Makefile` (+ 2 cibles), `frontend/package.json` (rename script).

### Story P7-1.2 : Annoter les DTOs de résolution, meeting, expense et poll

- **ID** : STORY-P7-102 | **Type** : Refactor | **Taille** : M
- **Bounded Context DDD** : Governance (résolutions, meetings), Finance (expenses), Community (polls)
- **Entité(s) DDD** : `Resolution`, `Vote`, `Meeting`, `Expense`, `Poll`
- **Principes SOLID** : OCP (annotations additives), ISP (chaque DTO expose son schéma)
- **Invariants** : les enum values générées dans `docs/api/openapi.json` doivent correspondre **exactement** aux variantes serde-serialisées (INV-new : "OpenAPI schema = serde output").
- **FR PRD** : FR-008 (résolutions), FR-004 (AG), FR-006 (facture), FR-012 (sondages)
- **User Story** : En tant que développeur frontend, je veux que les enums `MajorityType`, `ResolutionType`, `ExpenseCategory`, `PaymentStatus`, `MeetingType`, `PollStatus` apparaissent correctement dans `api.d.ts` pour ne plus jamais inventer des variantes fictives (ex: `MajorityType.Simple` qui n'existe pas).
- **Scénarios BDD** :

```gherkin
Scenario: Les enums résolution apparaissent dans le schéma OpenAPI généré
  Given le backend est annoté avec utoipa::ToSchema sur ResolutionType et MajorityType
  When "make openapi-export" est exécuté
  Then docs/api/openapi.json contient le schéma "MajorityType" avec enum values ["absolute","two_thirds","four_fifths","unanimity"]
  And le schéma "ResolutionType" contient enum values ["ordinary","extraordinary"]

Scenario: Le frontend peut importer le type depuis api.d.ts
  Given docs/api/openapi.json est à jour
  When "npm run types:generate" est exécuté
  Then frontend/src/types/api.d.ts contient "components[\"schemas\"][\"MajorityType\"]"
  And le type est une union des 4 valeurs snake_case
```

- **Taches techniques TDD** :
  1. [ ] Vérifier que `ResolutionType`, `MajorityType` (resolution.rs) ont déjà `#[derive(ToSchema)]`. Si oui, passer.
  2. [ ] Ajouter `#[derive(ToSchema)]` sur : `ExpenseCategory`, `PaymentStatus`, `ApprovalStatus` (expense.rs).
  3. [ ] Ajouter `#[derive(ToSchema)]` sur : `MeetingType`, `MeetingStatus` (meeting.rs), `ValidateQuorumRequest` (meeting_dto.rs).
  4. [ ] Ajouter `#[derive(ToSchema)]` sur : `PollStatus`, `PollType` (poll.rs) si non déjà présents.
  5. [ ] Lister les schemas dans `ApiDoc` via l'attribut `components(schemas(...))` pour forcer leur inclusion même si pas référencés par un handler annoté.
  6. [ ] RED : écrire un test `backend/tests/openapi_schema.rs` qui vérifie via `ApiDoc::openapi()` que `MajorityType` a bien 4 valeurs et `ResolutionType` 2 valeurs.
  7. [ ] GREEN : faire passer le test.
  8. [ ] `make openapi-export` et vérifier à l'œil les schemas générés.
  9. [ ] Commit : `refactor(api): expose resolution/meeting/expense/poll enums in OpenAPI schema`.
- **Dependances** : STORY-P7-101.
- **Endpoints** : aucun.
- **Fichiers** : `backend/src/domain/entities/{expense,meeting,poll}.rs`, `backend/src/application/dto/{meeting,poll}_dto.rs`, `backend/src/infrastructure/openapi.rs` (+ `components(schemas(..))`), `backend/tests/openapi_schema.rs` (nouveau).

### Story P7-1.3 : Migrer les wrappers frontend des enums critiques vers `api.d.ts`

- **ID** : STORY-P7-103 | **Type** : Refactor | **Taille** : S
- **Bounded Context DDD** : Frontend / API types
- **Entité(s) DDD** : N/A (couche TypeScript)
- **Principes SOLID** : DRY (une source de vérité unique = le YAML), DIP (les composants dépendent du type généré, pas du wrapper)
- **Invariants** : INV-new "Frontend enum === Backend enum". Impossible de créer une variante qui n'existe pas côté Rust.
- **FR PRD** : transverse.
- **User Story** : En tant que développeur frontend, je veux que quand j'import `MajorityType` depuis `lib/api/resolutions.ts`, j'obtienne le type généré depuis `api.d.ts` pour que TypeScript refuse à la compilation toute valeur qui n'existe pas côté backend.
- **Scénarios BDD** :

```gherkin
Scenario: Le wrapper resolutions.ts refuse une valeur fictive
  Given api.d.ts contient MajorityType = "absolute" | "two_thirds" | "four_fifths" | "unanimity"
  And resolutions.ts réexporte ce type
  When un développeur écrit "let m: MajorityType = 'simple'"
  Then TypeScript erreur TS2322 "'simple' is not assignable to type 'MajorityType'"

Scenario: Le composant ResolutionCreateForm consomme les vraies valeurs
  Given resolutions.ts exporte les constantes depuis api.d.ts
  When ResolutionCreateForm.svelte liste les options du dropdown
  Then exactement 4 options de majorité sont rendues
  And exactement 2 options de type de résolution sont rendues
```

- **Taches techniques TDD** :
  1. [ ] Dans `frontend/src/lib/api/resolutions.ts`, remplacer :
     ```ts
     export enum MajorityType { Absolute = "absolute", ... }
     ```
     par :
     ```ts
     import type { components } from "../../types/api";
     export type MajorityType = components["schemas"]["MajorityType"];
     export const MajorityType = {
       Absolute: "absolute" as const,
       TwoThirds: "two_thirds" as const,
       FourFifths: "four_fifths" as const,
       Unanimity: "unanimity" as const,
     } satisfies Record<string, MajorityType>;
     ```
  2. [ ] Idem pour `ResolutionType` (2 valeurs).
  3. [ ] Même pattern pour `tickets.ts` : `TicketPriority`, `TicketCategory`, `TicketStatus` depuis `api.d.ts`.
  4. [ ] Même pattern pour `polls.ts` : `PollStatus`, `PollType`.
  5. [ ] Vérifier par compilation : `cd frontend && npx svelte-check` doit passer sans nouvelle erreur.
  6. [ ] Grep le codebase pour toute référence à `MajorityType.Simple` / `Urgent` / `General` / `Emergency` — s'assurer qu'aucune n'existe (sinon erreur TS).
  7. [ ] Commit : `refactor(frontend): re-export critical enums from generated api.d.ts`.
- **Dependances** : STORY-P7-101, STORY-P7-102.
- **Endpoints** : aucun.
- **Fichiers** : `frontend/src/lib/api/{resolutions,tickets,polls}.ts`.

### Story P7-1.4 : CI garde de non-régression OpenAPI

- **ID** : STORY-P7-104 | **Type** : Infra | **Taille** : S
- **Bounded Context DDD** : CI/CD
- **Entité(s) DDD** : N/A
- **Principes SOLID** : OCP (la garde ne modifie pas les workflows existants)
- **Invariants** : aucun.
- **FR PRD** : transverse (qualité).
- **User Story** : En tant que mainteneur, je veux qu'un PR qui modifie un DTO sans régénérer `openapi.json` soit bloqué par la CI, pour éviter que la spec devienne stale comme elle l'était avant cet audit (18 jours d'écart).
- **Scénarios BDD** :

```gherkin
Scenario: La CI rejette un DTO modifié sans régénération du spec
  Given un PR modifie un champ de CreateMeetingRequest
  And le dev n'a pas régénéré docs/api/openapi.json
  When la CI exécute "make openapi-check"
  Then la CI échoue avec exit code != 0
  And le message suggère "run 'make openapi-export' and commit"

Scenario: Un PR qui ne touche pas les DTOs passe la garde
  Given un PR modifie uniquement du code dans application/use_cases/
  When la CI exécute "make openapi-check"
  Then la commande retourne exit code 0
```

- **Taches techniques TDD** :
  1. [ ] Ajouter une étape dans `.github/workflows/ci.yml` (ou équivalent) après `cargo build` :
     ```yaml
     - name: Check OpenAPI spec is up-to-date
       run: make openapi-check
     ```
  2. [ ] Garder `npm run types:check` déjà présent côté frontend.
  3. [ ] Tester localement : `git stash && make openapi-check` doit passer. Puis modifier un champ d'un DTO et `make openapi-check` doit échouer.
  4. [ ] Commit : `ci: guard against stale openapi.json via make openapi-check`.
- **Dependances** : STORY-P7-101.
- **Endpoints** : aucun.
- **Fichiers** : `.github/workflows/ci.yml` (ou équivalent), `Makefile`.

---

## Epic P7-2 : Ticket assignment UX | MUST HAVE

**Motivation** : `TicketAssignModal.svelte` est un stub qui demande un UUID saisi à la main, et le seed ne contient aucun "contractor" comme cible d'assignation. Un utilisateur réel ne connaît jamais les UUIDs. L'audit v7 l'a explicitement classé bloquant (#B14).

**Décision de design** : pas de nouveau rôle `Contractor`. Le domaine `UserRole` expose déjà `SuperAdmin`, `Syndic`, `Accountant`, `BoardMember`, `Owner`. Un contractor KoproGo = "la personne qui prend en charge le ticket" — ça peut être le syndic lui-même, un membre du conseil ou un propriétaire-bricoleur. On réutilise donc les users existants via un endpoint filtré.

### Story P7-2.1 : Endpoint backend `/tickets/assignable-users`

- **ID** : STORY-P7-201 | **Type** : Feature | **Taille** : S
- **Bounded Context DDD** : Maintenance / Ticket Management
- **Entité(s) DDD** : `User`, `Ticket`
- **Principes SOLID** : SRP (un endpoint = lister les cibles possibles), ISP (ne retourne qu'un sous-set minimal des champs User)
- **Invariants** : le résultat ne contient que des users de l'organisation du syndic appelant (INV-multi-tenant).
- **FR PRD** : FR-009 (gestion tickets)
- **User Story** : En tant que François (syndic, brief section 6.1), je veux récupérer la liste des personnes à qui je peux assigner un ticket (syndics, membres du conseil) dans ma copropriété pour les afficher dans un dropdown.
- **Scénarios BDD** :

```gherkin
Scenario: Lister les cibles d'assignation pour le syndic
  Given François est connecté en tant que syndic de l'organisation "Syndic Leroy"
  And l'organisation contient 1 syndic, 2 board members et 15 owners
  When François appelle "GET /api/v1/tickets/assignable-users"
  Then la réponse 200 contient exactement 3 users (1 syndic + 2 board members)
  And chaque user contient {id, first_name, last_name, role}
  And aucun champ privé (password_hash, email personnel) n'est exposé

Scenario: Un owner n'a pas accès à cet endpoint
  Given Charlie est connecté en tant que owner
  When Charlie appelle "GET /api/v1/tickets/assignable-users"
  Then la réponse 403 Forbidden est retournée

Scenario: Isolation multi-tenant
  Given François est syndic de l'org A
  And Marie est syndic de l'org B (1 board member)
  When François appelle l'endpoint
  Then la réponse ne contient aucun user de l'org B
```

- **Taches techniques TDD** :
  1. [ ] RED : tests integration `backend/tests/integration/assignable_users_test.rs` (3 scénarios ci-dessus).
  2. [ ] GREEN : nouveau handler `list_assignable_users` dans `backend/src/infrastructure/web/handlers/ticket_handlers.rs` :
     - Guard : `require_role(&[UserRole::Syndic, UserRole::SuperAdmin])`
     - Appel : `state.user_use_cases.list_by_organization(org_id)`
     - Filtre : `role IN (Syndic, BoardMember)`
     - Serialize : `AssignableUserDto { id, first_name, last_name, role }`
  3. [ ] Créer `AssignableUserDto` dans `backend/src/application/dto/ticket_dto.rs` avec `#[derive(Serialize, ToSchema)]`.
  4. [ ] Ajouter `list_by_organization` dans `UserUseCases` si pas déjà présent (sinon réutiliser).
  5. [ ] Router : `backend/src/infrastructure/web/routes.rs` — ajouter `.service(list_assignable_users)` **avant** `/tickets/{id}` pour que la collision de paths soit évitée (rappel MEMORY.md : "ALWAYS register specific routes BEFORE parameterized routes").
  6. [ ] Ajouter le handler à `ApiDoc::openapi()` via `paths(...::list_assignable_users)`.
  7. [ ] `make openapi-export` + commit.
  8. [ ] Verifier : `cargo test --test integration` + tests spécifiques passent.
- **Dependances** : STORY-P7-101 (openapi export), STORY-P7-102 (pour que le DTO soit dans le schéma).
- **Endpoints** : `GET /api/v1/tickets/assignable-users`
- **Fichiers** : `backend/src/infrastructure/web/handlers/ticket_handlers.rs`, `backend/src/infrastructure/web/routes.rs`, `backend/src/application/dto/ticket_dto.rs`, `backend/src/infrastructure/openapi.rs`, `backend/tests/integration/assignable_users_test.rs` (nouveau).

### Story P7-2.2 : TicketAssignModal avec dropdown contractors

- **ID** : STORY-P7-202 | **Type** : Feature | **Taille** : S
- **Bounded Context DDD** : Frontend / Tickets
- **Entité(s) DDD** : `Ticket.assigned_to`
- **Principes SOLID** : SRP (le modal ne fait qu'assigner)
- **Invariants** : impossible de soumettre sans avoir sélectionné un user dans la liste chargée au mount.
- **FR PRD** : FR-009
- **User Story** : En tant que François (syndic), je veux ouvrir le modal d'assignation d'un ticket et choisir un contractor dans un dropdown listant les noms des personnes disponibles pour que je n'aie plus à connaître leur UUID.
- **Scénarios BDD** (converti en Playwright smoke test) :

```gherkin
Scenario: Le modal charge la liste au mount
  Given un ticket existe avec status Open
  And 3 board members sont disponibles dans l'organisation
  When François ouvre le modal d'assignation
  Then un dropdown avec 3 options est affiché
  And chaque option montre "First Last (syndic)" ou "First Last (board_member)"

Scenario: Soumettre l'assignation
  Given le modal est ouvert avec 3 options dans le dropdown
  When François sélectionne "Marc Dubois (board_member)"
  And clique "Assigner"
  Then une requête PUT /api/v1/tickets/{id}/assign est envoyée avec { assigned_to: "<uuid-marc>" }
  And la réponse 200 retourne le ticket avec status "Assigned"
  And le modal se ferme
  And la liste des tickets affiche le nouveau nom d'assignee

Scenario: Le bouton Assigner est désactivé si rien n'est sélectionné
  Given le modal est ouvert
  When aucune option n'est choisie
  Then le bouton "Assigner" est disabled
```

- **Taches techniques TDD** :
  1. [ ] RED : Playwright test `frontend/tests/e2e/scenarios/assign-ticket.spec.ts` (3 scénarios ci-dessus avec `humanLogin(syndic)` + `stepPause`).
  2. [ ] GREEN : réécrire `frontend/src/components/tickets/TicketAssignModal.svelte` :
     - `onMount` : `api.get<AssignableUser[]>('/tickets/assignable-users')`
     - State : `let contractors: AssignableUser[] = []; let selectedContractorId = '';`
     - UI : `<select bind:value={selectedContractorId}>` avec `<option disabled value="">{$_('tickets.assign.selectAssignee')}</option>` puis `{#each contractors as c}<option value={c.id}>{c.first_name} {c.last_name} ({$_('roles.' + c.role)})</option>{/each}`
     - Submit : `ticketsApi.assign(ticketId, selectedContractorId)`
     - Button disabled si `!selectedContractorId`
  3. [ ] Supprimer le commentaire `"In a real implementation, this would be a searchable dropdown..."`.
  4. [ ] Ajouter clés i18n dans les 4 locales :
     - `tickets.assign.selectAssignee`
     - `roles.syndic`, `roles.board_member`, `roles.owner`, `roles.accountant`, `roles.super_admin`
  5. [ ] REFACTOR : si les 25 wrappers partagent le pattern `api.get + mapping`, extraire un helper (hors scope immédiat).
  6. [ ] Verifier : `npm run build` + Playwright smoke test passent.
- **Dependances** : STORY-P7-201.
- **Endpoints** : consomme `GET /tickets/assignable-users`, `PUT /tickets/{id}/assign`.
- **Fichiers** : `frontend/src/components/tickets/TicketAssignModal.svelte` (réécriture), `frontend/src/lib/api/tickets.ts` (+ type `AssignableUser`), `frontend/src/locales/{fr,nl,de,en}.json` (+ clés roles).

### Story P7-2.3 : Seed de 2 board members techniques pour tests d'assignation

- **ID** : STORY-P7-203 | **Type** : Seed | **Taille** : XS
- **Bounded Context DDD** : Seed / Fixtures
- **Entité(s) DDD** : `User`
- **Principes SOLID** : SRP (seed reste déterministe).
- **Invariants** : les users seedés ont des identités distinctes et un rôle board_member.
- **FR PRD** : transverse (QA / démonstration).
- **User Story** : En tant que testeur (humain ou agent cowork), je veux voir 2 board members nommés "Marc Dubois (Plombier)" et "Sophie Leroux (Électricienne)" dans le dropdown d'assignation pour pouvoir tester le workflow ticket complet.
- **Scénarios BDD** :

```gherkin
Scenario: Le seed world inclut 2 board members techniques
  Given le scénario seed "world" a été exécuté
  When je liste les users de l'organisation "Syndic Leroy"
  Then au moins 1 user a email "marc-dubois@board.syndic-leroy.be" avec role "board_member"
  And au moins 1 user a email "sophie-leroux@board.syndic-leroy.be" avec role "board_member"
```

- **Taches techniques TDD** :
  1. [ ] Dans `backend/src/infrastructure/database/seed.rs`, ajouter après François Leroy (~L3286) :
     ```rust
     self.create_demo_user(
         "marc-dubois@board.syndic-leroy.be", "marc123",
         "Marc", "Dubois", "board_member", Some(org_id),
     ).await?;
     self.create_demo_user(
         "sophie-leroux@board.syndic-leroy.be", "sophie123",
         "Sophie", "Leroux", "board_member", Some(org_id),
     ).await?;
     ```
  2. [ ] Tester : `make seed` + `GET /api/v1/users` doit contenir les 2 users.
  3. [ ] Commit : `seed: add 2 technical board members for ticket assignment tests`.
- **Dependances** : aucune.
- **Endpoints** : aucun.
- **Fichiers** : `backend/src/infrastructure/database/seed.rs`.

---

## Epic P7-3 : Meeting governance UI | MUST HAVE

**Motivation** : l'audit a surfacé deux bugs sur la page de détail AG : (B18) toutes les actions syndic sont visibles aux owners (permissions UI non gated) et (B4) il n'y a aucun moyen de valider le quorum depuis l'UI alors que le backend l'exige avant de créer une résolution (Art. 3.87 §5 Code civil belge).

### Story P7-3.1 : MeetingDetail gating `canManage` sur actions syndic

- **ID** : STORY-P7-301 | **Type** : Security | **Taille** : S
- **Bounded Context DDD** : Governance / Meeting
- **Entité(s) DDD** : `Meeting`, `User.role`
- **Principes SOLID** : SRP (gating concentré en un point), defense-in-depth (le backend valide déjà les permissions, le frontend ne doit que refléter).
- **Invariants** : INV-perm "Une action syndic n'est visible que pour un user `role IN (syndic, superadmin)`".
- **FR PRD** : FR-004, FR-011 (RBAC)
- **User Story** : En tant que Charlie (owner), je ne veux **pas** voir les boutons "Marquer comme terminée / Annuler / Reporter / + Ajouter document / + Ajouter résolution" sur la page d'une AG, parce que je n'ai pas le droit métier de les déclencher.
- **Scénarios BDD** :

```gherkin
Scenario: Un owner ne voit pas les actions syndic
  Given Charlie est connecté en tant que owner
  And une AG existe avec status "Scheduled"
  When Charlie ouvre la page /meeting-detail?id=<uuid>
  Then aucun bouton "Marquer terminée", "Annuler", "Reporter" n'est affiché
  And aucun bouton "+ Ajouter un document" n'est affiché
  And aucun bouton "+ Ajouter une résolution" n'est affiché

Scenario: Un syndic voit les actions
  Given François est connecté en tant que syndic
  And la même AG existe
  When François ouvre la même page
  Then les 5 boutons d'action sont visibles
```

- **Taches techniques TDD** :
  1. [ ] RED : Playwright test `frontend/tests/e2e/scenarios/meeting-permissions.spec.ts` (2 scénarios).
  2. [ ] GREEN : dans `frontend/src/components/MeetingDetail.svelte` :
     - Importer `authStore` et dériver `$: canManage = userRole === 'syndic' || userRole === 'superadmin'`
     - Wrapper tous les blocs `{#if meeting.status === 'Scheduled'}` en `{#if canManage && meeting.status === 'Scheduled'}`
     - Même pour les boutons de résolutions et documents
  3. [ ] Verifier : ouvrir le DOM avec Charlie et vérifier que les boutons n'existent même pas dans le DOM (pas juste hidden).
  4. [ ] Commit : `fix(meetings): gate syndic actions behind canManage in MeetingDetail`.
- **Dependances** : aucune.
- **Endpoints** : aucun.
- **Fichiers** : `frontend/src/components/MeetingDetail.svelte`, `frontend/tests/e2e/scenarios/meeting-permissions.spec.ts` (nouveau).

### Story P7-3.2 : QuorumPanel pour validation Art. 3.87 §5 CC

- **ID** : STORY-P7-302 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Governance / Meeting
- **Entité(s) DDD** : `Meeting` (quorum_validated, present_quotas, total_quotas), `ValidateQuorumRequest`
- **Principes SOLID** : SRP (un composant QuorumPanel), OCP (le backend existe déjà)
- **Invariants** : INV-Art3.87 "Le quorum doit être validé avant de créer des résolutions".
- **FR PRD** : FR-004 (AG conformité belge)
- **User Story** : En tant que François (syndic), à l'ouverture d'une AG, je veux saisir combien de tantièmes sont présents ou représentés (present_quotas) pour valider le quorum conformément à l'Art. 3.87 §5 CC et débloquer la création de résolutions. Si le quorum n'est pas atteint, le système doit planifier automatiquement une 2e convocation.
- **Scénarios BDD** :

```gherkin
Scenario: Valider un quorum atteint
  Given une AG "Scheduled" avec total_quotas 1000 et quorum_validated false
  And François est connecté en tant que syndic
  When François ouvre la page meeting-detail
  Then un panneau "Quorum / Présences" est visible
  And il contient deux champs: present_quotas (vide) et total_quotas (1000)
  When François saisit present_quotas = 800
  And clique "Valider le quorum"
  Then POST /meetings/{id}/validate-quorum est appelé avec { present_quotas: 800, total_quotas: 1000 }
  And la réponse contient { quorum_reached: true }
  And le panneau affiche un badge vert "Quorum validé — 800/1000 (80.0%)"
  And les boutons de création de résolution sont maintenant actifs

Scenario: Quorum non atteint déclenche 2e convocation
  Given la même AG
  When François saisit present_quotas = 400
  And clique "Valider le quorum"
  Then la réponse contient { quorum_reached: false }
  And un message info "Seconde convocation auto-planifiée (Art. 3.87 §5 CC)" est affiché
  And le meeting status reflète cette transition

Scenario: Un owner ne peut pas valider le quorum
  Given Charlie est connecté en tant que owner
  When Charlie ouvre la même AG
  Then le panneau "Quorum / Présences" affiche uniquement un badge en lecture seule
  And aucun formulaire n'est présent
```

- **Taches techniques TDD** :
  1. [ ] RED : Playwright test `frontend/tests/e2e/scenarios/quorum-validation.spec.ts` (3 scénarios ci-dessus avec multi-rôles).
  2. [ ] GREEN : créer `frontend/src/components/meetings/QuorumPanel.svelte` :
     - Props : `meeting: Meeting, canManage: boolean`
     - onMount : pré-remplir `total_quotas` depuis `meeting.total_quotas ?? 1000`
     - Si `meeting.quorum_validated` : badge vert `Quorum validé — {present}/{total} ({percentage}%)`
     - Sinon + `canManage` : `<form>` avec 2 inputs `present_quotas` / `total_quotas` + bouton "Valider le quorum"
     - Submit : `meetingsApi.validateQuorum(meeting.id, { present_quotas, total_quotas })`
     - Si réponse `quorum_reached: false` : toast info "Seconde convocation auto-planifiée"
  3. [ ] Ajouter `validateQuorum` dans `frontend/src/lib/api/meetings.ts` (ou créer le fichier s'il n'existe pas).
  4. [ ] Injecter `<QuorumPanel meeting={meeting} {canManage} />` dans `MeetingDetail.svelte` **au-dessus** de la section résolutions.
  5. [ ] Ajouter clés i18n FR/NL/DE/EN :
     - `meetings.quorum.title` "Quorum / Présences"
     - `meetings.quorum.presentQuotas` "Tantièmes présents ou représentés"
     - `meetings.quorum.totalQuotas` "Tantièmes totaux"
     - `meetings.quorum.validate` "Valider le quorum"
     - `meetings.quorum.validated` "Quorum validé — {present}/{total} ({pct}%)"
     - `meetings.quorum.failed` "Seconde convocation auto-planifiée (Art. 3.87 §5 CC)"
     - `meetings.quorum.readonly` "En attente de validation par le syndic"
  6. [ ] REFACTOR : s'assurer que le gating de STORY-P7-301 empêche Charlie de voir le formulaire.
  7. [ ] Verifier : seed une AG + ouvrir en syndic + valider quorum → le bouton "Ajouter résolution" s'active.
  8. [ ] Commit : `feat(meetings): add QuorumPanel for Art. 3.87 §5 CC validation`.
- **Dependances** : STORY-P7-301 (pour le `canManage`), STORY-P7-102 (pour que `ValidateQuorumRequest` soit dans le schéma OpenAPI).
- **Endpoints** : consomme `POST /api/v1/meetings/{id}/validate-quorum` (déjà existant).
- **Fichiers** : `frontend/src/components/meetings/QuorumPanel.svelte` (nouveau, ~120 LOC), `frontend/src/components/MeetingDetail.svelte` (injection), `frontend/src/lib/api/meetings.ts` (+ méthode), `frontend/src/locales/{fr,nl,de,en}.json` (+ 7 clés), `frontend/tests/e2e/scenarios/quorum-validation.spec.ts` (nouveau).

---

## Epic P7-4 : Error UX hardening | SHOULD HAVE

**Motivation** : v7 a observé que certains toasts API remontent encore en anglais brut (`API Error: 404`, `No owner record linked to this user`) et que les toasts ne s'auto-ferment pas, ce qui produit des empilements. La masquage DB leak (#B13) est déjà en place via le commit `268a24a` mais ne couvre pas les messages applicatifs.

### Story P7-4.1 : Mapping i18n des erreurs API backend courantes

- **ID** : STORY-P7-401 | **Type** : UX | **Taille** : S
- **Bounded Context DDD** : Frontend / Error UX
- **Entité(s) DDD** : N/A
- **Principes SOLID** : OCP (le mapping est extensible), SRP (un point unique dans apiFetch)
- **Invariants** : aucun message backend brut ne doit apparaître à l'utilisateur.
- **FR PRD** : FR-011 (i18n), transverse UX
- **User Story** : En tant qu'utilisateur francophone, je veux que les toasts d'erreur soient traduits et compréhensibles (jamais "API Error: 404" ou "No owner record linked to this user").
- **Scénarios BDD** :

```gherkin
Scenario: Traduire une 404 en message utilisateur
  Given un composant appelle GET /api/v1/nonexistent
  When l'erreur 404 est reçue
  Then le toast affiche "Ressource introuvable" (FR) ou "Resource not found" (EN) selon la locale

Scenario: Traduire une erreur métier connue
  Given un user sans owner appelle /owners/me
  When le backend retourne 404 avec { error: "No owner record linked to this user" }
  Then le toast affiche "Aucun compte copropriétaire associé à ce profil"

Scenario: Fallback pour un message inconnu
  Given le backend retourne 500 avec un message non mappé
  When l'erreur est reçue
  Then le toast affiche "Erreur serveur. Veuillez réessayer." (générique)
```

- **Taches techniques TDD** :
  1. [ ] Créer `frontend/src/lib/api-errors.ts` avec une map `Record<string, string>` pour les messages backend connus.
  2. [ ] Dans `frontend/src/lib/api.ts`, après le masquage DB errors existant, ajouter une étape : si `errorMessage` correspond à une clé de la map, remplacer par `$_('api.errors.' + key)`.
  3. [ ] Ajouter bloc `api.errors.*` dans les 4 locales avec au moins :
     - `notFound` "Ressource introuvable"
     - `noOwnerLinked` "Aucun compte copropriétaire associé à ce profil"
     - `forbidden` "Accès refusé"
     - `genericServer` "Erreur serveur. Veuillez réessayer."
     - `tooManyRequests` "Trop de tentatives. Réessayez dans 15 minutes."
  4. [ ] Remplacer les chaînes FR hardcodées du fichier api.ts actuel par ces clés.
  5. [ ] Commit : `feat(api): map backend error messages to i18n keys`.
- **Dependances** : aucune.
- **Endpoints** : aucun.
- **Fichiers** : `frontend/src/lib/api.ts`, `frontend/src/lib/api-errors.ts` (nouveau), 4 locales.

### Story P7-4.2 : Toast store auto-close + dedup

- **ID** : STORY-P7-402 | **Type** : UX | **Taille** : S
- **Bounded Context DDD** : Frontend / UI store
- **Entité(s) DDD** : N/A
- **Principes SOLID** : SRP (le store gère la dédup et le timeout)
- **Invariants** : jamais plus d'une instance d'un toast identique (même message + même type) dans la pile.
- **FR PRD** : transverse UX
- **User Story** : En tant qu'utilisateur, je veux que les toasts s'auto-ferment après 5 secondes et ne se dupliquent pas quand plusieurs erreurs identiques surviennent en parallèle.
- **Scénarios BDD** :

```gherkin
Scenario: Auto-close après 5 secondes
  Given un toast info "Action effectuée" est poussé
  When 5 secondes s'écoulent
  Then le toast disparaît automatiquement

Scenario: Dedup d'un toast identique
  Given un toast error "Erreur serveur" est déjà dans la pile
  When un second toast error avec le même message est poussé
  Then seule une instance est visible
  And l'auto-close timer est réinitialisé à 5 secondes

Scenario: Les toasts différents coexistent
  Given un toast info "Sauvegardé" est dans la pile
  When un toast error "Erreur de chargement" est poussé
  Then les 2 toasts sont visibles simultanément
```

- **Taches techniques TDD** :
  1. [ ] RED : tests unitaires `frontend/src/stores/toast.test.ts` (3 scénarios).
  2. [ ] GREEN : refactorer `frontend/src/stores/toast.ts` :
     - Le store contient `Array<{ id, message, type, createdAt }>`
     - `push({message, type})` : si un toast avec même `message+type` existe, réinitialiser son `createdAt` (au lieu de pousser un doublon). Sinon, créer et `setTimeout(() => remove(id), 5000)`.
     - `remove(id)` : filtre le store.
  3. [ ] Verifier avec `npm run test` ou équivalent.
  4. [ ] Commit : `feat(ui): toast store auto-close 5s + dedup identical messages`.
- **Dependances** : aucune.
- **Endpoints** : aucun.
- **Fichiers** : `frontend/src/stores/toast.ts`, `frontend/src/stores/toast.test.ts` (nouveau).

### Story P7-4.3 : "Back to Tickets" traduction

- **ID** : STORY-P7-403 | **Type** : i18n | **Taille** : XS
- **Bounded Context DDD** : Frontend / Navigation
- **User Story** : En tant qu'utilisateur francophone, je veux que le lien de retour depuis la page détail ticket affiche "Retour aux tickets" et non "Back to Tickets".
- **Taches techniques TDD** :
  1. [ ] Grep `"Back to Tickets"` dans `frontend/src/pages/ticket-detail.astro` et composants associés.
  2. [ ] Remplacer par `$_('common.backToTickets')`.
  3. [ ] Ajouter clé dans les 4 locales (`FR: "Retour aux tickets"`, `NL: "Terug naar tickets"`, etc.)
  4. [ ] Commit : `i18n: translate 'Back to Tickets' link`.
- **Dependances** : aucune.
- **Endpoints** : aucun.
- **Fichiers** : `frontend/src/pages/ticket-detail.astro`, 4 locales.

---

## Epic P7-5 : Testing hygiene | COULD HAVE

**Motivation** : le v7 a épinglé un "bug" B16 ("Svelte 5 bind:value non réactif via form_input programmatique") qui n'en est pas un : c'est l'outil d'automation qui n'émet pas d'`InputEvent` après avoir utilisé le setter DOM natif. Un utilisateur humain qui tape dans un input n'a pas ce problème. Il faut documenter le pattern correct pour que les futurs audits cowork n'inventent plus de bugs à partir de cette limitation.

### Story P7-5.1 : Documenter le pattern de saisie programmatique Svelte 5

- **ID** : STORY-P7-501 | **Type** : Docs | **Taille** : XS
- **Bounded Context DDD** : Documentation / QA
- **User Story** : En tant qu'auteur de prompt cowork, je veux savoir comment simuler une saisie utilisateur dans un formulaire Svelte 5 via MCP automation pour que mes rapports ne génèrent pas de faux positifs "submit silencieux".
- **Taches techniques TDD** :
  1. [ ] Créer `docs/TESTING_SVELTE5.md` avec :
     - Explication : en mode legacy (`export let` + `$:`), `input.value = "x"` ne déclenche pas `bind:value`.
     - Pattern recommandé : setter natif + `dispatchEvent(new InputEvent('input', { bubbles: true }))`.
     - Snippet JS complet copiable.
     - Alternative : utiliser `user.type(input, ...)` via `@testing-library/user-event` ou Playwright `page.fill()` natif qui dispatch l'event correctement.
     - Lien vers l'issue upstream Svelte si pertinent.
  2. [ ] Référencer ce doc depuis `CLAUDE.md` section "Testing Philosophy" et depuis les prompts cowork.
  3. [ ] Commit : `docs(testing): document Svelte 5 programmatic input pattern`.
- **Dependances** : aucune.
- **Fichiers** : `docs/TESTING_SVELTE5.md` (nouveau), `CLAUDE.md` (section testing).

---

## Synthèse traçabilité Epic → FR PRD → Bug audit

| Epic | Story | Taille | FR PRD | Bug audit v7 | Dépendances | Statut |
|------|-------|--------|--------|--------------|-------------|--------|
| P7-1 | STORY-P7-101 (openapi-export bin) | S | transverse | — (qualité) | — | PENDING |
| P7-1 | STORY-P7-102 (annotations DTO) | M | FR-004,006,008,012 | — | P7-101 | PENDING |
| P7-1 | STORY-P7-103 (frontend re-exports) | S | transverse | prévention v4-v7 | P7-101, P7-102 | PENDING |
| P7-1 | STORY-P7-104 (CI guard) | S | transverse | — (régression) | P7-101 | PENDING |
| P7-2 | STORY-P7-201 (GET assignable-users) | S | FR-009 | B14 | P7-101, P7-102 | PENDING |
| P7-2 | STORY-P7-202 (dropdown modal) | S | FR-009 | B14 | P7-201 | PENDING |
| P7-2 | STORY-P7-203 (seed board members) | XS | transverse | B14 | — | PENDING |
| P7-3 | STORY-P7-301 (canManage gating) | S | FR-004, FR-011 | B18 | — | PENDING |
| P7-3 | STORY-P7-302 (QuorumPanel) | M | FR-004 (Art. 3.87 §5) | B4 (déféré v6) | P7-301, P7-102 | PENDING |
| P7-4 | STORY-P7-401 (error i18n map) | S | FR-011 | B17 | — | PENDING |
| P7-4 | STORY-P7-402 (toast dedup + auto-close) | S | transverse | B21 | — | PENDING |
| P7-4 | STORY-P7-403 (Back to Tickets i18n) | XS | FR-011 | B19 | — | PENDING |
| P7-5 | STORY-P7-501 (doc Svelte 5) | XS | — | B16 | — | PENDING |

**Total effort estimé** (T-shirt sizing) :
- XS (≤ 1h) : 3 stories = 3h
- S (1-3h) : 7 stories = 14-21h
- M (3-6h) : 3 stories = 9-18h

**Fourchette totale** : 26-42h (3-5 jours à temps plein pour un dev seul).

---

## Ordre d'exécution recommandé

Les dépendances imposent l'ordre suivant pour minimiser les blocages :

```
Sprint Post-Audit (1 semaine)

Jour 1 — Fondation type-safety
├─ STORY-P7-101 (openapi-export bin)         [S]
├─ STORY-P7-102 (annotations DTO)            [M]
└─ STORY-P7-104 (CI guard)                   [S]

Jour 2 — Migration frontend types
└─ STORY-P7-103 (re-exports)                 [S]

Jour 3 — Ticket assignment
├─ STORY-P7-203 (seed board members)         [XS]
├─ STORY-P7-201 (GET assignable-users)       [S]
└─ STORY-P7-202 (dropdown modal)             [S]

Jour 4 — Meeting governance
├─ STORY-P7-301 (canManage gating)           [S]
└─ STORY-P7-302 (QuorumPanel)                [M]

Jour 5 — UX polish + docs
├─ STORY-P7-402 (toast dedup)                [S]
├─ STORY-P7-401 (error i18n)                 [S]
├─ STORY-P7-403 (Back to Tickets)            [XS]
└─ STORY-P7-501 (doc Svelte 5)               [XS]
```

Chaque fin de journée : `make openapi-export && make pre-commit && git commit` thématique, puis `git push` sur `feature/dev`.

Fin de semaine : prompt cowork v8 "verify all 13 stories" avec scénarios BDD → si vert, merge `feature/dev` → `main` et tag éventuel.

---

## Invariants Maury respectés

- ✅ **DDD** : chaque story nomme son Bounded Context et ses entités.
- ✅ **BDD** : chaque story fonctionnelle contient des scénarios Gherkin exécutables.
- ✅ **TDD** : chaque story liste les tâches RED → GREEN → REFACTOR.
- ✅ **SOLID** : chaque story cite les principes qui la gouvernent.
- ✅ **Traçabilité** : chaque story trace vers un FR PRD ou un bug audit identifié.
- ✅ **Hexagonal** : les stories backend séparent Domain / Application / Infrastructure.
- ✅ **YAGNI** : limité à ce que v7 demande, pas de spéculation.
- ✅ **DRY** : les enums deviennent générés (pas 2 sources de vérité).

## Ce qui N'EST PAS dans ce plan

Hors scope explicite, pour rester dans une semaine :

- Migration vers les runes Svelte 5 (`$state`, `$derived`, `$props`) — refacto majeur, à planifier en Epic séparé.
- Annotation utoipa à 100 % (seuls les DTOs des 4 modules v7-critiques sont ciblés).
- Refonte des 25 wrappers `lib/api/*.ts` (seuls 3 en Story P7-103).
- Nettoyage du seed pollué (1628 orgs `workreport Org …`) — data, pas code.
- Accessibilité WCAG (focus trap, aria-labels) — nice-to-have, Epic futur.
- Contractor comme rôle domaine distinct — réutilise `BoardMember`, voir Story P7-2 Option A.

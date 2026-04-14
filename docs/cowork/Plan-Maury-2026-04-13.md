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

## Epic P7-6 : Svelte 5 runes migration (178 composants) | SHOULD HAVE

**Motivation** : le projet tourne en **Svelte 5 legacy mode** (`export let` + `$:` + `createEventDispatcher`) au lieu des runes modernes (`$state`, `$props`, `$derived`, `$effect`). Ce mode legacy est la cause directe du bug B3-v7 (building_id perdu parce que `let formData = { building_id: buildingId }` capture le prop **avant** que `mount()` ne l'assigne). La migration vers les runes élimine cette classe entière de bugs de "race d'initialisation" et aligne le code avec l'évolution upstream du framework.

**Stratégie** : migration par vagues. Commencer par un composant pilote pour valider la compatibilité avec Astro islands, puis batcher par domaine fonctionnel. Chaque story est un commit atomique et la suite de tests (unitaire + Playwright E2E) doit rester verte entre chaque.

**Coverage cible** : 100 % des 178 composants `.svelte` migrés. Pas de mode hybride en fin de parcours.

### Story P7-6.1 : Activer le mode runes + migration pilote sur TicketCreateModal

- **ID** : STORY-P7-601 | **Type** : Refactor | **Taille** : M
- **Bounded Context DDD** : Frontend / Reactivity
- **Entité(s) DDD** : N/A
- **Principes SOLID** : OCP (le reste du code n'est pas modifié), DIP (les composants migrés continuent d'implémenter les mêmes "ports" — props, events)
- **Invariants** : INV-pilot "Le composant pilote doit passer tous ses tests Playwright existants avant d'étendre la migration".
- **FR PRD** : transverse (infrastructure frontend)
- **User Story** : En tant que développeur frontend, je veux que `TicketCreateModal.svelte` utilise `$props()` + `$state()` pour supprimer la garde défensive `if (!formData.building_id && buildingId)` introduite par le commit 241a336, parce que cette garde est un workaround du mode legacy qui disparaît avec les runes.
- **Scénarios BDD** (Playwright, réutilisation de ticket-create.spec.ts) :

```gherkin
Scenario: Le modal reçoit building_id correctement au premier tick
  Given le projet est configuré avec runes = true
  And TicketCreateModal utilise $props() et $state()
  When un test Playwright ouvre le modal avec prop buildingId="<uuid>"
  Then dans les 100ms suivant le mount, l'input caché du formulaire contient l'UUID
  And aucun fallback défensif n'est déclenché

Scenario: Le composant est rétrocompatible avec son parent
  Given la page /tickets monte TicketCreateModal via Svelte 5 mount()
  When l'utilisateur soumet le formulaire
  Then dispatch("created", result) et onCreated?.(result) continuent de fonctionner
  And la suite de tests tickets E2E reste à 283/283 green
```

- **Taches techniques TDD** :
  1. [ ] Créer `frontend/svelte.config.js` avec `compilerOptions: { runes: true }`. Vérifier que Astro `@astrojs/svelte` prend en compte ce fichier.
  2. [ ] RED : lancer la suite existante `frontend/tests/e2e/tickets/*.spec.ts` — doit encore passer (0 régression) avant migration.
  3. [ ] Migrer [TicketCreateModal.svelte](frontend/src/components/tickets/TicketCreateModal.svelte) :
     - `export let open = false;` → `let { open = $bindable(false), buildingId = '', requesterId, unitId, onCreated, onClose } = $props();`
     - `let formData: CreateTicketDto = { building_id: buildingId, ... }` → `let formData = $state({ building_id: buildingId, ... })`
     - `$: if (open && buildings.length === 0) loadBuildings();` → `$effect(() => { if (open && buildings.length === 0) loadBuildings(); });`
     - Supprimer la garde défensive `if (!formData.building_id && buildingId)` dans handleSubmit (les runes captent la valeur finale du prop).
  4. [ ] Supprimer `createEventDispatcher` et sa variable `dispatch` ; remplacer par les callback props `onCreated`/`onClose` déjà exposés.
  5. [ ] GREEN : re-exécuter la suite Playwright tickets — 283/283 green attendu.
  6. [ ] REFACTOR : nettoyer les commentaires "workaround Svelte 5 mount" du fichier.
  7. [ ] Documenter le pattern de migration dans `docs/MIGRATION_SVELTE5_RUNES.md` (modèle pour les 177 autres composants).
  8. [ ] Commit : `refactor(svelte5): pilot runes migration on TicketCreateModal (+ docs pattern)`.
- **Dependances** : STORY-P7-501 (pour avoir le contexte "legacy → runes" documenté).
- **Endpoints** : aucun.
- **Fichiers** : `frontend/svelte.config.js` (nouveau), `frontend/src/components/tickets/TicketCreateModal.svelte`, `docs/MIGRATION_SVELTE5_RUNES.md` (nouveau).

### Story P7-6.2 : Migrer les composants tickets (7 fichiers)

- **ID** : STORY-P7-602 | **Type** : Refactor | **Taille** : M
- **Bounded Context DDD** : Frontend / Tickets
- **Principes SOLID** : SRP (chaque composant garde sa responsabilité), LSP (même contrat de props)
- **User Story** : En tant que développeur, je veux que tous les composants tickets utilisent les runes pour avoir un code uniforme dans le module.
- **Taches techniques TDD** :
  1. [ ] Migrer dans l'ordre : `TicketList.svelte`, `TicketDetail.svelte`, `TicketAssignModal.svelte`, `TicketStatusBadge.svelte`, `TicketPriorityBadge.svelte`, `TicketStatistics.svelte`.
  2. [ ] Pattern systématique : `export let X` → `let { X } = $props()`, `let Y = ...` → `let Y = $state(...)`, `$: Z = ...` → `let Z = $derived(...)` ou `$effect()`.
  3. [ ] Pour chaque fichier : lancer `npx svelte-check` et les tests Playwright pertinents.
  4. [ ] Commit groupé : `refactor(svelte5): migrate ticket components to runes`.
- **Dependances** : STORY-P7-601.
- **Fichiers** : 6 fichiers `frontend/src/components/tickets/*.svelte`.

### Story P7-6.3 : Migrer les composants meetings (7 fichiers)

- **ID** : STORY-P7-603 | **Type** : Refactor | **Taille** : M
- **Fichiers** : `MeetingCreateModal.svelte`, `MeetingDetail.svelte`, `MeetingList.svelte`, `MeetingDocuments.svelte`, `ConvocationPanel.svelte`, `ConvocationRecipientList.svelte`, nouveau `QuorumPanel.svelte` (si P7-302 déjà posé).
- **Dependances** : STORY-P7-601, STORY-P7-302 (pour que QuorumPanel soit directement en runes).
- **Commit** : `refactor(svelte5): migrate meeting components to runes`.

### Story P7-6.4 : Migrer les composants resolutions (4 fichiers)

- **ID** : STORY-P7-604 | **Type** : Refactor | **Taille** : S
- **Fichiers** : `ResolutionCreateForm.svelte`, `ResolutionVotePanel.svelte`, `ResolutionList.svelte`, `ResolutionStatusBadge.svelte`.
- **Dependances** : STORY-P7-601.

### Story P7-6.5 : Migrer les composants expenses + invoices (5 fichiers)

- **ID** : STORY-P7-605 | **Type** : Refactor | **Taille** : M
- **Fichiers** : `InvoiceForm.svelte`, `InvoiceLineItems.svelte`, `ExpenseList.svelte`, `ExpenseDetail.svelte`, `ExpenseDocuments.svelte`.
- **Impact** : supprime la 2e garde défensive (`if (!selectedBuildingId && buildingId)`) introduite dans commit 841e47c.

### Story P7-6.6 : Migrer les composants polls + notices + SEL (15 fichiers)

- **ID** : STORY-P7-606 | **Type** : Refactor | **Taille** : L
- **Fichiers** : tous les composants sous `polls/`, `notices/`, `local-exchanges/`.

### Story P7-6.7 : Migrer les composants admin + dashboards + navigation (20 fichiers)

- **ID** : STORY-P7-607 | **Type** : Refactor | **Taille** : L
- **Fichiers** : `admin/*`, `dashboards/*`, `Navigation.svelte`, `NotificationBell.svelte`, etc.

### Story P7-6.8 : Migrer les composants restants (130+ fichiers, par lots de 20)

- **ID** : STORY-P7-608 | **Type** : Refactor | **Taille** : XL (scriptable)
- **Strategy** : script codemod semi-automatique (`jscodeshift` ou `svelte-migrate`) appliqué par dossier avec revue humaine.
- **Checklist** :
  1. [ ] Écrire `scripts/migrate-to-runes.js` qui applique les transformations mécaniques (export let → $props, $: → $derived).
  2. [ ] Lancer sur un dossier `frontend/src/components/{budgets,etats-dates,gamification,iot,journal-entries,work-reports,ag-sessions,age-requests,board,contractor-reports,convocations,documents,energy,inspections,payments,resource-bookings,sharing,skills,quotes,two-factor,...}`.
  3. [ ] `npx svelte-check` après chaque lot.
  4. [ ] Commits atomiques par dossier.
- **Dependances** : STORY-P7-607 (pattern rodé sur les composants complexes).

### Story P7-6.9 : Supprimer toute trace de legacy reactivity

- **ID** : STORY-P7-609 | **Type** : Cleanup | **Taille** : S
- **Invariant** : INV-no-legacy "Aucun `export let`, `$:`, `createEventDispatcher` dans le codebase frontend".
- **Taches** :
  1. [ ] `grep -r "export let" frontend/src/components` → 0 match attendu.
  2. [ ] `grep -r "^\s*\$:" frontend/src/components` → 0 match.
  3. [ ] `grep -r "createEventDispatcher" frontend/src` → 0 match (sauf doc).
  4. [ ] Supprimer les suppressions Svelte legacy dans `svelte.config.js`.
  5. [ ] Commit : `chore(svelte5): complete runes migration, no legacy reactivity remains`.

---

## Epic P7-7 : Coverage utoipa 100 % + refactor des 25 wrappers API frontend | SHOULD HAVE

**Motivation** : l'Epic P7-1 annote seulement les DTOs critiques des 4 modules v7-impliqués. Mais le projet a 52 DTOs + 62 entités + 511 endpoints. Chaque bloc non annoté reste un risque de mismatch futur. La méthode Maury exige la cohérence totale : si la spec OpenAPI est notre source de vérité, elle doit couvrir 100 % de la surface API.

De plus, le frontend a 25 wrappers `lib/api/*.ts` hand-written. Seuls 3 sont migrés dans P7-103. Les 22 autres restent des sources potentielles de bugs d'enum mismatch.

### Story P7-7.1 : Annoter les 41 DTOs restants

- **ID** : STORY-P7-701 | **Type** : Refactor | **Taille** : L
- **Bounded Context DDD** : tous les modules applicatifs
- **Entité(s) DDD** : tous les DTOs de requête et de réponse
- **Principes SOLID** : OCP (annotations additives), ISP (chaque DTO expose son propre schéma)
- **Invariants** : INV-100percent "Tout struct `#[derive(Serialize)]` ou `#[derive(Deserialize)]` dans `application/dto/` doit aussi dériver `utoipa::ToSchema`".
- **User Story** : En tant que développeur, je veux que 100 % des DTOs soient annotés pour que n'importe quel endpoint soit générable en TypeScript sans intervention manuelle.
- **Fichiers cibles** (41) : `account_dto.rs`, `ag_session_dto.rs`, `age_request_dto.rs`, `annual_report_dto.rs`, `board_decision_dto.rs`, `board_member_dto.rs`, `budget_dto.rs`, `building_dto.rs`, `charge_distribution_dto.rs`, `consent_dto.rs`, `contractor_report_dto.rs`, `convocation_dto.rs`, `convocation_recipient_dto.rs`, `document_dto.rs`, `energy_bill_upload_dto.rs`, `energy_campaign_dto.rs`, `etat_date_dto.rs`, `exchange_dto.rs`, `financial_report_dto.rs`, `gamification_dto.rs`, `inspection_dto.rs`, `iot_dto.rs`, `journal_entry_dto.rs`, `local_exchange_dto.rs`, `marketplace_dto.rs`, `meeting_dto.rs`, `notice_dto.rs`, `organization_dto.rs`, `owner_dto.rs`, `owner_contribution_dto.rs`, `payment_reminder_dto.rs`, `poll_dto.rs`, `public_dto.rs`, `quote_dto.rs`, `resource_booking_dto.rs`, `sel_dto.rs`, `sharing_dto.rs`, `skill_dto.rs`, `two_factor_dto.rs`, `unit_dto.rs`, `unit_owner_dto.rs`, `user_dto.rs`, `work_report_dto.rs`.
- **Taches techniques TDD** :
  1. [ ] Pour chaque fichier : ajouter `utoipa::ToSchema` au `#[derive(...)]` existant sur tous les structs publics.
  2. [ ] Ajouter les enums de domaine référencés (ex. `BudgetStatus`, `PollType`, etc.) dans `components(schemas(...))` de `ApiDoc`.
  3. [ ] `make openapi-export` après chaque groupe de 5 fichiers et vérifier que `openapi.json` croît sans erreur.
  4. [ ] `cargo check` pour s'assurer qu'aucun DTO ne casse la compilation utoipa (certains types complexes peuvent nécessiter `#[schema(value_type = String)]`).
  5. [ ] Commits batchés par module : `refactor(api): annotate <module> DTOs with ToSchema`.
- **Dependances** : STORY-P7-101, STORY-P7-102.
- **Fichiers** : 41 fichiers dans `backend/src/application/dto/`.

### Story P7-7.2 : Annoter les 54 entités de domaine restantes

- **ID** : STORY-P7-702 | **Type** : Refactor | **Taille** : L
- **Scope** : toutes les entités domain qui apparaissent dans une réponse API (ex. `Building`, `Unit`, `Owner`, `Expense`, `Budget`, etc.).
- **Taches** :
  1. [ ] Pour chaque entité : `#[derive(..., utoipa::ToSchema)]`.
  2. [ ] Traiter les enums de statut (`BudgetStatus`, `PollStatus`, etc.) avec attention au `rename_all = "snake_case"` s'il est présent.
  3. [ ] Tester à nouveau `make openapi-export`.
- **Dependances** : STORY-P7-701.
- **Fichiers** : 54 entités dans `backend/src/domain/entities/`.

### Story P7-7.3 : Annoter les ~420 handlers restants avec `#[utoipa::path]`

- **ID** : STORY-P7-703 | **Type** : Refactor | **Taille** : XL (scriptable)
- **Scope** : 511 endpoints - 90 déjà annotés = 421 à traiter.
- **Stratégie** :
  1. [ ] Script de détection : `grep -rL "#\[utoipa::path" backend/src/infrastructure/web/handlers/ --include="*.rs"` liste les handlers manquants.
  2. [ ] Template d'annotation par type d'endpoint (GET list, GET by id, POST create, PUT update, DELETE).
  3. [ ] Annoter par module (ticket, poll, resolution, meeting, etc.) pour commits atomiques.
  4. [ ] Ajouter chaque nouveau path à `ApiDoc::openapi()` paths(...).
  5. [ ] `make openapi-export` + `make openapi-check` en fin de chaque module.
- **Dependances** : STORY-P7-701, STORY-P7-702.
- **Estimation** : ~10h de travail mécanique.
- **Commits** : 1 par module, format `docs(api): annotate <module> handlers with utoipa::path`.

### Story P7-7.4 : Migrer les 22 wrappers frontend restants vers les types générés

- **ID** : STORY-P7-704 | **Type** : Refactor | **Taille** : L
- **Scope** : tous les fichiers sous `frontend/src/lib/api/*.ts` sauf ceux déjà migrés par STORY-P7-103 (`resolutions`, `tickets`, `polls`).
- **Fichiers cibles** (22) : `ag-sessions`, `age-requests`, `bookings`, `budgets`, `charge-distributions`, `convocations`, `energy-campaigns`, `etats-dates`, `gamification`, `inspections`, `local-exchanges`, `marketplace`, `notices`, `notifications`, `payment-reminders`, `payments`, `quotes`, `sel`, `sharing`, `skills`, `tickets`, `work-reports`.
- **Pattern** (extrait de STORY-P7-103) :
  ```ts
  import type { components } from "../../types/api";
  export type Foo = components["schemas"]["Foo"];
  export const Foo = { Bar: "bar" as const, Baz: "baz" as const } satisfies Record<string, Foo>;
  ```
- **Taches** :
  1. [ ] Pour chaque wrapper : grep les `export enum` et `export interface` puis les remplacer par des re-exports.
  2. [ ] `npx svelte-check` après chaque fichier.
  3. [ ] Commits atomiques : `refactor(frontend): re-export <module> types from api.d.ts`.
- **Dependances** : STORY-P7-701, STORY-P7-702 (pour que tous les types soient dans `api.d.ts`).

### Story P7-7.5 : Garde d'invariant "no hand-written enum" en CI

- **ID** : STORY-P7-705 | **Type** : Infra | **Taille** : S
- **Invariant** : INV-api-types "Aucun `export enum` dans `frontend/src/lib/api/` (hors fichiers explicitement exclus comme mcp.ts qui est custom)".
- **Tache** :
  1. [ ] Script `scripts/check-api-enums.sh` qui grep `export enum` dans `lib/api/` et exit 1 si match.
  2. [ ] Ajouter à la CI après `npm run build`.
  3. [ ] Ajouter au pre-commit hook.
- **Dependances** : STORY-P7-704.

---

## Epic P7-8 : Seed determinism + cleanup pollution historique | MUST HAVE

**Motivation** : les audits ont révélé une pollution massive du seed (1628 organisations `workreport Org …`, 1773 users, 1278 immeubles) qui rend les tests admin inutilisables et pollue les dashboards. Ce n'est pas "juste de la data" — c'est un défaut de design du scénario de seed qui ne nettoie pas avant d'insérer.

### Story P7-8.1 : Scénario seed `world` idempotent

- **ID** : STORY-P7-801 | **Type** : Refactor | **Taille** : S
- **Bounded Context DDD** : Seed / Fixtures
- **Entité(s) DDD** : toutes (Organization, User, Building, ...)
- **Invariants** : INV-seed-idempotent "Exécuter `POST /seed/scenario/world` N fois produit exactement les mêmes entités et le même nombre de lignes dans chaque table".
- **User Story** : En tant que testeur, je veux pouvoir relancer le seed autant de fois que je veux sans polluer la base de données.
- **Scénarios BDD** :

```gherkin
Scenario: Le seed world est idempotent
  Given la base contient déjà le scénario world
  When "POST /seed/scenario/world" est appelé
  Then la réponse est 200 (pas 409 ni 500)
  And le nombre de lignes dans orgs/users/buildings reste identique à avant l'appel

Scenario: Le seed world nettoie automatiquement les doublons historiques
  Given la base contient 1628 organisations incluant 1625 doublons "workreport Org XXX"
  When "POST /seed/scenario/world" est appelé
  Then seulement 3 organisations restent (celles du scénario officiel)
  And les 1625 doublons sont supprimés en cascade (users, buildings, etc.)
```

- **Taches techniques TDD** :
  1. [ ] RED : test integration `backend/tests/integration/seed_idempotent_test.rs` qui :
     - Appelle `seed_world` 3 fois
     - Vérifie que `COUNT(*)` est stable
     - Vérifie qu'aucune FK violation n'est levée
  2. [ ] GREEN : dans `backend/src/infrastructure/database/seed.rs`, modifier `seed_world()` :
     - Étape 1 : `DELETE FROM <toutes tables> WHERE organization_id IN (SELECT id FROM organizations WHERE slug IN ('syndic-leroy', 'residence-parc', 'terrasses-flagey'))` — en respectant l'ordre FK.
     - Étape 2 : insertions normales.
     - Wrapper dans une transaction.
  3. [ ] Ajouter endpoint `DELETE /api/v1/seed/scenario/world` qui appelle la partie cleanup isolée.
  4. [ ] Commit : `feat(seed): make world scenario idempotent (clean + reinsert)`.
- **Dependances** : aucune.
- **Endpoints** : `POST /api/v1/seed/scenario/world` (comportement changé), `DELETE /api/v1/seed/scenario/world` (nouveau).
- **Fichiers** : `backend/src/infrastructure/database/seed.rs`, `backend/src/infrastructure/web/handlers/seed_handlers.rs`, `backend/tests/integration/seed_idempotent_test.rs` (nouveau).

### Story P7-8.2 : Migration SQL de cleanup pollution historique

- **ID** : STORY-P7-802 | **Type** : Migration | **Taille** : S
- **Bounded Context DDD** : Database maintenance
- **Invariants** : INV-clean "Après exécution, aucune table ne contient d'entité `workreport Org …`".
- **User Story** : En tant qu'opérateur, je veux nettoyer en un seul script la pollution historique accumulée par les seeds précédents.
- **Taches** :
  1. [ ] Créer migration `backend/migrations/[timestamp]_cleanup_workreport_pollution.sql` :
     ```sql
     -- Delete test pollution left by earlier audit seeds
     DELETE FROM organizations WHERE name LIKE 'workreport Org %' OR slug LIKE 'unitowner-%' OR slug LIKE '2fa-%';
     -- FK cascade cleans users, buildings, etc.
     ```
  2. [ ] Tester sur une copie de dev : `sqlx migrate run` + comparer counts avant/après.
  3. [ ] Commit : `migration: cleanup workreport seed pollution from prior audit runs`.
- **Dependances** : aucune.
- **Fichiers** : `backend/migrations/[timestamp]_cleanup_workreport_pollution.sql`.

### Story P7-8.3 : `make seed-reset` target

- **ID** : STORY-P7-803 | **Type** : Infra | **Taille** : XS
- **Tache** :
  1. [ ] Dans `Makefile` :
     ```makefile
     seed-reset: ## Reset the database to a clean world seed
         @curl -s -X POST http://localhost/api/v1/auth/login -H 'Content-Type: application/json' -d '{"email":"admin@koprogo.com","password":"admin123"}' | jq -r .token > /tmp/.koprogo-token
         @curl -s -X POST http://localhost/api/v1/seed/scenario/world -H "Authorization: Bearer $$(cat /tmp/.koprogo-token)"
         @rm /tmp/.koprogo-token
         @echo "✅ Seed reset complete"
     ```
- **Dependances** : STORY-P7-801.

---

## Epic P7-9 : Accessibilité WCAG 2.1 AA complète | SHOULD HAVE

**Motivation** : KoproGo est une plateforme SaaS B2B à destination de syndics belges, qui incluent des copropriétaires âgés et/ou malvoyants. La conformité WCAG 2.1 AA n'est pas optionnelle : c'est à la fois un impératif éthique (accessibilité universelle), légal (loi belge du 19/07/2018 transposant la directive européenne 2016/2102) et commercial (argument de vente pour les copropriétés avec mandataires publics).

### Story P7-9.1 : Focus trap sur toutes les modales

- **ID** : STORY-P7-901 | **Type** : A11y | **Taille** : M
- **Bounded Context DDD** : Frontend / Accessibility
- **Principes SOLID** : DRY (un helper `useFocusTrap` unique pour 100 % des modales)
- **Invariants** : INV-focus-trap "Tab depuis le dernier élément focusable d'une modale retourne au premier. Shift+Tab depuis le premier retourne au dernier. Escape ferme la modale".
- **User Story** : En tant qu'utilisateur clavier-only (handicap moteur, préférence ergonomique), je veux que mon focus reste piégé dans la modale active pour ne pas perdre ma saisie en tabulant derrière.
- **Scénarios BDD** (Playwright) :

```gherkin
Scenario: Tab en boucle dans une modale ouverte
  Given TicketCreateModal est ouvert
  And le focus est sur le dernier bouton "Créer un ticket"
  When l'utilisateur presse Tab
  Then le focus revient sur le premier input (titre)
  And ne va PAS sur un élément derrière la modale (backdrop, navigation)

Scenario: Escape ferme la modale et restaure le focus précédent
  Given TicketCreateModal est ouvert
  And le trigger "+ Créer un ticket" avait le focus avant l'ouverture
  When l'utilisateur presse Escape
  Then la modale est fermée
  And le focus retourne sur le bouton "+ Créer un ticket"
```

- **Taches techniques TDD** :
  1. [ ] RED : Playwright test `frontend/tests/e2e/a11y/focus-trap.spec.ts` avec 5 scénarios sur 5 modales différentes.
  2. [ ] GREEN : créer `frontend/src/lib/actions/focusTrap.ts` — une Svelte action :
     ```ts
     export function focusTrap(node: HTMLElement) {
       const focusable = () => node.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
       const handleKeydown = (e: KeyboardEvent) => { /* tab logic */ };
       const previousFocus = document.activeElement as HTMLElement;
       focusable()[0]?.focus();
       node.addEventListener('keydown', handleKeydown);
       return {
         destroy() { node.removeEventListener('keydown', handleKeydown); previousFocus?.focus(); }
       };
     }
     ```
  3. [ ] Appliquer `use:focusTrap` sur toutes les modales : `TicketCreateModal`, `TicketAssignModal`, `MeetingCreateModal`, `InvoiceForm`, `PollCreateForm`, `NoticeCreateModal`, `ExchangeRequestModal`, et les ~15 autres.
  4. [ ] `Modal.svelte` (composant partagé) : appliquer focusTrap par défaut.
  5. [ ] Commit : `a11y(modals): add focus trap to all dialogs (WCAG 2.4.3)`.
- **Dependances** : Epic P7-6 (runes), pour éviter de migrer deux fois le même fichier.
- **Fichiers** : `frontend/src/lib/actions/focusTrap.ts` (nouveau), ~20 composants modal, `frontend/src/components/ui/Modal.svelte`.

### Story P7-9.2 : ARIA labels sur tous les boutons icône

- **ID** : STORY-P7-902 | **Type** : A11y | **Taille** : M
- **Invariants** : INV-aria "Tout `<button>` qui ne contient que du texte non verbal (emoji, icône SVG) doit avoir un attribut `aria-label` explicite".
- **User Story** : En tant qu'utilisateur de lecteur d'écran (NVDA, JAWS, VoiceOver), je veux que chaque bouton icône m'annonce sa fonction au lieu de dire "bouton" ou "cloche".
- **Taches** :
  1. [ ] Script d'audit `scripts/a11y-icon-buttons.js` qui parse tous les `.svelte` et liste les `<button>` sans `aria-label`.
  2. [ ] Corriger manuellement (ex. `<button aria-label={$_('notifications.openBell')}><BellIcon /></button>`).
  3. [ ] Ajouter clés i18n correspondantes dans les 4 locales.
  4. [ ] Commit : `a11y(buttons): add aria-label to all icon-only buttons (WCAG 4.1.2)`.
- **Dependances** : Epic P7-6.

### Story P7-9.3 : Audit navigation clavier complet

- **ID** : STORY-P7-903 | **Type** : A11y | **Taille** : S
- **Invariants** : INV-keyboard "Tous les parcours P1-P10 sont exécutables sans souris".
- **Taches** :
  1. [ ] Playwright test qui refait les 10 parcours en utilisant `page.keyboard.press('Tab')` uniquement.
  2. [ ] Corriger les composants qui capturent incorrectement le focus (ex. `<div on:click>` au lieu de `<button>`).
  3. [ ] Commit : `a11y(nav): ensure all user journeys work keyboard-only (WCAG 2.1.1)`.

### Story P7-9.4 : Audit contraste couleurs via axe-core CI

- **ID** : STORY-P7-904 | **Type** : A11y | **Taille** : S
- **Invariants** : INV-contrast "Tout texte respecte un ratio de contraste WCAG AA (4.5:1 normal, 3:1 large)".
- **Taches** :
  1. [ ] Intégrer `@axe-core/playwright` dans la suite E2E.
  2. [ ] Dans `frontend/tests/e2e/a11y/axe.spec.ts`, lancer axe sur chaque page principale et exporter un rapport JSON.
  3. [ ] Corriger les violations sur les gris clairs (`text-gray-400` sur `bg-white`).
  4. [ ] Étape CI qui échoue si axe trouve une violation `impact === serious` ou plus.
  5. [ ] Commit : `a11y(ci): integrate axe-core in e2e tests`.
- **Fichiers** : `.github/workflows/ci.yml`, `frontend/tests/e2e/a11y/axe.spec.ts` (nouveau), `frontend/package.json` (+ `@axe-core/playwright`).

### Story P7-9.5 : Tests manuels avec lecteur d'écran

- **ID** : STORY-P7-905 | **Type** : A11y QA | **Taille** : M
- **Taches** :
  1. [ ] Exécuter les 10 parcours avec NVDA + Chrome sur Windows.
  2. [ ] Documenter les correctifs nécessaires dans `docs/A11Y_SCREEN_READER_AUDIT.md`.
  3. [ ] Corriger les issues (annonces redondantes, sections sans landmark, tables sans caption).
  4. [ ] Commit : `a11y(sr): screen reader audit + fixes`.

---

## Epic P7-10 : Contractor as first-class domain role | MUST HAVE

**Motivation** : STORY-P7-2 a proposé une "Option A" (réutiliser `BoardMember` comme cible d'assignation) comme compromis pour aller vite. Mais d'un point de vue domain-driven design, **un contractor n'est pas un board member**. Ce sont deux concepts distincts dans la réalité métier belge :
- **BoardMember** : copropriétaire élu au conseil de copropriété (Art. 577-8 CC)
- **Contractor** : prestataire externe engagé pour réaliser des travaux ou un service (plombier, électricien, menuisier, etc.) — peut ne pas être copropriétaire.

Fusionner les deux viole DDD / SRP et empêchera plus tard d'ajouter les fonctions contractor-spécifiques (portfolio, avis clients, disponibilités, SIREN/TVA, assurances professionnelles).

Cet epic remplace l'Option A temporaire par une Option B propre.

### Story P7-10.1 : Ajouter `Contractor` au domain UserRole

- **ID** : STORY-P7-1001 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Identity & Access / Maintenance
- **Entité(s) DDD** : `User`, nouvelle `Contractor` (profil étendu)
- **Principes SOLID** : SRP (Contractor est un rôle distinct avec ses invariants), OCP (ajout sans modifier les autres rôles)
- **Invariants** : INV-contractor "Un User avec `role = Contractor` doit avoir un `ContractorProfile` lié (profession, SIREN, assurance)".
- **FR PRD** : FR-009 (gestion tickets), FR-013 (nouveau — gestion prestataires)
- **User Story** : En tant que syndic, je veux enregistrer mon plombier "Marc Dubois" comme Contractor avec sa profession, son numéro SIREN/TVA et son assurance pour pouvoir lui assigner des tickets et valider légalement l'intervention.
- **Scénarios BDD** :

```gherkin
Scenario: Créer un contractor avec profil complet
  Given François est syndic de "Syndic Leroy"
  When François crée un Contractor "Marc Dubois" avec
    | email            | marc@plomberie-dubois.be |
    | profession       | Plombier                 |
    | siren_or_vat     | BE0123456789             |
    | insurance_number | AXA-12345                |
  Then un User avec role=Contractor est créé
  And un ContractorProfile lié est créé avec les champs ci-dessus
  And le contractor apparaît dans GET /contractors?organization_id=...

Scenario: Un contractor peut prendre en charge un ticket
  Given un Contractor Marc existe
  And un ticket "Fuite" existe en statut Open
  When le syndic assigne le ticket à Marc
  Then le ticket passe en Assigned
  And assigned_to contient l'ID de Marc (qui est un User avec role=Contractor)

Scenario: Un contractor ne peut PAS voter aux AG
  Given un Contractor Marc n'est pas copropriétaire
  When Marc tente de voter sur une résolution
  Then 403 Forbidden est retourné
  And le message est "Only co-owners can vote"
```

- **Taches techniques TDD** :
  1. [ ] RED : test unitaire `backend/src/domain/entities/user.rs` vérifiant que `UserRole::Contractor` existe.
  2. [ ] GREEN : étendre l'enum `UserRole` dans `backend/src/domain/entities/user.rs` avec variant `Contractor`.
  3. [ ] Migration SQL `[timestamp]_add_contractor_role.sql` : `ALTER TYPE user_role ADD VALUE 'contractor';`
  4. [ ] Créer domain entity `backend/src/domain/entities/contractor.rs` :
     ```rust
     pub struct ContractorProfile {
         pub id: Uuid,
         pub user_id: Uuid,
         pub profession: String,
         pub siren_or_vat: String, // Belgian SIREN/VAT
         pub insurance_number: Option<String>,
         pub insurance_expires_at: Option<DateTime<Utc>>,
         pub hourly_rate: Option<f64>,
         pub created_at: DateTime<Utc>,
     }
     ```
  5. [ ] Validation : SIREN/TVA belge via regex `^BE[0-9]{10}$`.
  6. [ ] Migration `[timestamp]_create_contractor_profiles.sql`.
  7. [ ] Port `ContractorRepository` avec 5 méthodes (CRUD + `find_by_organization`).
  8. [ ] Use case `ContractorUseCases` avec règles métier (création + invariant profil obligatoire).
  9. [ ] BDD feature `backend/tests/features/contractors.feature`.
- **Dependances** : aucune.
- **Endpoints** : `POST /contractors`, `GET /contractors`, `GET /contractors/:id`, `PUT /contractors/:id`, `DELETE /contractors/:id`.
- **Fichiers** : `backend/src/domain/entities/{user,contractor}.rs`, `backend/src/application/ports/contractor_repository.rs`, `backend/src/application/use_cases/contractor_use_cases.rs`, `backend/src/infrastructure/database/repositories/contractor_repository_impl.rs`, `backend/src/infrastructure/web/handlers/contractor_handlers.rs`, 2 migrations SQL, `backend/tests/features/contractors.feature`.

### Story P7-10.2 : UI Gestion des contractors (CRUD)

- **ID** : STORY-P7-1002 | **Type** : Feature | **Taille** : M
- **Bounded Context DDD** : Frontend / Contractor Management
- **User Story** : En tant que syndic, je veux gérer mon carnet d'adresses de contractors depuis une page dédiée avec leur profession, coordonnées et assurance.
- **Taches** :
  1. [ ] Créer `frontend/src/pages/contractors.astro` avec liste paginée.
  2. [ ] Créer `frontend/src/components/contractors/{ContractorList,ContractorCreateModal,ContractorDetail,ContractorEditForm}.svelte`.
  3. [ ] Ajouter wrapper `frontend/src/lib/api/contractors.ts`.
  4. [ ] Ajouter entrée sidebar "👷 Prestataires" dans `Navigation.svelte`.
  5. [ ] i18n FR/NL/DE/EN.
  6. [ ] Playwright smoke test.
- **Dependances** : STORY-P7-1001.

### Story P7-10.3 : TicketAssignModal reprend le concept Contractor

- **ID** : STORY-P7-1003 | **Type** : Feature | **Taille** : S
- **Remplace/étend** : STORY-P7-2 (Option A → Option B).
- **User Story** : En tant que syndic, je veux que le dropdown d'assignation de ticket affiche les **Contractors** (et éventuellement les BoardMembers/Syndics si besoin) avec leur profession à côté du nom.
- **Taches** :
  1. [ ] Modifier endpoint `GET /tickets/assignable-users` pour inclure les users `role IN (Syndic, BoardMember, Contractor)` avec leur profession si Contractor.
  2. [ ] Mettre à jour `TicketAssignModal` (qui sera déjà migré en runes par P7-602) : afficher "Marc Dubois — Plombier" au lieu de "Marc Dubois (board_member)".
  3. [ ] Adapter le seed STORY-P7-203 pour créer les contractors au lieu de board_members.
- **Dependances** : STORY-P7-202, STORY-P7-1001.

### Story P7-10.4 : Ratings + portfolio des contractors

- **ID** : STORY-P7-1004 | **Type** : Feature | **Taille** : L
- **User Story** : En tant que syndic, je veux voir les avis moyens d'un contractor et son historique d'interventions avant de lui assigner un nouveau ticket.
- **Taches** :
  1. [ ] Entité `ContractorRating` (1-5 étoiles + commentaire + liaison ticket).
  2. [ ] Migration + repository + use case + handler.
  3. [ ] UI : page détail contractor affiche historique + moyenne.
  4. [ ] Après la résolution d'un ticket, proposer au syndic de noter le contractor.
- **Dependances** : STORY-P7-1001, STORY-P7-1002.

### Story P7-10.5 : Alerte assurance contractor expirée

- **ID** : STORY-P7-1005 | **Type** : Feature | **Taille** : S
- **Invariants** : INV-insurance "Un contractor ne peut pas être assigné si son assurance pro est expirée".
- **User Story** : En tant que syndic, je veux être alerté si j'essaie d'assigner un ticket à un contractor dont l'assurance est expirée.
- **Taches** :
  1. [ ] Validation dans `TicketUseCases::assign_ticket` : vérifier `contractor_profile.insurance_expires_at >= today`.
  2. [ ] Retour 422 avec message explicite si expirée.
  3. [ ] Dashboard widget "Contractors avec assurance expirée".
- **Dependances** : STORY-P7-1001.

---

## Synthèse traçabilité Epic → FR PRD → Bug audit

| Epic | Story | Taille | FR PRD | Bug audit v7 | Dépendances | Statut |
|------|-------|--------|--------|--------------|-------------|--------|
| **P7-1 Type-safety foundation** | | | | | | |
| P7-1 | STORY-P7-101 openapi-export bin | S | transverse | — (qualité) | — | PENDING |
| P7-1 | STORY-P7-102 annotations DTO critiques | M | FR-004,006,008,012 | — | P7-101 | PENDING |
| P7-1 | STORY-P7-103 frontend re-exports (3) | S | transverse | prévention v4-v7 | P7-101, P7-102 | PENDING |
| P7-1 | STORY-P7-104 CI guard | S | transverse | régression | P7-101 | PENDING |
| **P7-2 Ticket assignment UX** | | | | | | |
| P7-2 | STORY-P7-201 GET assignable-users | S | FR-009 | B14 | P7-101, P7-102 | PENDING |
| P7-2 | STORY-P7-202 dropdown modal | S | FR-009 | B14 | P7-201 | PENDING |
| P7-2 | STORY-P7-203 seed board members | XS | transverse | B14 | — | PENDING (sera remplacé par P7-10) |
| **P7-3 Meeting governance** | | | | | | |
| P7-3 | STORY-P7-301 canManage gating | S | FR-004, FR-011 | B18 | — | PENDING |
| P7-3 | STORY-P7-302 QuorumPanel | M | FR-004 (Art. 3.87 §5) | B4 | P7-301, P7-102 | PENDING |
| **P7-4 Error UX hardening** | | | | | | |
| P7-4 | STORY-P7-401 error i18n map | S | FR-011 | B17 | — | PENDING |
| P7-4 | STORY-P7-402 toast dedup + auto-close | S | transverse | B21 | — | PENDING |
| P7-4 | STORY-P7-403 Back to Tickets i18n | XS | FR-011 | B19 | — | PENDING |
| **P7-5 Testing hygiene** | | | | | | |
| P7-5 | STORY-P7-501 doc Svelte 5 | XS | — | B16 | — | PENDING |
| **P7-6 Svelte 5 runes migration** | | | | | | |
| P7-6 | STORY-P7-601 pilote TicketCreateModal | M | transverse | prévention B3-v7 | P7-501 | PENDING |
| P7-6 | STORY-P7-602 tickets (6) | M | transverse | — | P7-601 | PENDING |
| P7-6 | STORY-P7-603 meetings (7) | M | FR-004 | — | P7-601, P7-302 | PENDING |
| P7-6 | STORY-P7-604 resolutions (4) | S | FR-008 | — | P7-601 | PENDING |
| P7-6 | STORY-P7-605 expenses (5) | M | FR-006 | — | P7-601 | PENDING |
| P7-6 | STORY-P7-606 community (15) | L | FR-012 | — | P7-601 | PENDING |
| P7-6 | STORY-P7-607 admin+dashboards (20) | L | FR-011 | — | P7-601 | PENDING |
| P7-6 | STORY-P7-608 restants (130+) | XL | transverse | — | P7-607 | PENDING |
| P7-6 | STORY-P7-609 cleanup legacy | S | transverse | — | P7-608 | PENDING |
| **P7-7 utoipa 100 %** | | | | | | |
| P7-7 | STORY-P7-701 DTO 41 restants | L | transverse | — | P7-101 | PENDING |
| P7-7 | STORY-P7-702 entités 54 | L | transverse | — | P7-701 | PENDING |
| P7-7 | STORY-P7-703 handlers 421 | XL | transverse | — | P7-702 | PENDING |
| P7-7 | STORY-P7-704 wrappers 22 | L | transverse | — | P7-701, P7-702 | PENDING |
| P7-7 | STORY-P7-705 CI guard enums | S | transverse | régression | P7-704 | PENDING |
| **P7-8 Seed determinism** | | | | | | |
| P7-8 | STORY-P7-801 seed idempotent | S | transverse | B20 | — | PENDING |
| P7-8 | STORY-P7-802 migration cleanup | S | transverse | B20 | — | PENDING |
| P7-8 | STORY-P7-803 make seed-reset | XS | transverse | B20 | P7-801 | PENDING |
| **P7-9 WCAG 2.1 AA** | | | | | | |
| P7-9 | STORY-P7-901 focus trap modales | M | FR-a11y (nouveau) | — | P7-6 complet | PENDING |
| P7-9 | STORY-P7-902 aria-label boutons | M | FR-a11y | — | P7-6 complet | PENDING |
| P7-9 | STORY-P7-903 audit clavier | S | FR-a11y | — | P7-902 | PENDING |
| P7-9 | STORY-P7-904 axe-core CI | S | FR-a11y | — | — | PENDING |
| P7-9 | STORY-P7-905 screen reader QA | M | FR-a11y | — | P7-901-904 | PENDING |
| **P7-10 Contractor as domain role** | | | | | | |
| P7-10 | STORY-P7-1001 domain + migration | M | FR-013 (nouveau) | — | — | PENDING |
| P7-10 | STORY-P7-1002 UI CRUD | M | FR-013 | — | P7-1001 | PENDING |
| P7-10 | STORY-P7-1003 ticket assignment refonte | S | FR-009, FR-013 | B14 | P7-202, P7-1001 | PENDING |
| P7-10 | STORY-P7-1004 ratings + portfolio | L | FR-013 | — | P7-1001, P7-1002 | PENDING |
| P7-10 | STORY-P7-1005 alerte assurance | S | FR-013 | — | P7-1001 | PENDING |

**Total effort estimé** (T-shirt sizing) :
- XS (≤ 1h) : 5 stories → 5h
- S (1-3h) : 17 stories → 34-51h
- M (3-6h) : 12 stories → 36-72h
- L (6-12h) : 7 stories → 42-84h
- XL (12-24h) : 2 stories → 24-48h

**Fourchette totale** : **141-260h** (18-32 jours ETP à temps plein, réalistement ~6 semaines pour un dev seul avec contexte switching).

---

## Ordre d'exécution recommandé (programme post-audit complet)

Découpage en **6 sprints** (1 semaine chacun), avec priorité ordonnancée par criticité métier et dépendances techniques.

### Sprint 1 — Fondation type-safety + ticket UX (semaine 1)

```
Jour 1-2 : Type-gen pipeline
├─ STORY-P7-101  openapi-export bin                     [S]
├─ STORY-P7-102  annotations DTO critiques              [M]
├─ STORY-P7-104  CI guard                               [S]
└─ STORY-P7-103  re-exports 3 wrappers                  [S]

Jour 3   : Seed cleanup (débloque tests)
├─ STORY-P7-802  migration cleanup pollution            [S]
├─ STORY-P7-801  seed idempotent                        [S]
└─ STORY-P7-803  make seed-reset                        [XS]

Jour 4   : Meeting governance (B4 + B18)
├─ STORY-P7-301  canManage gating                       [S]
└─ STORY-P7-302  QuorumPanel                            [M]

Jour 5   : UX polish + docs (bugs cosmétiques)
├─ STORY-P7-402  toast dedup + auto-close               [S]
├─ STORY-P7-401  error i18n map                         [S]
├─ STORY-P7-403  Back to Tickets i18n                   [XS]
└─ STORY-P7-501  doc Svelte 5                           [XS]
```

Fin Sprint 1 : **prompt cowork v8** — valider Epics P7-1, P7-3, P7-4, P7-5, P7-8 + acquis de la session actuelle.

### Sprint 2 — Contractor domain + ticket assignment (semaine 2)

```
Jour 1-2 : Contractor domain (base)
├─ STORY-P7-1001 domain UserRole::Contractor + migration [M]
└─ STORY-P7-203  seed (transformé en contractors)       [XS]

Jour 3-4 : Contractor UI
├─ STORY-P7-1002 CRUD pages                             [M]
└─ STORY-P7-1005 alerte assurance                       [S]

Jour 5   : Ticket assignment avec contractors
├─ STORY-P7-201  GET assignable-users (étendu)          [S]
├─ STORY-P7-202  dropdown modal                         [S]
└─ STORY-P7-1003 ticket assign refonte contractor       [S]
```

Fin Sprint 2 : **prompt cowork v9** — valider P7-2 + P7-10 partiel.

### Sprint 3 — Svelte 5 runes migration (vague 1) (semaine 3)

```
Jour 1   : Pilote + doc
├─ STORY-P7-601  pilote TicketCreateModal               [M]

Jour 2-5 : Migration dirigée par domaine
├─ STORY-P7-602  tickets (6 composants)                 [M]
├─ STORY-P7-603  meetings (7 composants)                [M]
├─ STORY-P7-604  resolutions (4 composants)             [S]
└─ STORY-P7-605  expenses (5 composants)                [M]
```

Fin Sprint 3 : **prompt cowork v10** — aucune régression sur les 4 modules migrés.

### Sprint 4 — Svelte 5 runes migration (vague 2) + Contractor advanced (semaine 4)

```
Jour 1-3 : Runes vague 2
├─ STORY-P7-606  community polls/notices/SEL (15)       [L]
└─ STORY-P7-607  admin + dashboards + nav (20)          [L]

Jour 4-5 : Contractor ratings + portfolio
└─ STORY-P7-1004 ratings + portfolio                    [L]
```

### Sprint 5 — utoipa 100 % + runes vague 3 (semaine 5)

```
Jour 1-2 : utoipa extension
├─ STORY-P7-701  DTO 41 restants                        [L]
└─ STORY-P7-702  entités 54                             [L]

Jour 3-5 : Runes vague 3 (script semi-auto)
├─ STORY-P7-608  restants 130+ composants               [XL]
└─ STORY-P7-609  cleanup legacy                         [S]
```

### Sprint 6 — utoipa finale + accessibilité (semaine 6)

```
Jour 1-2 : utoipa handlers + garde
├─ STORY-P7-703  handlers 421 annotations               [XL]
├─ STORY-P7-704  wrappers 22 migrés                     [L]
└─ STORY-P7-705  CI guard enums                         [S]

Jour 3-5 : WCAG 2.1 AA
├─ STORY-P7-901  focus trap modales                     [M]
├─ STORY-P7-902  aria-label boutons                     [M]
├─ STORY-P7-903  audit clavier                          [S]
├─ STORY-P7-904  axe-core CI                            [S]
└─ STORY-P7-905  screen reader QA                       [M]
```

Fin Sprint 6 : **audit cowork v11 final** (re-parcours 10 parcours complet) + merge `feature/dev` → `main` + tag `v0.1.0`.

---

## Invariants Maury respectés

- ✅ **DDD** : chaque story nomme son Bounded Context et ses entités. Épic P7-10 rejette explicitement la fusion Contractor/BoardMember par SRP.
- ✅ **BDD** : chaque story fonctionnelle contient des scénarios Gherkin exécutables et des tests Playwright cibles.
- ✅ **TDD** : chaque story liste les tâches RED → GREEN → REFACTOR avec vérification `cargo test` / `npx svelte-check`.
- ✅ **SOLID** : chaque story cite les principes qui la gouvernent (SRP / OCP / LSP / ISP / DIP).
- ✅ **Traçabilité** : chaque story trace vers un FR PRD, un invariant domaine ou un bug audit identifié.
- ✅ **Hexagonal** : les stories backend séparent Domain / Application / Infrastructure ; les stories frontend séparent composants / stores / services.
- ✅ **DRY** : les enums deviennent 100 % générés en Sprint 5, pas 2 sources de vérité.
- ✅ **Completude Maury** : **aucun "hors scope"** — le plan couvre tout ce que l'audit v7 a identifié, y compris les recommandations structurelles de la conclusion. La méthode Maury fait les choses à fond ou ne les fait pas.

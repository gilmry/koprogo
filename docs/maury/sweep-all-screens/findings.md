---
feature: sweep-all-screens
status: EN COURS
date: 2026-08-09
authors: [Claude Sonnet 5 (drafting)]
parent_brief: brief.md
---

# Findings — sweep-all-screens

Journal continu des trouvailles pendant l'audit systématique, par rôle. Chaque entrée : constat, preuve, action (fixé / documenté / hors-scope).

## Admin

### ✅ FIXÉ — `ExpenseDetail.svelte` : `getPaymentStatusBadge`/`getPaymentMethodLabel` en PascalCase

- **Constat** : comparaient `payment.status`/`payment.method` (type `TransactionStatus`/`PaymentMethodType`, backend `#[serde(rename_all = "snake_case")]`, `payment.rs:7-30`) à des clés PascalCase (`'Succeeded'`, `'SepaDebit'`...). La section "Paiements" de la fiche dépense affichait du texte brut non traduit au lieu du badge.
- **Trouvé par** : grep systématique `#[serde(rename_all = "snake_case")]` sur toutes les entités backend + cross-check frontend, après avoir découvert le même pattern 2× (Story 1 `payment_status`).
- **Fix** : clés `Record` passées en snake_case (`'succeeded'`, `'sepa_debit'`, etc.), même correction que Story 1.
- **Vérifié** : `npm run build` 0 erreur.

### 📋 DOCUMENTÉ (pas fixé) — `InspectionStatus` frontend diverge du backend

- **Constat** : `frontend/src/lib/api/inspections.ts:32-37` définit `InspectionStatus = {Pending, Completed, Failed, PassedWithRemarks}` (valeurs `pending/completed/failed/passed_with_remarks`), alors que le backend `InspectionStatus` (`technical_inspection.rs:72-79`) a `{Scheduled, InProgress, Completed, Failed, Overdue, Cancelled}` (`scheduled/in_progress/completed/failed/overdue/cancelled`) — vocabulaire complètement différent, pas juste une casse.
- **Déjà connu** : commentaire explicite dans le code, `STORY-P7-704`, "Enums kept hand-written because backend schemas diverge". Pas une régression fraîche.
- **Impact réel non mesuré** : pas de endpoint `GET /inspections` plat pour vérifier rapidement (scope building/organization uniquement) ; nécessiterait un fixture building pour tester en direct.
- **Action** : hors-scope de ce sweep (réconciliation = son propre chantier, cf. brief §4). Signalé pour arbitrage @gilmry : soit aligner le frontend sur le vrai contrat backend, soit clarifier pourquoi le hand-written diverge (peut-être un DTO de réponse différent de l'entité domaine, à vérifier).

## Syndic

Visite des 36 écrans syndic (console + requêtes réseau) : **33/36 propres**. 2 écarts investigués :

### ✅ Non-bug (investigué, confirmé bénin) — `getMyOwner()` 404 sur `/exchanges` et `/notices`

- `exchanges.astro`/`notices.astro` appellent `getMyOwner()` (`GET /owners/me`) pour résoudre l'ownerId d'un syndic qui n'a pas de fiche Owner — 404 normal, catché silencieusement (`if (owner) {...}`, pas de toast). Le "console.error" observé est juste le logging réseau automatique de Chrome, pas une erreur applicative. Corrige/précise une trouvaille de l'exploration passive du début de session ("toast trompeur owners/me") — après investigation, il n'y a **pas** de toast, juste un log console silencieux.

### ✅ FIXÉ — Toast "Accès refusé" superflu sur `/syndic/role-delegations`

- **Constat** : `RoleDelegationsPage.svelte:82` appelle `GET /organizations?per_page=1000` (superadmin-only) pour enrichir l'affichage des noms d'organisation dans les délégations — 403 pour un syndic. Le `.catch()` local empêche le crash mais **`api.get()` déclenche un toast automatique avant** que le catch ne s'exécute (toute erreur HTTP sauf `silent: true`, `lib/api.ts:141-143`).
- **Fix** : ajout de `{ silent: true }`, même pattern déjà établi dans `tryGetOrganizationName()` (`lib/api/organizations.ts:50`) pour exactement ce cas (appel optionnel d'enrichissement, 403 attendu pour non-superadmin).
- **Vérifié** : `npm run build` 0 erreur. Les autres appels à `/organizations?per_page=1000` du repo sont tous dans des pages superadmin-only (`OrganizationList`, `AcpList`, `UserForm` via `UserListAdmin`, `RoleAssignmentForm`) — jamais atteints par un rôle non-superadmin, pas de 403 possible en pratique, pas besoin du même fix.

### Grep exhaustifs confirmant l'absence d'autres instances des 2 bugs connus (2026-08-09)

- `<Composant ... on:click=/on:submit=/on:change=/on:input=/on:keydown=` sur toute balise capitalisée : **0 résultat**.
- Comparaisons `.status === 'PascalCase'` croisées avec la liste des enums `#[serde(rename_all = "snake_case")]` backend : Resolution/Poll utilisent un pattern sûr (const object généré, valeurs déjà snake_case) ; Budget/AgSession/PaymentReminder/Ticket n'ont **pas** de `rename_all` donc PascalCase est correct côté backend — pas des bugs. Seul `payment.rs` (Transaction/PaymentMethodType) était cassé, cf. ci-dessus.

### Pivot méthodologique : du click-sweep aveugle aux parcours de création remplis jusqu'au bout

Le click-sweep exhaustif (chaque bouton, cliqué individuellement, détection par diff DOM) a produit un **incident** (514 suppressions réelles de `role_assignments` via un bug de dédoublonnage sur un libellé contenant un UUID — contenu localement, aucune donnée seed/prod touchée, cf. `2026-08-09-...` agent-activity si présent) puis, une fois corrigé, une cascade de faux négatifs (sélecteur de langue, filtres d'onglets, boutons désactivés par la conformité, 3 patterns de modale différents dans le code legacy vs Svelte). Deux batches de click-sweep « sécurisé » : **0 bug réel trouvé**, temps d'investigation élevé (faux positifs à vérifier un par un dans le code).

Pivot vers des **parcours de création remplis et soumis jusqu'au bout** (`SyndicCreationJourneys.spec.ts`, ex-`zzz-journeys/journeys-batch1.spec.ts`) : 4 parcours (owner-contributions, budgets, etats-dates, syndic/board-members). Résultat : **5 bugs réels trouvés en un seul batch**, contre 0 pour deux batches de click-sweep — confirme que remplir et soumettre un formulaire jusqu'à son effet de bord réel (POST + 2xx) a un bien meilleur ratio signal/bruit que le clic aveugle sur une UI dont la plupart des interactions n'ont pas d'effet observable sans données de test correctement remplies.

### ✅ FIXÉ — `OwnerContributionForm.svelte` : réponse paginée non dépaquetée

- **Constat** : `loadData()` assignait directement `ownersResp`/`unitsResp` (forme `{data: [...], pagination: {...}}`) à `owners`/`units` au lieu de `.data` — les `<select>` owner/unit restaient vides, aucune contribution ne pouvait jamais être créée depuis l'UI.
- **Fix** : dépaquetage explicite via `withErrorHandling`, `owners = result.ownersData` / `units = result.unitsData`.

### ✅ FIXÉ — `owner-contributions.astro` : clé localStorage morte + course async sur `authStore.init()`

- **Constat** : `localStorage.getItem('organizationId')` — cette clé n'a jamais existé depuis la refonte JWT/cookie (WP-FE1), donc `organizationId` était toujours `''` et l'`$effect` de chargement du formulaire ne se déclenchait jamais.
- **Fix** : `await authStore.init(); const auth = get(authStore); const organizationId = auth.user?.organizationId ?? auth.user?.activeRole?.organizationId ?? ''`, pattern déjà établi dans `settings.astro`. Le `await` est nécessaire : `authStore.init()` est asynchrone (silent-refresh via cookie) et un `get(authStore)` synchrone avant résolution lit un store encore vide.
- **Vérifié non-régressif** : `authStore.init()` ne rejette jamais (toutes les erreurs internes — fetch, cache local — sont catchées ; cf. `stores/auth.ts:198-221` et `:296+`), donc le top-level `await` de la page ne peut pas laisser un écran blanc en cas de défaillance réseau/cookie.

### ✅ FIXÉ — `EtatDateCreateForm.svelte` : check de conformité `=== undefined` qui ne se déclenche jamais

- **Constat** : `GET /buildings` (liste paginée) renvoie **toujours** des métriques vides explicites (`is_conformant: false`, jamais `undefined` — choix de perf intentionnel documenté dans `building_use_cases.rs`, `BuildingMetrics::empty()`, pour éviter un JOIN coûteux sur la pagination). Le formulaire vérifiait `selectedBuilding.is_conformant === undefined` pour décider s'il fallait recharger les vraies métriques — ce check ne se déclenchait **jamais**, donc **tout** immeuble apparaissait non-conforme, bloquant la génération d'état daté pour des immeubles réellement conformes. Bug potentiellement bloquant en production pour l'ensemble des syndics.
- **Fix** : recharge systématique de `GET /buildings/{id}` (seul endpoint qui calcule les vraies métriques via `LEFT JOIN units`) après sélection d'un immeuble, sans dépendre d'un `undefined` qui n'arrive jamais.
- **Bug métier légitime rencontré en marge (pas un bug)** : "Unit has no active owners" — un état daté porte sur un lot avec propriétaire actif ; le fixture de test devait lier explicitement le owner au unit via `POST /units/{id}/owners` avant de générer.

### ✅ FIXÉ — Nouvelle organisation jamais seedée en comptes PCMN (backend)

- **Constat** : `POST /organizations` (seul point de création d'organisation en production — vérifié par grep exhaustif des appelants de `organization_use_cases.create` et des `INSERT INTO organizations`) ne seedait jamais le plan comptable belge (PCMN) par défaut. Le seeding n'existait que via `seed_belgian_pcmn_for_all_organizations()`, appelé une fois au **boot** du serveur (`main.rs:131`) — ne rattrape que les organisations déjà existantes à ce moment-là, jamais celles créées après. Toute organisation créée après le dernier redémarrage du backend avait **zéro compte**, donc toute création d'`owner_contribution`/`expense` référençant un `account_code` échouait avec une violation de FK (`fk_account_code`), avec un message d'erreur SQL brut peu explicite renvoyé côté client.
- **Fix** : `POST /organizations` (`organization_handlers.rs`) appelle désormais `account_use_cases.seed_belgian_pcmn(org.id)` juste après la création, en loggant (mais sans faire échouer la création d'org) en cas d'erreur — idempotent par construction (`seed_belgian_pcmn` refuse si des comptes existent déjà).
- **Choix assumé, pas de test Rust dédié** : la couverture de non-régression pour ce fix est le parcours E2E `SyndicCreationJourneys.spec.ts` (owner-contributions, bout en bout, échoue si le seeding casse) plutôt qu'un test unitaire côté backend.

### ✅ FIXÉ — `OwnerContributionForm.svelte` : codes de compte PCMN inventés (`7000`/`7100`)

- **Constat** : bug indépendant du précédent, découvert seulement une fois le seeding d'organisation corrigé (même symptôme — violation de `fk_account_code` — deux causes racines distinctes empilées). Le formulaire fixait `account_code` à `'7000'` (régulier) / `'7100'` (extraordinaire) selon `contribution_type`. Ces codes n'ont **jamais existé** dans le PCMN réellement seedé (`get_belgian_pcmn_seed_data()`, `account_use_cases.rs`) : seuls les comptes feuilles `700001` ("Appels de fonds ordinaires"), `700002` ("...extraordinaires") et `700003` ("Provisions mensuelles") sont `direct_use: true` ; `700`/`70` existent mais ne sont pas utilisables directement en écriture. Donc **toute** contribution, quel que soit le type, était rejetée en 400.
- **Fix** : mapping corrigé sur les 4 `contribution_type` : `regular→700001`, `extraordinary→700002`, `advance→700003`, `adjustment→700001` (pas de compte dédié, fallback sur ordinaire).
- **Vérifié** : `SyndicCreationJourneys.spec.ts` 4/4 ; suite de régression ciblée (`Budgets`, `BoardOfDirectors`, `OwnerDashboard`, `AdminDashBoard.improved` — 37 tests non liés au sweep, tous déjà verts avant) rejouée après les 4 fixes de ce batch, 0 régression.

### ✅ FIXÉ — `TicketCreateModal.svelte` : plantage Svelte au montage, modale de création de ticket jamais rendue

- **Constat** : `formData.unit_id` initialisé à `undefined` (`unit_id: undefined`) puis bindé via `<FormInput ... bind:value={formData.unit_id} />`. `FormInput.svelte` déclare `value = $bindable('')` (fallback non-`undefined`) — passer explicitement `undefined` à un `bind:value` sur un prop bindable avec fallback est une erreur runtime Svelte 5 (`props_invalid_value` : *"Cannot do bind:value={undefined} when value has a fallback value"*), qui fait planter tout l'arbre du composant au montage. Résultat en pratique : cliquer sur « Créer un nouveau ticket » sans `unitId` déjà pré-rempli (le cas normal — création d'un ticket au niveau immeuble, pas depuis la fiche d'un lot) mount silencieusement rien du tout : la modale reste absente du DOM, aucune erreur visible pour l'utilisateur, aucun ticket ne peut jamais être créé depuis cette page. Trouvé uniquement grâce aux logs `pageerror` capturés pendant le parcours de test (invisible sans instrumentation, l'UI ne montre aucun signe d'échec).
- **Fix** : `unit_id: ""` au lieu de `unit_id: undefined` dans l'état initial de `formData` — le payload envoyé au submit fait déjà `formData.unit_id || undefined` (ligne 105), donc le changement est purement cosmétique côté state interne, sans impact sur le contrat API.
- **Grep de vérification** : recherche exhaustive de tout champ `$state` initialisé à `undefined` littéral et bindé via `FormInput`/`FormSelect` (composants avec fallback `$bindable`) dans tout `src/components/` — seul ce cas matchait ; les autres correspondances (`energy-campaigns/*`, `InspectionList.svelte`) bindent des `<input>`/`<select>` HTML natifs, qui ne portent pas cette contrainte Svelte 5 et sont sains.
- **Ajouté** : `SyndicTicketsAndNoticesJourneys.spec.ts` (tickets + notices, bout en bout). Suite de régression ciblée (`Tickets.spec.ts`, `Notices.spec.ts`) rejouée, 12/12 verts.

### Garde permanente ajoutée : `failOnPageErrors` (helpers/pageErrors.ts)

Le bug `TicketCreateModal` ci-dessus était invisible à tout le reste de l'outillage : `npm run build` passait, `cargo clippy` passait, `Tickets.spec.ts` passait (il crée via l'API directement, pas via l'UI), et le snapshot DOM ne montrait rien d'anormal — seule l'écoute temporaire de `page.on("pageerror")` pendant le debug a révélé le crash Svelte 5 (`props_invalid_value`). Ce n'était pas un coup de chance : c'est le seul détecteur pour toute une classe de bugs (crash runtime au montage d'un composant) à laquelle le reste de l'outillage est structurellement aveugle.

Ajouté `tests/e2e/helpers/pageErrors.ts` (`failOnPageErrors(page)`, appelé en `beforeEach`) dans les 2 specs de parcours (`SyndicCreationJourneys.spec.ts`, `SyndicTicketsAndNoticesJourneys.spec.ts`) — **scope volontairement limité à ces specs de parcours**, pas de rétrofit sur les 30+ specs existantes (risque de faux positifs sur du code déjà en prod et non couvert par cette investigation). Vérifié : les 6 tests existants restent verts avec la garde active (aucun faux positif).

### Grep de vérification élargi : bug class "bind:value={undefined} sur bindable avec fallback" — confirmé isolé au seul cas ci-dessus

Le grep initial (`: undefined,` littéral) était plus étroit que la vraie condition de crash (tout champ absent de l'initializer `$state`, pas seulement explicitement `undefined`). Recherche élargie : tous les usages de `bind:value={...}` vers `FormInput`/`FormSelect`/`FormTextarea` (les 3 seuls composants `ui/` avec un fallback `$bindable('')`) dans tout `src/components/` (6 fichiers : `RegisterForm`, `admin/BuildingForm`, `admin/OrganizationForm`, `admin/UserForm`, `payments/PaymentMethodAddModal`, `tickets/TicketCreateModal`), puis vérification de l'initialisation de chaque champ bindé dans son fichier. Seul `TicketCreateModal.unit_id` (déjà fixé) était concerné — `BuildingForm.construction_year` utilise `null` (pas `undefined`, donc sain : la règle Svelte 5 ne s'applique qu'à `undefined` explicite), tous les autres champs sont initialisés à `''`. Bug class confirmée fermée.

### ✅ FIXÉ — `work-reports.astro` / `BuildingDetail.svelte` : `WorkReportList` jamais reçoit `organizationId`

- **Constat** : `WorkReportList.svelte` a `organizationId = ""` par défaut et l'envoie tel quel dans le payload de création (`organization_id: organizationId`). Ni `work-reports.astro` (montage direct) ni `BuildingDetail.svelte` (onglet "Travaux" de la fiche immeuble) ne passaient cette prop — **toute** création de rapport de travaux échouait avec 400 `"Invalid organization_id format"` (chaîne vide n'est pas un UUID valide), depuis les deux points d'entrée possibles.
- **Fix** : `work-reports.astro` résout `organizationId` via `await authStore.init(); get(authStore)` (même pattern que `owner-contributions.astro`). `BuildingDetail.svelte` réutilise l'`acp.organization_id` déjà chargé pour résoudre `organizationName` (nouvelle variable `organizationId`, dégradation silencieuse à `""` pour une ACP auto-gérée sans organisation — cas légitime, pas une régression introduite).
- **Vérifié** : `SyndicWorkReportJourney.spec.ts` (nouveau) + `Buildings.spec.ts` (régression ciblée, 5 tests) — 6/6 verts. `npx astro check` : 0 erreur.

### ✅ FIXÉ — "Demander un devis" (`/quotes`) : incohérence d'architecture backend/frontend, fonctionnalité 100% cassée

- **Constat** : `QuoteList.svelte` (`handleCreate()`) envoie `POST /quotes` avec seulement `{building_id, contractor_id, project_title, project_description, work_category}` — c'est la sémantique attendue d'une **demande** de devis (le syndic sollicite un prix, ne le connaît pas encore). Mais le DTO backend `CreateQuoteDto` (`quote_dto.rs:7-18`) exigeait en plus, **tous non-optionnels** : `amount_excl_vat`, `vat_rate`, `validity_date`, `estimated_duration_days`, `warranty_years` (`estimated_start_date` était le seul champ optionnel). La table `quotes` (migration `20251120150000_create_quotes.sql`) avait ces mêmes colonnes en `NOT NULL` au niveau DB. Résultat : **tout** clic sur "Demander un devis" échouait systématiquement en 400 (`Json deserialize error: missing field 'amount_excl_vat'`), reproduit directement en `curl` sur `POST /quotes`.
- **Preuve que c'était bien une incohérence de conception, pas un oubli frontend** : le backend a un endpoint distinct `POST /quotes/{id}/submit` (`quote_handlers.rs:110`) et le frontend a un DTO `SubmitQuoteDto` séparé (`amount_excl_vat_cents`, `vat_rate`, `validity_date`, `estimated_duration_days`, `warranty_years`) dans `lib/api/quotes.ts:57-63` — le statut `QuoteStatus::Requested` (premier statut du workflow, `quote_status` enum) confirme que le modèle de domaine était bien pensé en 2 phases (1. syndic demande, 2. contractant soumet un prix plus tard). C'était `CreateQuoteDto` qui contredisait ce modèle en exigeant les champs de la phase 2 dès la phase 1. Pire : `submit_quote()` (use case) ne prenait même pas de corps de requête — le `SubmitQuoteDto` du frontend était silencieusement ignoré, aucune donnée de prix n'était jamais persistée à la soumission.
- **Fix** (architecture, pas un patch frontend) : migration `20260810000000_quotes_two_phase_workflow.sql` rend `amount_excl_vat`, `vat_rate`, `amount_incl_vat`, `validity_date`, `estimated_duration_days` nullable en DB + ajoute `work_category`. Domaine (`Quote` entity) : ces 5 champs deviennent `Option<...>`, nouveau type `QuoteSubmission` porte les données de la phase 2, `Quote::submit(pricing: Option<QuoteSubmission>)` applique le prix à la soumission (ou exige qu'il soit déjà présent via l'ancien chemin de compat `set_initial_pricing`). `CreateQuoteDto` : champs prix `#[serde(default)] Option<...>` (la demande sans prix est maintenant le chemin nominal, la création avec prix immédiat reste possible en legacy). `SubmitQuoteDto` (nouveau, unités cents/pourcentage comme `lib/api/quotes.ts`) est maintenant réellement lu par le handler `POST /quotes/{id}/submit` et persisté. `QuoteResponseDto` : champs prix `Option<i64>` (cents)/`Option<Decimal>` (pourcentage) pour rester fidèle à l'état réel du devis. `compare_quotes()` : ignore les devis pas encore soumis dans les calculs min/max/moyenne au lieu de planter.
- **Vérifié** : `cargo test --lib` (1663/1663), `cargo clippy --all-targets --all-features -D warnings` (clean), BDD "Contractor Quotes Management" (13/13 scénarios, 124/124 steps). Vérification live contre la stack Docker (cargo watch, migration appliquée en base réelle) : nouveau test `Quotes.spec.ts` "should request a quote without pricing then submit real pricing (2-phase workflow)" reproduit exactement le payload envoyé par `QuoteList.svelte` en phase 1 (aucun champ prix) puis soumet un `SubmitQuoteDto` réel en phase 2 — 8/8 tests `Quotes.spec.ts` verts.
- **Sévérité** : haute — fonctionnalité "Demander un devis" syndic entièrement non-fonctionnelle en l'état, pour tous les syndics, sans aucun message d'erreur utilisateur clair (juste un toast générique `errorMessage`). Maintenant fonctionnelle de bout en bout.
- **Commit de suivi** (`e1901e7d`) : durcissement post-fix, trouvé en auto-relecture (pas par les tests) — `compare_quotes()` échouait encore si un des devis du lot n'était pas soumis (statut `Requested` normal et atteignable désormais), corrigé par partition scoré/en-attente ; `submit_quote` passait de `Option<web::Json<T>>` à `web::Bytes` + parsing manuel pour distinguer un corps vide d'un JSON malformé (qui doit renvoyer 400, pas être avalé silencieusement comme "pas de prix"). `e2e_quotes.rs` mis à jour en conséquence. Vérifié sans nouvelle régression : `bdd_governance` Quote feature 13/13, `e2e_quotes.rs` échoue uniquement sur le bug préexistant `buildings_acp_id_fkey` (non lié).

### ✅ Propre — Documents (`/documents`) : upload → liste → téléchargement → suppression

- **Parcours testé** (`SyndicDocumentsJourney.spec.ts`, nouveau) : upload réel d'un fichier via l'UI (multipart, `DocumentUploadModal`), vérification de l'apparition dans la liste, téléchargement (200), suppression avec confirmation navigateur (204) puis disparition de la liste. `failOnPageErrors` actif.
- **Résultat** : vert du premier coup, aucun bug applicatif trouvé.
- **Incident d'infra rencontré en marge (pas un bug applicatif)** : le conteneur `koprogo-frontend` était `Exited (1)` avec `"Another astro dev server is already running"`, provoqué par un fichier de lock Astro (`frontend/.astro/dev.json`, bind-monté depuis l'hôte) resté d'un process précédent crashé (`pid 38`, horodaté `2026-08-11`). Astro dev refuse de redémarrer tant que ce lock existe, donc **tout** `docker compose up/restart frontend` échouait silencieusement depuis. Fix : suppression du fichier de lock puis restart — sans ce fix, aucun test E2E frontend (Documents ou autre) n'aurait pu tourner.

### ✅ Propre — Échanges locaux / SEL (`/exchanges/new`) : création d'offre

- **Parcours testé** (`SyndicLocalExchangesJourney.spec.ts`, nouveau) : création réelle d'une offre d'échange via `CreateExchangeForm.svelte`, en tant que propriétaire (le SEL est un parcours Owner, pas Syndic — `provider_id` est résolu côté backend depuis `auth.user_id` via `owner_repo.find_by_user_id`, jamais envoyé par le frontend, ce qui est le design correct).
- **Piège de fixture rencontré (pas un bug produit)** : `GET /buildings` est scopé via `unit_owners` pour le rôle `owner` (Story 1.3 / BUG-WF14-2) — un `Owner` sans lot lié ne voit **aucun** immeuble, donc `BuildingSelector` reste vide et le formulaire ne peut jamais soumettre (aucun POST déclenché, juste un blocage de validation silencieux). Le helper `loginAsSyndicWithLinkedOwner` crée un compte Owner lié à un user mais jamais à un lot — il fallait ajouter explicitement un lot + `POST /units/{id}/owners` dans le test, même pattern que le fix `EtatDateCreateForm` déjà noté plus haut. Un vrai propriétaire SEL possède toujours un lot, donc pas un bug produit.
- **Résultat** : une fois le fixture corrigé, vert du premier coup — aucun bug applicatif trouvé.

### ✅ Propre — Sondages (`/polls/new`) : création de sondage Oui/Non

- **Parcours testé** (`SyndicPollsJourney.spec.ts`, nouveau) : création réelle d'un sondage Oui/Non via `CreatePollForm.svelte`, en tant que syndic.
- **Piège de fixture rencontré (pas un bug produit)** : `create_poll` (`poll_use_cases.rs:79-92`) calcule `total_eligible_voters` côté backend à partir des `unit_owners` actifs de l'immeuble et **rejette** la création si ce total est `0` (`Poll::new`, `domain/entities/poll.rs:92-93`, `"Total eligible voters must be positive"`) — règle métier légitime (on ne peut pas consulter des copropriétaires qui n'existent pas), mais `loginAsSyndicWithBuilding` seede des lots sans jamais les lier à un propriétaire. Il fallait créer un propriétaire + le lier à un lot (`POST /units/{id}/owners`) avant de créer le sondage, même pattern que le fix SEL ci-dessus et `EtatDateCreateForm` plus haut.
- **Observation mineure, pas fixée (hors-scope)** : `/polls/new` ne lit jamais le paramètre `?building=` que `PollList.svelte` ajoute à son lien "créer un sondage" (`href="/polls/new?building={buildingId}"`) — `CreatePollForm` refait son propre `BuildingSelector` qui auto-sélectionne le premier immeuble de la liste, sans tenir compte de l'immeuble déjà sélectionné sur `/polls`. Invisible avec un seul immeuble de test (le cas de tous les fixtures E2E existants), mais un syndic gérant plusieurs immeubles qui clique "créer un sondage" depuis la liste de l'immeuble B pourrait silencieusement créer le sondage sur l'immeuble A (le premier de la liste) sans s'en rendre compte. Même défaut probable sur `/exchanges/new` (`CreateExchangeForm` a le même pattern `BuildingSelector` autonome). Signalé pour arbitrage, pas fixé dans ce sweep (nécessite de faire passer `building_id` en query param jusqu'au composant, plusieurs écrans concernés).
- **Résultat** : une fois le fixture corrigé, vert du premier coup — aucun bug applicatif trouvé sur le parcours de création lui-même.

### 🚨 TROUVÉ, PAS FIXÉ (hors-scope brief §4, arbitrage requis) — Partage d'objets (`/sharing`) : la fonctionnalité est non-fonctionnelle en UI, sur ses deux seuls points d'entrée

Le brief de ce sweep (`brief.md` §1/§3) est explicitement un audit, pas une nouvelle capacité — ce constat est documenté, pas corrigé, conformément au §4 hors-scope. Deux problèmes indépendants, empilés :

**1. Créer un objet partagé est impossible depuis l'UI — le formulaire n'existe simplement pas.**
`sharingApi.createObject()` (`lib/api/sharing.ts:120`) et l'endpoint backend `POST /shared-objects` existent et fonctionnent tous les deux (vérifié : `resolve_owner(user_id, ...)` résout correctement le vrai propriétaire depuis l'auth côté backend, `dto.owner_id` du DTO frontend est un champ mort jamais utilisé — pas de faille d'autorisation). Mais **aucun composant, aucune page, dans tout `frontend/src`, n'appelle jamais `createObject()`** (grep exhaustif de `sharingApi\.` et `SharedObject` sur tout `src/**/*.svelte` + `*.astro`) : `SharedObjectList.svelte` n'a ni bouton ni lien "Ajouter/Partager un objet". `Sharing.spec.ts` (préexistant) ne l'a jamais détecté car ses 4 tests créent tous l'objet via `page.request.post` direct sur l'API, jamais via l'UI — exactement le point mort que ce sweep a été monté pour trouver (cf. section méthodologique plus haut). Résultat pratique : un copropriétaire ne peut **jamais** lister un objet à prêter via le produit, seul un accès API direct le permettrait.

**2. Le seul bouton visible de toute la fonctionnalité (`LoanPanel` → "Request to Borrow") appelle une API qui n'existe pas sur le backend.**
`LoanRequestModal.svelte` appelle `sharingApi.createLoan()` → `POST /loans`. Grep exhaustif de `"/loans"` sur tout `backend/src` : **zéro résultat**. Confirmé en direct : `curl -X POST http://localhost/api/v1/loans` → `404`. Le frontend modélise un cycle de vie `Loan` complet et séparé de l'objet (`createLoan`/`approveLoan`/`startLoan`/`returnLoan`/`cancelLoan`/`rateBorrower`/`rateLender`/`getOverdueLoans`, 12 méthodes dans `sharingApi`), alors que le backend implémente un modèle radicalement plus simple sans ressource `Loan` séparée : `POST /shared-objects/{id}/borrow` (`BorrowObjectDto { duration_days: Option<i32> }`, emprunteur résolu depuis l'auth) + un endpoint de retour + `GET /shared-objects/my-borrowed` / `/buildings/{id}/shared-objects/borrowed`. Les deux couches ont clairement divergé sans intégration croisée — aucun test E2E n'a jamais cliqué ce bouton (même constat que le point 1 : tous les tests existants passent par l'API `/shared-objects` brute, jamais par `/loans` ni `/borrow`).

**Conséquence** : la fonctionnalité "Partage d'objets / SEL objets" est **entièrement inutilisable en production** — impossible d'y entrer (pas de création) et impossible d'en sortir (le seul CTA restant plante en 404). C'est la trouvaille la plus sévère de tout ce sweep : contrairement au bug devis (mauvais contrat DTO, correctible en un patch), ici les deux couches modélisent des ressources différentes (`Loan` vs `borrow`/`return` direct sur l'objet) — la remise en état exige un choix de conception (reconstruire `/loans` côté backend pour matcher le frontend, ou réécrire `LoanPanel`/`LoanRequestModal`/`sharingApi` pour utiliser le modèle `borrow`/`return` déjà fonctionnel), pas juste un fix de payload. Signalé pour arbitrage @gilmry — pas fixé dans ce sweep.

### ✅ Propre — Compétences (`/skills`) : proposer une compétence

- **Parcours testé** (`SyndicSkillsJourney.spec.ts`, nouveau) : ouverture de la modale `SkillOfferCreateModal` via `#create-offer-btn`, remplissage nom + description, soumission, en tant que propriétaire lié à un lot. Contrairement au Partage d'objets ci-dessus, le nommage diverge entre couches (`Skill`/`CreateSkillDto` côté backend vs `SkillOffer`/`CreateSkillOfferDto` côté frontend) mais **la route et le contrat sont bien alignés** (`POST /skills`, `owner_id` résolu depuis l'auth) — pas un bug, juste une incohérence de nommage cosmétique entre les deux couches.
- **Résultat** : vert du premier coup une fois le même fixture "owner lié à un lot" appliqué (même piège que SEL/Sondages).

### ✅ FIXÉ — `CreateCampaignForm.svelte` : bouton bloqué en spinner "Création…" si validation échoue

- **Constat** : `handleSubmit()` met `loading = true` avant les 4 checks de validation JS (nom, type d'énergie, échéance renseignée, échéance future), mais chacun de ces checks fait un `return` anticipé sans jamais remettre `loading = false`. Reproduit en direct : soumettre le formulaire avec un nom rempli mais **aucun type d'énergie coché** (pas de validation HTML5 native possible sur un groupe de checkboxes) affiche bien l'erreur, mais le bouton reste **définitivement** bloqué en `disabled` + spinner "Création…" — impossible de corriger et resoumettre sans recharger la page entière.
- **Fix** : ajout de `loading = false;` sur les 4 retours anticipés de validation.
- **Trouvaille annexe, fixée au passage** : `}}` en trop dans `{$_("energy.campaign.gdprPoint4")}}` (ligne 216) — affichait un `}` littéral après le texte du 4e point RGPD, à chaque affichage du formulaire. Corrigé.
- **Vérifié** : `SyndicEnergyCampaignsJourney.spec.ts` (nouveau, 2 tests) — création bout en bout (201) + régression validation (bouton réutilisable après erreur), tous deux verts.
- **Note mineure non fixée** : `energy-campaigns/new.astro` calcule `organizationId` via `get(authStore)` **sans** `await authStore.init()` au préalable (même anti-pattern que l'ancien bug `owner-contributions.astro`, déjà fixé plus haut) et le passe en prop à `CreateCampaignForm` — mais ce prop n'est en réalité **jamais utilisé** dans le composant (`organization_id` est correctement résolu côté backend depuis l'auth, `energy_campaign_handlers.rs:20-22`). Code mort, sans impact fonctionnel, non corrigé (hors-scope d'un simple nettoyage).

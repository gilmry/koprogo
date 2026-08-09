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

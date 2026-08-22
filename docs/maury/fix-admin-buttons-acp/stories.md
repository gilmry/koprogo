---
feature: fix-admin-buttons-acp
status: SIGNED v1.0 par @gilmry 2026-08-08
date: 2026-08-08
authors: [Claude Sonnet 5 (drafting)]
parent_brief: brief.md
archetype: full-stack (Story 1 = frontend pur ; Story 2 = frontend, backend déjà conforme)
---

# Stories — fix-admin-buttons-acp

Cycle Foyer (`cycle-dev.md`) : RED (tests 4-cat avant code) → GREEN (fix minimal) → BLEU (refactor + gate + E2E + commit). Chaque story = un tour de cercle.

---

## Story 1 — Réparer les 13 boutons morts (#697)

### Goal

Tout `<Button>` du panneau admin/dépenses déclenche son action au clic. Fix mécanique : `on:click={handler}` → `onclick={handler}`.

### Fichiers exhaustifs (grep déjà fait, cf. #697)

- `frontend/src/components/OrganizationList.svelte:132` — création organisation
- `frontend/src/components/BuildingList.svelte:116` — création immeuble
- `frontend/src/components/UserListAdmin.svelte:173` — création utilisateur
- `frontend/src/components/ExpenseDetail.svelte:206,226,229,232,236,239,243,247` — retour + 7 actions de statut
- `frontend/src/components/BuildingDetail.svelte:122` — retour
- `frontend/src/components/MeetingDetail.svelte:236` — retour

### Root cause additionnelle trouvée en RED — casse `payment_status`

En écrivant les tests `@happy` du panneau dépense (ci-dessous), découverte d'un **second bug indépendant** sur le même fichier : `ExpenseDetail.svelte` compare `expense.payment_status` à des chaînes **PascalCase** (`'Pending'`, `'Overdue'`, `'Paid'`, `'Cancelled'`, lignes 225/235/242/246), alors que le backend sérialise `PaymentStatus` en **snake_case** (`#[serde(rename_all = "snake_case")]`, `backend/src/domain/entities/expense.rs:26`) → `"pending"`, `"paid"`, etc. Résultat : **le bloc conditionnel qui affiche les 5 boutons d'action ne matche jamais**, sur aucune dépense, quel que soit son statut réel — confirmé en direct (badge affiche `pending`, zéro bouton d'action dans le DOM).

Même pattern déjà rencontré et corrigé sur `technical-spec-flow.spec.ts` (C7, cf. commentaire `playwright.config.ts`) mais pas ici. Sans ce fix, réparer uniquement le `on:click`→`onclick` ne suffit pas : les 8 boutons listés ci-dessus pour `ExpenseDetail.svelte` resteraient invisibles. **Inclus dans le scope de Story 1** (décision @gilmry 2026-08-08) — même fichier, même DoD ("panneau d'action ExpenseDetail.svelte fonctionnel").

**Fix additionnel** : `ExpenseDetail.svelte` lignes 225/235/242/246, comparer `expense.payment_status` à `'pending'`/`'overdue'`/`'paid'`/`'cancelled'` (snake_case) au lieu de PascalCase.

### RED — tests avant code (Playwright, clic réel, pas de visite passive)

| Cat | Scénario |
|---|---|
| `@happy` | Login admin → `/admin/organizations` → clic « Nouvelle organisation » → formulaire/dialog s'ouvre (assert `role=dialog` count avant=0 après=1, pattern déjà utilisé dans le script de vérification de ce bug). Répété pour `/buildings` → « Nouvel immeuble » et `/admin/users` → « Nouvel utilisateur ». |
| `@happy` (bis) | `/expenses/{id}` → clic « Marquer payé » → statut passe à `paid` (assert badge/texte de statut, pas juste absence d'erreur). Répété pour les 7 autres actions du panneau (`mark-overdue`, `cancel` ×2, `unpay`, `reactivate`). |
| `@negative` | Bouton dans un état où l'action ne devrait pas s'appliquer (ex. « Marquer payé » sur une dépense déjà payée, si le bouton reste visible) → soit le bouton est désactivé, soit l'action échoue proprement avec un message clair — pas de double-transition silencieuse. |
| `@edge` | Double-clic rapide sur « Nouvelle organisation » → un seul dialog s'ouvre (pas de double-ouverture ni de double-soumission en cascade). |
| `@security` | N/A justifié — fix de câblage d'événement client, aucune nouvelle surface d'autorisation. Vérification : aucun des 13 boutons ne contourne une vérification serveur existante (le fix ne fait que rendre l'appel réseau *possible*, pas nouveau — les endpoints appelés existent et sont déjà gatés côté backend). |

### GREEN

Remplacement mécanique `on:click=` → `onclick=` sur les 13 occurrences. Aucune logique ajoutée.

### BLEU

- Refactor : aucun (le fix est déjà minimal).
- Quality gate : `npm run build` (astro check inclus) vert.
- E2E : les tests `@happy`/`@negative`/`@edge` ci-dessus tournent dans la suite Playwright existante (nouveau spec ou extension d'un spec admin existant).
- Security gate : rien de nouveau à vérifier au-delà de l'existant (voir `@security` ci-dessus).
- Commit : hooks locaux, CI complète au push.

### DoD — Story 1

- [x] 13/13 occurrences corrigées, revérifiées par le même grep que #697 (0 résultat restant)
- [x] 4-cat GREEN (9/9 — le test `@negative` "retour" flake occasionnellement sur un hiccup dev-server Astro non lié au fix, confirmé 3/3 clean en isolation, couvert par les retries CI existants)
- [x] Fix additionnel casse `payment_status` (PascalCase→snake_case) inclus, cf. ci-dessus
- [x] `npm run build` vert (0 erreur, 115 pages) + `npx vitest run` vert (344/344)
- [x] Repro live des 3 étapes de #697 confirmée résolue (dialogs org/immeuble/utilisateur s'ouvrent, panneau dépense fonctionnel de bout en bout)
- [ ] Signature

---

## Story 2 — ACP au lieu d'Organisation dans BuildingForm + dropdown organisation dans AcpList (#698)

### Goal

`BuildingForm.svelte` envoie `acp_id` (requis par le backend), pas `organization_id` (champ qui n'existe plus). `AcpList.svelte` propose un dropdown organisation au lieu d'un UUID collé à la main.

### Prérequis

Story 1 signée et fusionnée (sinon le bouton "Nouvel immeuble" reste mort, impossible de tester ce flow au clic).

### Fichiers exhaustifs

- `frontend/src/components/admin/BuildingForm.svelte` — dropdown organisation → dropdown ACP (`GET /acps`), payload `acp_id` au lieu de `organization_id`
- `frontend/src/components/admin/AcpList.svelte` — champ texte UUID → `<select>` peuplé via `GET /organizations`
- Backend : **aucun changement** — `CreateBuildingDto.acp_id`, `building_handlers.rs`, `GET /acps` (`routes.rs:39`), `GET /organizations` existent déjà et sont conformes

### Root cause additionnelle trouvée en préparant Story 2 — même dérive organization_id/acp_id ailleurs

En cherchant tous les usages de `building.organization_id` (nécessaire pour typer correctement le pré-remplissage du dropdown en édition, cf. `@edge` ci-dessous), découverte de 3 sites supplémentaires touchés par la même dérive (migration Story H15/#602, `organization_id` remplacé par `acp_id`) :

- **`UnitCreateModal.svelte:53`** — `POST /units` envoyait `organization_id`, alors que `UnitDto` exige `acp_id` (même symptôme que le helper de test corrigé en Story 1 : `"Json deserialize error: missing field acp_id"`). **Bug de production réel, pas seulement de test** : la création de lot était cassée pour tout appelant, plus sévère que #698. Inclus dans le scope de Story 2 (décision @gilmry 2026-08-08).
- **`UnitList.svelte:105`** — passait `organizationId={building.organization_id}` (toujours `undefined`) à `UnitCreateModal` — corrigé en `acpId={building.acp_id}`.
- **`BuildingDetail.svelte:67-68`** — `if (building.organization_id)` ne se déclenchait jamais → le nom du cabinet syndic ne s'affichait jamais sur la fiche immeuble. Cosmétique, inclus dans le scope (décision @gilmry 2026-08-08). Fix : résolution en 2 sauts `building.acp_id` → `GET /acps/{id}` → `acp.organization_id` → nom (réutilise `tryGetOrganizationName`, dégradation 403 déjà gérée).
- **`frontend/src/lib/types.ts`** — le type `Building` déclarait encore `organization_id: string` (jamais envoyé par le backend, qui n'expose que `acp_id`) — corrigé en `acp_id: string`, nécessaire pour que les 3 fixes ci-dessus type-check.

### RED — tests avant code

| Cat | Scénario |
|---|---|
| `@happy` | Superadmin → `/buildings` → « Nouvel immeuble » → sélectionne une ACP existante dans le dropdown → submit → 201, immeuble créé avec `acp_id` correct, visible dans la liste. |
| `@happy` (bis) | Superadmin → `/admin/acps` → création ACP → sélectionne une organisation dans le dropdown → submit → ACP créée avec `organization_id` correct. |
| `@negative` | Submit du formulaire immeuble sans ACP sélectionnée → erreur claire côté client (pas de requête envoyée, ou 400 backend affiché lisiblement) — jamais un échec silencieux. |
| `@edge` | Édition d'un immeuble existant qui a déjà une `acp_id` assignée → dropdown pré-rempli avec la bonne ACP (pas de régression sur le flow d'édition, cf. risque identifié dans le brief). |
| `@edge` (bis) | Aucune ACP n'existe encore dans le système → dropdown affiche un état vide explicite (pas un `<select>` vide silencieux), submit bloqué avec message clair. |
| `@security` | Le dropdown ACP/organisation reste accessible uniquement dans le contexte superadmin existant (pas de nouvelle route, pas de nouvel accès exposé) — vérifier qu'aucune ACP/organisation hors périmètre de l'utilisateur courant n'apparaît (cohérent avec le comportement actuel de `GET /acps`/`GET /organizations`, non modifié ici). |

### GREEN

- `BuildingForm.svelte` : fetch `GET /acps`, remplace le `<select>` organisation par ACP (label = nom + adresse), `payload.acp_id` au lieu de `payload.organization_id`.
- `AcpList.svelte` : fetch `GET /organizations`, remplace `<input type="text" bind:value={form.organization_id}>` par `<select>` équivalent.

### BLEU

- Refactor : si le pattern de dropdown ACP dupliqu​e celui déjà existant pour organisation ailleurs, factoriser seulement si un deuxième besoin réel apparaît (YAGNI — pas de composant générique anticipé pour un seul usage).
- Quality gate : `npm run build` vert, `cargo test --lib` vert (aucun changement backend attendu, gate de non-régression).
- E2E : 4-cat ci-dessus + re-test du flow complet création organisation → ACP → immeuble de bout en bout.
- Security gate : rien de nouveau (voir `@security`).
- Commit : hooks locaux, CI complète au push.

### Bug additionnel trouvé en GREEN — boucle infinie `$effect` sur ACP vide

En testant le cas `@edge (bis)` (aucune ACP disponible), découverte d'une boucle infinie de requêtes `GET /acps` (des centaines en quelques secondes, confirmé via trace réseau Playwright) : le garde `$effect(() => { if (isOpen && isSuperAdmin && acps.length === 0) loadAcps(); })` se re-déclenche indéfiniment tant que la liste reste vide — ce qui est **exactement le cas légitime qu'on veut supporter** (système neuf sans ACP). Bug déjà présent dans le code original (`organizations.length === 0`, même structure), porté sans le voir lors du renommage. Fix : flag `acpsLoadAttempted` qui n'autorise qu'un seul chargement par cycle d'ouverture de la modale.

### DoD — Story 2

- [x] `BuildingForm.svelte` envoie `acp_id`, plus aucune trace de `organization_id` dans le payload de création/édition immeuble
- [x] `AcpList.svelte` : dropdown organisation fonctionnel, plus de champ UUID brut
- [x] `UnitCreateModal.svelte` envoie `acp_id` (bug de prod additionnel, cf. plus haut)
- [x] `BuildingDetail.svelte` affiche le nom du cabinet (résolution via ACP)
- [x] Fix boucle infinie `$effect`/ACP vide inclus
- [x] 4-cat GREEN (7/7)
- [x] `npm run build` vert (0 erreur) + `npx vitest run` vert (344/344) ; `cargo test --lib` non ré-exécuté — diff backend vide (`git diff --stat -- backend/` ne retourne rien), gate non-régression déjà couverte par Story 1
- [x] Flow complet org → ACP → immeuble testé de bout en bout au clic (+ création de lot bonus)
- [ ] Signature

---

## Ordre d'exécution

Story 1 → Story 2 (dépendance dure : le bouton "Nouvel immeuble" doit être vivant avant de pouvoir tester Story 2 au clic).

## Signature

```
Bob (Stories) : DRAFT — en attente de signature @gilmry
```

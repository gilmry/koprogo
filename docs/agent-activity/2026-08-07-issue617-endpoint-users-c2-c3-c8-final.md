# Agent activity — 2026-08-07 — #617 C2/C3/C8 clôturés (endpoint org-scopé users)

**Persona :** implémentation (Tier 2) — Stories S1/S2 signées (`docs/maury/syndic-org-users-endpoint/stories.md`, SIGNED v1.0 @gilmry 2026-08-07). Brief/PRD/Architecture/Stories tous signés avant tout code (règle CRITICAL.md #5).

**Contexte :** clôture du chantier ouvert par le brief `docs/maury/syndic-org-users-endpoint/` (issue #691), lui-même né de l'investigation #617 Phase C (C2/C3/C4/C8 bloqués par l'absence d'un endpoint `GET /users` accessible à un syndic non-superadmin).

---

## Story S1 — `GET /organizations/{id}/users`

- `UserUseCases::list_by_organization` + handler `list_organization_users` + route + OpenAPI, mirror exact de `list_organization_tickets` (`verify_org_access`, superadmin bypass).
- `backend/tests/e2e_organization_users.rs` — 7 tests 4-cat (7/7 verts, 88.92s).
- `cargo clippy --all-targets --all-features -D warnings` propre.
- `make openapi-export` + `make types-sync` régénérés et committés.
- Commit `51982510`.

## Story S2 — Migration FE (3 pages)

- `listOrganizationUsers()` dans `organizations.ts`.
- `MagicLinksPage.svelte` (nouveau wrapper) + `magic-links.astro` migré.
- `MandatesPage.svelte` / `ContractorEvaluationsPage.svelte` migrés (`organizationId` lu via `authStore`).
- Commit `1a7b1c70`.

### Vérification — branche de création réellement exercée (pas de vacuous skip)

- `mandate-issue.spec.ts` : 3 runs consécutifs, zéro flake. Logs backend : `GET /organizations/{id}/users → 200` (plus de `GET /users → 403`).
- `contractor-eval.spec.ts` : 3 runs consécutifs, zéro flake. Logs backend : `GET /organizations/{id}/users → 200`, `GET /acps → 200`, `POST /contractor-evaluations → 201`.
- `magic-link-issue.spec.ts` (4 tests, dont @happy + @negative traversent l'UI réelle) : 3 runs consécutifs, zéro flake.
- `playwright.config.ts` : les deux specs retirés du `testIgnore` (`contractor-eval.spec.ts` l'était déjà depuis le cherry-pick C5-C8).
- `npx astro check` : 0 erreur / 0 warning (baseline inchangée).
- Vitest `src/lib/components/syndic/` : 69/69 verts.

## Bugs de production découverts en cours de vérification (hors scope signé, corrigés car bloquants pour AC-S2)

En essayant de faire *réellement* passer la branche de création (pas juste garder le test vert par tolérance), trois bugs de production distincts sont apparus — tous invisibles jusqu'ici car cette branche n'était jamais exercée en E2E.

### 1. Casse de statut — `ContractorEvaluationForm.svelte` (même classe que C7)

`approvedSpecs = specs.filter(s => s.status === "Approved")` comparait en PascalCase alors que le backend sérialise `TechnicalSpecStatus` en snake_case (`"approved"`) via son `impl Display`. Le sélecteur de spec était **toujours vide/disabled en production**, quel que soit le nombre de specs réellement approuvées — C7 avait corrigé le même bug dans `TechnicalSpecDetail.svelte`/`TechnicalSpecVersionTimeline.svelte` mais pas ici. Fixé + fixtures `ContractorEvaluationForm.test.ts` alignées.

### 2. `listSpecs()` sans `acp_id` — `ContractorEvaluationsPage.svelte`

Le backend exige `acp_id` (non optionnel, `ListTechnicalSpecsQuery.acp_id: Uuid`) — l'appel `listSpecs()` sans paramètre échouait silencieusement (`.catch(() => [])`), `specs` restait toujours vide. Fix : `listAcps()` + `listSpecs(acp.id)` en parallèle sur toutes les ACP du syndic, agrégées.

### 3. Panic worker sur `GET /acps` — `PostgresAcpRepository::list()`

Découvert en vérifiant le fix #2 : `GET /acps` renvoyait systématiquement `502` (pas un 500 — le panic tue le worker actix). Cause : `list()` (branches `All` et `Organization`) ne sélectionnait pas `total_tantiemes` alors que `row_to_acp()` fait un `row.get("total_tantiemes")` strict (pas `try_get`, volontaire pour ce champ métier). La branche `Owner` avait déjà la bonne colonne — oubli sur les deux autres. **Affecte tout appelant de `GET /acps`, pas seulement ce flow** — commit séparé `9ff501da`, à surveiller si d'autres régressions apparaissent ailleurs dans le code qui dépend de cet endpoint.

### 4. `magic-link-issue.spec.ts` — bug de test pré-existant, pas produit

`injectSyndicAuth` injectait `{token, user}` dans `localStorage["koprogo_auth"]` — mécanisme devenu obsolète depuis WP-FE1 (access token en mémoire uniquement, jamais persisté). La session injectée était invalide, l'app redirigeait vers `/login` en cours de test. Remplacé par un vrai login UI (`uiLoginSyndic`), cohérent avec le pattern déjà utilisé dans `mandate-issue.spec.ts`/`contractor-eval.spec.ts`. Le mock `/users` est devenu inutile (vraies données via l'endpoint org-scopé) ; le mock tickets resserré sur `/organizations/{id}/tickets` (un pattern générique `/tickets/` interceptait aussi le widget stats du dashboard syndic et cassait le rendu de toute la page).

## Bilan #617 Phase C — clôture

| # | Spec | Statut | Nature |
|---|---|---|---|
| C2 | magic-link-issue.spec.ts | ✅ | endpoint livré (S1) + bug de test localStorage/WP-FE1 |
| C3 | mandate-issue.spec.ts | ✅ | endpoint livré (S1) + acteur admin→syndic réel + notary role direct |
| C8 | contractor-eval.spec.ts | ✅ (complet) | endpoint livré (S1) + 2 bugs produit (casing + acp_id) + panic acps |

**Reste hors scope (PRD §4, non traité ici) :**
- C4 `role-delegation.spec.ts` — `RoleDelegationsPage.svelte` non migré, syndic reste bloqué sur cette page spécifique.
- Chantier user↔ACP scoping (#694).
- `GET /users/{id}` (lookup single-user) toujours inexistant (repéré pendant C8, cf. `2026-08-06-issue617-c8-contractor-eval.md`).
- Le panic `GET /acps` (bug #3 ci-dessus) mériterait un audit des autres endpoints du même repository pour des trous similaires (colonnes manquantes vs `row.get` strict).

## Actions prises

- `backend/src/application/use_cases/user_use_cases.rs`, `user_handlers.rs`, `routes.rs`, `openapi.rs` — Story S1 (commit `51982510`).
- `backend/tests/e2e_organization_users.rs`, `backend/tests/common/mod.rs` — Story S1.
- `docs/api/openapi.json`, `frontend/src/types/api.d.ts` — régénérés (Story S1).
- `frontend/src/lib/api/organizations.ts`, `MagicLinksPage.svelte`, `magic-links.astro`, `MandatesPage.svelte`, `ContractorEvaluationsPage.svelte` — Story S2 (commit `1a7b1c70`).
- `frontend/src/lib/components/syndic/ContractorEvaluationForm.svelte` + `.test.ts` — bug casing (commit `1a7b1c70`).
- `backend/src/infrastructure/database/repositories/acp_repository_impl.rs` — bug panic (commit `9ff501da`, isolé).
- `backend/src/application/use_cases/auth_use_cases.rs` — cherry-pick fix casse email (commit `c6763704`, nécessaire pour que les acteurs syndic réels de C3/C8 puissent se logger avec des emails de test à préfixe capitalisé).
- `frontend/tests/e2e/refonte-ux/phase-b-fe/mandate-issue.spec.ts` — acteur admin→syndic + notary rôle direct.
- `frontend/tests/e2e/refonte-ux/phase-b-fe/magic-link-issue.spec.ts` — `injectSyndicAuth` → vrai login UI.
- `frontend/playwright.config.ts` — `magic-link-issue.spec.ts` et `mandate-issue.spec.ts` retirés du `testIgnore`.

## Note de transparence

Un commit (`c6763704`, cherry-pick du fix casse email) a embarqué par erreur du WIP de Story S2 (`git add -u` du hook pre-commit après `cargo fmt`/`prettier --write .`) — l'organizationId-wiring de `MandatesPage.svelte`/`ContractorEvaluationsPage.svelte` s'y trouve donc, pas dans le commit Story S2 dédié. Signalé explicitement ici et dans la description de PR pour qu'un reviewer ne soit pas surpris par le diff.

---
feature: syndic-org-users-endpoint
phase: D (Stories TOGAF)
status: DRAFT — en attente signature @gilmry
date: 2026-08-07
authors: [Claude Sonnet 5 (drafting)]
depends_on: brief.md (SIGNED v1.0), prd.md (DRAFT), architecture.md (DRAFT)
---

# Stories — Endpoint de listing des users pour le syndic (org-scopé)

> Phase D TOGAF — stories self-contained briefables par un agent fresh sans contexte session. Chaque story = goal + parent + user journey + AC 4-cat + data-testid + files exhaustifs + DoD.

## Plan d'exécution

| Story | Couche | Déps | Taille |
|---|---|---|---|
| **S1** — Endpoint `GET /organizations/{id}/users` + tests 4-cat | BE | aucune | M |
| **S2** — Migration FE (3 pages) + re-vérification C2/C3/C8 | FE | S1 | M |

Séquentiel : S2 a besoin de l'endpoint S1 pour être testable end-to-end. Pas de parallélisme utile ici (contrairement aux chantiers plus gros type Track H) — scope volontairement petit.

## Légende AC

- `@happy` chemin nominal.
- `@edge` borne (org vide).
- `@security` cross-org (syndic org A → org B), superadmin bypass.
- `@negative` défaillance correcte (pas de panic, erreur typée).

---

## Story S1 — `GET /organizations/{id}/users`

### Goal

Livrer l'endpoint org-scopé + use-case + tests 4-cat. Aucun changement FE dans cette story — le endpoint doit être testable seul (curl/Postman) avant toute migration de page.

### Parent

- Brief §CB-1, CB-2 ; PRD FR-1 ; Architecture §2.
- Pattern mirror : `ticket_handlers::list_organization_tickets`.

### User journey

1. Syndic org A (JWT `organization_id = org_A`) appelle `GET /organizations/{org_A}/users` avec son Bearer token.
2. Backend vérifie `verify_org_access(org_A)` → OK (même org).
3. Réponse `200 { "data": [ {id, email, first_name, last_name, role, organization_id, is_active, roles, active_role}, ... ] }`.
4. Le même syndic appelle `GET /organizations/{org_B}/users` (org différente) → `403 { "error": "Access denied: resource belongs to another organization" }`.
5. Superadmin appelle `GET /organizations/{n'importe_quelle_org}/users` → `200` toujours.

### AC détaillées 4-cat

#### `@happy`

- AC-S1.h1 — Syndic org A liste org A → `200`, `data` contient exactement les users de `organization_id = org_A` (mêmes champs que `GET /users` aujourd'hui).
- AC-S1.h2 — Accountant org A liste org A → `200` (même règle que syndic, `verify_org_access` ne distingue pas syndic/accountant).
- AC-S1.h3 — Superadmin liste une org quelconque → `200`.

#### `@edge`

- AC-S1.e1 — Organisation existante mais sans aucun user rattaché → `200`, `data: []`.
- AC-S1.e2 — Organisation avec un seul user (le syndic lui-même, sans contractor) → `200`, `data` contient ce seul user.

#### `@security`

- AC-S1.s1 — Syndic org A appelle avec `org_B` → `403`, message générique (pas de fuite d'info sur l'existence de org_B).
- AC-S1.s2 — Owner (rôle sans `organization_id` significatif pour ce endpoint, ou avec un org différent) → `403` si org différente, `200` si même org (owner peut légitimement lister — pas de restriction de rôle sur ce endpoint au-delà de l'org, cf. PRD — le filtrage par rôle métier reste côté FE).
- AC-S1.s3 — Requête sans Bearer token → `401` (géré par l'extracteur `AuthenticatedUser` existant, pas de code nouveau).

#### `@negative`

- AC-S1.n1 — `organization_id` dans le path non-UUID → `400` (extracteur Actix `web::Path<Uuid>`).
- AC-S1.n2 — Erreur DB simulée (mock repo) → `500` avec message safe (pas de leak SQL brut, cf. pattern `apiFetch` FE qui masque déjà les erreurs DB — ici c'est le comportement backend standard déjà en place pour `list_users`).

### data-testid

Aucun — endpoint backend pur, pas de composant FE dans cette story.

### Files exhaustifs

#### Backend

- `backend/src/infrastructure/web/handlers/user_handlers.rs` — nouveau handler `list_organization_users`.
- `backend/src/application/use_cases/user_use_cases.rs` — nouvelle méthode `list_by_organization`.
- `backend/src/infrastructure/web/routes.rs` — `.service(list_organization_users)`.
- `backend/src/infrastructure/openapi.rs` — enregistrement du path.
- `backend/tests/` — tests 4-cat (unit sur use-case + intégration testcontainers si pattern BDD existant pour `list_users`/`list_organization_tickets` le permet ; sinon unit + e2e).

### DoD-S1

- [ ] `cargo check --lib` / `cargo check --tests` propres.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` propre.
- [ ] 4-cat GREEN (AC-S1.h1 à AC-S1.n2).
- [ ] `make openapi-check` vert.
- [ ] `make types-sync` vert (régénère `api.d.ts`, à committer).
- [ ] `curl` manuel vérifié : syndic org A → 200, syndic org A → org B → 403, superadmin → n'importe quelle org → 200 (repro comme celle faite en session le 2026-08-06/07 pour C1/C4).
- [ ] Aucune régression sur `GET /users` (test existant re-passé).

---

## Story S2 — Migration FE (3 pages) + re-vérification C2/C3/C8

### Goal

Câbler les 3 pages syndic sur le nouvel endpoint. Confirmer par re-exécution des 3 specs E2E que la branche de création (jusqu'ici jamais exercée, cf. #617 C2/C3/C8) fonctionne réellement de bout en bout.

### Parent

- Brief §CB-3 ; PRD FR-2 ; Architecture §3.
- Dépend de S1 mergée (endpoint disponible).

### User journey

1. **Syndic** ouvre `/syndic/magic-links` → sélecteur "Destinataire" peuplé avec les contractors de son org → émet un lien magique → écran "issued" avec URL `/c?t=...`.
2. **Syndic** ouvre `/syndic/mandates` → sélecteur "Mandataire" peuplé → émet un mandat → row visible avec `ExpirationBadge`.
3. **Syndic** ouvre `/syndic/contractor-evaluations` → sélecteur "Contractor" peuplé → crée une évaluation → visible côté `/contractor-reputation`.

### AC détaillées 4-cat

#### `@happy`

- AC-S2.h1 — `MagicLinksPage.svelte` créé (pattern `MandatesPage.svelte`), `magic-links.astro` migré. Test `magic-link-issue.spec.ts` @happy : le sélecteur contractor a des options, le flow complet (émission → écran issued → ouverture PWA contractor) s'exécute sans skip.
- AC-S2.h2 — `MandatesPage.svelte` migré vers `listOrganizationUsers`. Test `mandate-issue.spec.ts` @happy vert (déjà vert avec acteur syndic depuis la session du 2026-08-07 — cette story rend le sélecteur réellement peuplé au lieu de vide).
- AC-S2.h3 — `ContractorEvaluationsPage.svelte` migré. Test `contractor-eval.spec.ts` @happy : la branche `if (newBtnEnabled && spec.status === "approved")` s'exécute réellement (vérifié via logs backend `GET /organizations/{id}/users → 200`, plus de `GET /users → 403`).

#### `@edge`

- AC-S2.e1 — Organisation du syndic sans aucun contractor → sélecteur vide mais **pas d'erreur** (déjà géré par le `.catch(() => [])` existant dans les 3 wrappers), bouton "Nouveau" reste cohérent avec l'état vide (comportement existant, pas de régression).

#### `@security`

- AC-S2.s1 — Aucune régression sur les AC `@security` déjà vertes de `mandate-issue.spec.ts` et `contractor-eval.spec.ts` (INV-24 append-only, non-transitivité, etc. — non touchées par ce chantier).

#### `@negative`

- AC-S2.n1 — Si `listOrganizationUsers` échoue (réseau, 500) → toast d'erreur existant (`api.ts` gère déjà), liste vide, pas de crash composant.

### data-testid

Aucun nouveau — les composants purs (`MagicLinkIssueForm`, `MandateIssueForm`, `ContractorEvaluationForm`) ne changent pas, seuls les wrappers qui les alimentent changent.

### Files exhaustifs

#### Frontend

- `frontend/src/lib/api/organizations.ts` (ou fichier le plus proche) — `listOrganizationUsers()`.
- `frontend/src/lib/components/syndic/MagicLinksPage.svelte` — **nouveau**.
- `frontend/src/pages/syndic/magic-links.astro` — monte `MagicLinksPage` au lieu du form nu.
- `frontend/src/lib/components/syndic/MandatesPage.svelte` — migration ligne ~50.
- `frontend/src/lib/components/syndic/ContractorEvaluationsPage.svelte` — migration ligne ~71.

#### Tests E2E (re-vérification, pas de nouveau fichier)

- `frontend/tests/e2e/refonte-ux/phase-b-fe/magic-link-issue.spec.ts`
- `frontend/tests/e2e/refonte-ux/phase-b-fe/mandate-issue.spec.ts`
- `frontend/tests/e2e/refonte-ux/phase-b-fe/contractor-eval.spec.ts`

### Notes anti-pattern

- Ne PAS câbler les scope kinds `quote`/`invoice`/`contractor_evaluation` de `MagicLinkIssueForm` dans cette story — seul `ticket` est exercé par les tests existants (cf. Architecture §3.2, brief §6 hors-scope).
- Ne PAS toucher `RoleDelegationsPage.svelte` (C4) — hors scope explicite (PRD §4). Un syndic réel resterait bloqué sur ce point après cette story ; à traiter séparément si confirmé nécessaire.
- Ne PAS factoriser le type `UserLike` dupliqué dans les 3 wrappers dans cette story sauf si ça tombe naturellement — pas un objectif de ce chantier (éviter le refactor-creep).

### DoD-S2

- [ ] `npx astro check` : 0 erreur, 0 warning (baseline inchangée).
- [ ] Vitest existants (`MagicLinkIssueForm.test.ts`, `MandateIssueForm.test.ts`, `ContractorEvaluationForm.test.ts`) toujours verts — composants purs non modifiés.
- [ ] `magic-link-issue.spec.ts`, `mandate-issue.spec.ts`, `contractor-eval.spec.ts` : **3 runs consécutifs chacun, zéro flake**, branche de création réellement exécutée (pas de skip).
- [ ] `frontend/playwright.config.ts` : les 3 specs retirés du `testIgnore` du projet `chromium` (rejoignent C1/C5/C6/C7 déjà actifs).
- [ ] Logs backend vérifiés en direct : `GET /organizations/{id}/users → 200` (plus de `GET /users → 403`) pendant les 3 runs.
- [ ] Log Tier-2 `docs/agent-activity/<date>-issue617-endpoint-users-c2-c3-c8-final.md` récapitulant la clôture de C2/C3/C8 (C4 reste noté comme partiellement résolu — email fixé, `/users` toujours cassé pour cette page spécifique).

## Signature

```
Mary (Brief)          : SIGNED v1.0 par @gilmry 2026-08-07
John (PRD)             : DRAFT — en attente signature @gilmry
Winston (Architecture) : DRAFT — en attente signature @gilmry
Bob (Stories)          : DRAFT — en attente signature @gilmry
```

→ Une fois signé, exécution (S1 puis S2).

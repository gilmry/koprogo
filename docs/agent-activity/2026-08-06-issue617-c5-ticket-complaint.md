# Agent activity — 2026-08-06 — #617 Phase C, C5 stabilisé (ticket-complaint.spec.ts)

**Persona :** diagnostic + fix root cause (Tier 2 — code non-prod, tests). Fix appliqué et vérifié 3x sans flake.

**Contexte :** suite de C4. Sous-tâche **C5** (`ticket-complaint.spec.ts`, Story B5).

---

## Root cause — bug de test (ordre de capture de l'URL)

Contrairement à C2/C3/C4, ce spec utilise déjà les bons acteurs (owner-plaignant puis syndic, jamais admin) et ne dépend pas du endpoint `/users` cassé — l'owner crée son ticket via `/tickets/new`, pas via un sélecteur de destinataire.

Le test capturait `page.url()` **après** `logoutUi(page)` + `uiLogin(page, syndic...)` pour en extraire l'id du ticket (`/ticket-detail?id=...`) — mais à ce moment, `page.url()` pointe déjà vers le dashboard du syndic (`/syndic`), pas vers la page ticket-detail de l'owner (écrasée par la navigation de login). Le regex `id=` ne matchait donc jamais, le `if` restait silencieusement no-op, et le test terminait sur `/syndic` au lieu de `/ticket-detail` — d'où l'échec sur `ticket-detail-title` introuvable.

**Fix** : capture de `ticketUrl`/`ticketIdMatch` déplacée juste après `page.waitForURL(/\/ticket-detail/)` (pendant que l'owner est encore connecté), avant le `logoutUi`. Ajout d'une assertion explicite `expect(ticketIdMatch).not.toBeNull()` pour que ce genre de régression échoue bruyamment plutôt que silencieusement à l'avenir.

## Résultat

1/1 test vert, **3 runs consécutifs sans flake**.

## Actions prises

- `frontend/tests/e2e/refonte-ux/phase-b-fe/ticket-complaint.spec.ts` — fix ordre de capture URL.
- `frontend/playwright.config.ts` — `ticket-complaint.spec.ts` retiré du `testIgnore` du projet `chromium` (vérifié : `--project=chromium --list` ne remonte plus que ce seul test dans `phase-b-fe/`, les 6 autres specs C2/C3/C4/C6/C7/C8 restent exclus).

## Restant sur #617

C6 (`syndic-response-sla.spec.ts`), C7 (`technical-spec-flow.spec.ts`), C8 (`contractor-eval.spec.ts`) non investigués. C8 partage probablement le même trou `/users` que C2/C3/C4 (`ContractorEvaluationsPage.svelte:71`, déjà identifié dans l'investigation C2).

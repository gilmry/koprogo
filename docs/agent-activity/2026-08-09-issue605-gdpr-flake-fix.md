# Agent activity — 2026-08-09 — Fix #605 (Gdpr Playwright flaky)

**Persona :** investigation + fix (Tier 2, un seul fichier de test touché).

**Contexte :** demande utilisateur « attaque 605 » — dernier bloquant go-live réellement ouvert après le resync WBS du 2026-08-08 (`85e55997`).

## Investigation

Reproduction en 3 étapes :

1. `npx playwright test Gdpr.spec.ts` (workers par défaut, séquentiel) → 5/5 vert.
2. `--repeat-each=3` avec workers parallèles (défaut multi-cœurs local) → 4/15 échecs, tous liés à des `POST /auth/refresh` → 401 en cascade (trace réseau capturée). Root cause probable : collision sur le compte admin fixe partagé (`admin@koprogo.com`) entre workers parallèles — **non représentatif de CI**, qui tourne avec `workers: 1` pour le projet `chromium` (`playwright.config.ts`).
3. `--workers=1 --repeat-each=5` (config CI réelle) → 1/25 échec (4%), même symptôme : `page.goto("/admin/gdpr")` juste après `loginAsSuperAdmin()` redirige vers `/login?redirect=...` avec un 401 en console.

Vérification CI réelle (job `Playwright E2E Tests`, run du 2026-08-08 16:59, commit `3291f1b1`) : Gdpr.spec.ts 5/5 vert — confirme que le flake est rare (~4%), pas systématique.

## Root cause

`loginViaUI()` (`tests/e2e/Gdpr.spec.ts:65-74`) fait `waitForURL(...)` puis rend la main immédiatement. Le cookie de refresh `HttpOnly` posé par `authStore.login()` n'a pas forcément fini de se stabiliser côté navigateur quand le test enchaîne une **2e navigation complète** (`page.goto("/admin/gdpr")`). Cette 2e navigation déclenche son propre silent-refresh (`RouteGuard` → `authStore.init()`), qui peut échouer en 401 si la course n'est pas absorbée → redirect `/login`.

Même classe de bug que celui déjà rencontré et corrigé ce même jour dans `story1-admin-buttons.spec.ts` (course cookie/refresh sur double navigation), mais ici côté login **UI réel** (pas `injectAuth`). Le pattern correct existe déjà ailleurs dans le repo : `refonte-ux/phase-b-fe/role-assignment.spec.ts`'s `humanLogin()` fait `waitForURL` **puis** `waitForLoadState("networkidle")` avant de continuer — `Gdpr.spec.ts` ne l'avait jamais adopté.

## Fix

`loginViaUI()` : ajout de `await page.waitForLoadState("networkidle").catch(() => undefined);` après le `waitForURL`, alignant le comportement sur `humanLogin()`.

## Vérification

- `--workers=1 --repeat-each=6` (30 exécutions, config CI) : **30/30 vert** (vs 1/25 échec avant fix).
- `npm run build` : 0 erreur, 115 pages.

## Différé

Le comportement observé sous parallélisme (workers>1, hors config CI actuelle) — collision potentielle sur le compte admin fixe partagé entre workers concurrents — n'est pas traité ici : non reproduit en CI réelle (`workers:1`), hors scope de #605 qui documente spécifiquement le comportement séquentiel. À rouvrir si la config CI passe un jour en parallèle sur ce projet.

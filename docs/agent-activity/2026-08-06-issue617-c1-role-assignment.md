# Agent activity — 2026-08-06 — #617 Phase C, C1 role-assignment.spec.ts stabilisé

**Persona :** diagnostic + fix root cause (Tier 2 — code non-prod, tests, doc ; aucun merge/push/tag).

**Contexte :** suite de la session « avancer le WBS » (cf. `2026-08-06-wbs-status-resync.md`). Choix utilisateur : poursuivre le diagnostic de l'issue [#617](https://github.com/gilmry/koprogo/issues/617) (Phase C — Documentation Vivante e2e), sous-tâche **C1** (`role-assignment.spec.ts`, Story B1).

---

## Méthode de reproduction fidèle

Plusieurs harnais de debug se sont révélés être des artefacts (pas le bug réel) :
- `docker compose run frontend` avec réécriture DNS `localhost`→Traefik : Chromium résout `localhost` en interne (bypass hosts file) → échecs de navigation non représentatifs.
- Pointer directement `frontend:3000` : bloqué par `server.allowedHosts` de Vite.
- `PUBLIC_API_URL=http://backend:8080` (cross-origin réel) : cookie `SameSite=Strict` jamais envoyé → 401 en boucle non représentatif.

**Méthode retenue** (fidèle à la topologie CI/prod — same-origin via Traefik) :
```bash
docker run --rm --network host \
  -v "$(pwd)/frontend:/app" -v /app/node_modules -w /app \
  -e PLAYWRIGHT_BASE_URL=http://localhost -e PLAYWRIGHT_API_BASE=http://localhost/api/v1 \
  koprogo-frontend:latest npx playwright test <spec> --project=chromium
```
Réutilisable telle quelle pour C2-C8.

## Root cause #1 (bloquant @edge) — `valid_until` jamais persisté

`backend/src/infrastructure/database/repositories/user_role_repository_impl.rs` : les 5 requêtes SQL (`create`, `list_for_user`, `list_for_users`, `find_by_id`, `set_primary_role`, `replace_all`) ne projetaient ni `valid_until` ni `delegated_from_user_id` — colonnes ajoutées par la migration `20260605030000_extend_user_roles_delegation.sql` (Story 3.5, délégation temporaire). Le domaine et le DTO API étaient corrects ; `map_row` avait même un commentaire documentant explicitement le gap (« legacy SELECT queries that do not yet project these columns »). Un repository parallèle dédié (`role_delegation_repository_impl.rs`, feature RoleDelegation) avait lui bien les bonnes colonnes — seul l'ancien repository (endpoint admin `POST /users/{id}/role-assignments`, Story 3.1/B0bis) était resté désynchronisé.

**Vérifié en direct** : `curl POST .../role-assignments` avec `valid_until` renvoyait `valid_until: null` avant fix, `valid_until: "2026-08-06T23:59:59Z"` après.

**Fix** : ajout des 2 colonnes dans les 5 requêtes SQL du fichier. `cargo check --lib` propre (exit 0), `cargo fmt --check` propre.

## Root cause #2 (bloquant @security) — le HTML statique pré-généré au build affiche le panel avant le gate client

`astro.config.mjs` (`output: "static"`) : KoproGo est en SSG, pas en SSR — le HTML est généré une fois au build, identique pour tout visiteur, puis les îlots Svelte s'hydratent côté client. `/admin/*` est réservé `SUPERADMIN` (`frontend/src/lib/guards.ts`), mais le HTML statique du panel (bouton « Nouvelle assignation » inclus) est déjà dans la page avant que `RouteGuard.svelte` (`client:load`) n'ait fini son check et ne redirige un syndic. Le backend reste la vraie frontière (403 réel, testé plus bas dans le spec) — c'est un flash UI pré-hydratation, pas une fuite de données. **C'est la dette déjà documentée et différée dans le WBS** (`### Track C — Frontend sécurité (refacto #343 / SSR client:load DIFFÉRÉS post-bêta)` — le titre de ce Track cite "SSR" par raccourci historique pour "rendu du contenu avant hydratation client", à ne pas lire comme une confirmation que le site tourne en mode SSR) : je n'ai donc **pas** touché à l'architecture RouteGuard/hydratation (ça contredirait une décision produit déjà actée et violerait la règle CRITICAL.md #5 « brief/PRD/architecture signés avant de coder »).

**Fix** (test seul) : `role-assignment.spec.ts`, test `@security` — remplacement du `click()` synchrone par une course `Promise.race` entre le clic et une navigation-away, qui traite la redirection concurrente comme un cas conforme (même intention que documentée dans le commentaire original du test, juste mal implémentée).

## Résultat

4/4 tests du spec verts, **3 runs consécutifs sans flake** (DoD #617). `frontend/playwright.config.ts` : `role-assignment.spec.ts` retiré du `testIgnore` du projet `chromium` (C2-C8 restent exclus individuellement, pas de régression sur le reste du gate — vérifié `npx playwright test --project=chromium --list` = exactement 4 tests dans `phase-b-fe/`).

## Fichiers modifiés (Tier 2, non commités)

- `backend/src/infrastructure/database/repositories/user_role_repository_impl.rs` — fix persistance.
- `frontend/tests/e2e/refonte-ux/phase-b-fe/role-assignment.spec.ts` — fix race @security.
- `frontend/playwright.config.ts` — réactivation ciblée C1 dans le gate `chromium`.

## Restant sur #617

C2 (`magic-link-issue.spec.ts`) à C8 (`contractor-eval.spec.ts`) toujours exclus et non investigués cette session.

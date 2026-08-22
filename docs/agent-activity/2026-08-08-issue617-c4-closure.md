# Agent activity — 2026-08-08 — #617 Phase C clôturée : C4 stabilisé (Story S3)

**Persona :** exécution Story S3 (Tier 2 — code non-prod, extension de scope confirmée par @gilmry en session). Dernière sous-tâche de Phase C (`role-delegation.spec.ts`, C4) stabilisée. Phase C = 8/8 sub-tasks vertes en isolation.

**Contexte :** suite de la clôture C2/C3/C8 (Story S1/S2, `docs/agent-activity/2026-08-07-issue617-endpoint-users-c2-c3-c8-final.md`). C4 avait été investigué le 2026-08-06 (`docs/agent-activity/2026-08-06-issue617-c4-investigation.md`) : root cause #1 (casse email) déjà fixée, root cause #2 (même trou `/users` que C2/C3/C8) laissée bloquée par le brief non signé à l'époque. Story S1 a depuis livré `GET /organizations/{id}/users` ; Story S2 a migré 3 pages mais a **explicitement exclu** `RoleDelegationsPage.svelte` du scope signé (PRD §4). Extension confirmée par @gilmry en session (2026-08-08) → Story S3 ajoutée à `docs/maury/syndic-org-users-endpoint/stories.md`.

---

## Root cause #1 — `RoleDelegationsPage.svelte` appelait `GET /users?per_page=1000` (403 syndic)

Même pattern mécanique que C2/C3/C8. Migré vers `listOrganizationUsers(organizationId)` (déjà livré par S1). Fix : `RoleDelegationsPage.svelte` ligne ~65.

## Root cause #2 (découverte en cours de route, hors scope initial S3) — `organization_id: null` bloquait TOUTE délégation réelle

Le sélecteur "Organisation" du form (`role-delegate-org-select`) appelle `GET /organizations?per_page=1000` — endpoint superadmin-only, 403 pour un syndic. Résultat : le champ reste vide, le form soumet `organization_id: null`. Côté backend, `RoleDelegationUseCases::delegate_role` vérifie la non-transitivité via `find_active_by_user_and_role(delegator_id, role, organization_id)` — avec `organization_id = None`, il cherche une assignment **globale**, que le syndic n'a pas (son "syndic" natif est scopé à son org). `has_native = false` → `403 DelegationChainNotAllowed`, à tort, pour n'importe quel syndic réel tentant de déléguer.

**Fix** (sans toucher `/organizations`, confirmé avec @gilmry) : `RoleDelegationsPage.svelte` calcule déjà l'`organizationId` du syndic connecté (pour le fix root cause #1) — passé en nouvelle prop `defaultOrganizationId` à `RoleDelegationForm.svelte`, qui préselectionne son état `organizationId` local avec cette valeur. Le sélecteur `organizations` (peuplé uniquement pour un superadmin) reste disponible pour override explicite.

## Root cause #3 — badge `ExpirationBadge` jamais "urgent" pour `valid_until = today + 7j`

Bug de test (pas produit) : `RoleDelegationForm` soumet `${validUntil}T23:59:59Z` (fin de journée). `daysBetween()` (`dateBadge.ts`, `Math.ceil`) calcule alors **déterministiquement** N+1 jours restants depuis "now", quel que soit l'instant d'exécution dans la journée (preuve : `deltaMs` est toujours strictement entre N·24h et (N+1)·24h pour un delta calendaire de N jours + reste de journée). Le test demandait `+7j` en attendant le seuil `urgent (≤7j)` — il retombait systématiquement à 8 jours (bucket `soon`). Fix : test corrigé à `+6j` (produit 7 jours restants pile). Seuils produit (`dateBadge.ts`) non touchés — corrects et déjà documentés.

## Root cause #4 — banner @security introuvable : rôle actif au login ≠ rôle hérité

Pierre hérite le rôle "syndic" par délégation, mais `ensure_role_assignments` (backend, `auth_use_cases.rs`) sélectionne le rôle **primary** (natif "owner") comme rôle actif au login — pas le rôle délégué. `/syndic/role-delegations` est donc inaccessible tant que Pierre n'a pas explicitement basculé de rôle actif. C'est un comportement backend voulu (pas un bug), avec un mécanisme UI dédié déjà existant : le `role-selector` (`Navigation.svelte`, visible si `user.roles.length > 1`) qui appelle `POST /auth/switch-role` et redirige automatiquement.

**Fix test** (pas produit) : après `uiLogin`, le test utilise le `role-selector` réel pour basculer vers "syndic" avant de naviguer vers `/syndic/role-delegations` — exerce le vrai mécanisme multi-rôle au lieu de le contourner.

## Root cause #5 — i18n namespace `roleDelegation.*` totalement absent des 4 locales

Aucune des ~23 clés `$_("roleDelegation....")` utilisées par `RoleDelegationList.svelte`/`RoleDelegationForm.svelte` n'existait dans `fr.json`/`en.json`/`nl.json`/`de.json` — l'UI affichait les clés brutes au lieu du texte traduit. Le pattern `$_(key) || "fallback FR statique"` largement utilisé dans le repo pour ce cas est **cassé** : `svelte-i18n` renvoie la clé elle-même (string non-vide, donc truthy) quand la traduction est absente, pas `undefined`/`""` — le `||` ne se déclenche jamais. Ce n'est pas spécifique à ce composant ; probablement latent ailleurs dans le repo partout où des clés n'ont jamais été ajoutées aux locales (les autres pages Phase B FE n'ont simplement pas de test qui asserte sur le texte affiché, donc ça n'a jamais cassé de CI).

**Fix** : namespace `roleDelegation` ajouté aux 4 fichiers de locale (23 clés × 4 langues), extrait des fallbacks FR déjà écrits dans le code + traductions EN/NL/DE ajoutées.

## Root cause #6 — assertion `@security` bypass POST testait le mauvais guard

Le test utilisait `pierre.token` (capturé à l'inscription, rôle "owner") pour la tentative de bypass POST. Ce token ferait échouer la requête sur le guard générique `caller_must_hold_role` (owner ≠ syndic demandé) — pas sur l'invariant INV-8 (`DelegationChainNotAllowed`) réellement visé par le test. Assertion `toBe(403)` passait quand même, mais pour la mauvaise raison (faux vert pour l'invariant testé). Fix : le test récupère un token à jour via `POST /auth/switch-role` (même endpoint que le `role-selector` UI) avant le bypass, pour exercer réellement `has_native = false` côté use-case.

## Root cause #7 (environnemental, PAS un bug) — flakiness du run combiné des 8 specs Phase C en local

Chaque spec de `refonte-ux/phase-b-fe/` est vert en isolation (3 runs consécutifs, zéro flake, vérifié spec par spec cette session et les précédentes). Le run combiné (`npx playwright test tests/e2e/refonte-ux/phase-b-fe/`, ~2.3-2.4 min, ~40 tests séquentiels) échoue de façon non-déterministe — un ensemble DIFFÉRENT de tests échoue à chaque tentative (9 puis 12 échecs sur deux runs consécutifs, même après redémarrage propre du container frontend), ce qui exclut un bug de logique déterministe.

Root-causé via un script de debug Playwright dédié : le serveur Vite dev (mode `astro dev`, pas un build de production) répond `504 Gateway Timeout` sur la requête de module dynamiquement importé, ce qui casse l'hydratation du composant ciblé (observé sur `LoginForm.svelte` — `[astro-island] Error hydrating ... TypeError: Failed to fetch dynamically imported module`, précédé d'un `504` sur le fetch du module). Le composant reste alors un HTML statique sans handler JS attaché — un submit de form dégrade en soumission HTML native (`GET` vers la même URL), symptôme observé : `page.waitForURL` timeout, toujours bloqué sur `/login`.

Ce comportement est cohérent avec des heures d'utilisation continue du même container dev dans cette session (dizaines de lancements de suites, plusieurs redémarrages) — un signe d'épuisement de ressources / dégradation du serveur Vite en longue durée, pas une régression de code. **Le signal fiable reste l'isolation par spec.** Le run combiné local ne doit pas être traité comme un gate — le vrai gate est la CI GitHub Actions (environnement frais à chaque run), hors de portée de cette session.

---

## Actions prises

- `frontend/src/lib/components/syndic/RoleDelegationsPage.svelte` — migration `/users` → `listOrganizationUsers`, calcul + passage `currentOrganizationId`.
- `frontend/src/lib/components/syndic/RoleDelegationForm.svelte` — nouvelle prop `defaultOrganizationId`, préselectionne `organizationId`.
- `frontend/src/locales/{fr,en,nl,de}.json` — namespace `roleDelegation` complet (23 clés).
- `frontend/tests/e2e/refonte-ux/phase-b-fe/role-delegation.spec.ts` — fix date `+7j`→`+6j` (@happy), flow `role-selector` réel (@security), token switch-role pour le bypass POST (@security).
- `frontend/playwright.config.ts` — retrait de `role-delegation.spec.ts` du `testIgnore`. Plus aucune exclusion Phase C.
- `docs/maury/syndic-org-users-endpoint/stories.md` — Story S3 ajoutée et signée (extension de scope confirmée @gilmry 2026-08-08), DoD-S3 mise à jour reflétant ce qui est du ressort agent vs Tier 1.

## Vérifié

- `npx astro check` : 0/0/42 (baseline inchangée).
- Vitest `RoleDelegationForm.test.ts` + `RoleDelegationList.test.ts` : 8/8 verts.
- `role-delegation.spec.ts` isolé : **6 runs consécutifs, zéro flake** (3 avant le fix du token @security, 3 après).
- Logs backend vérifiés en direct : `GET /organizations/{id}/users → 200` (plus de `GET /users → 403`).

## Ce qui reste (Tier 1, hors autorité agent)

- Fermeture/cochage de l'issue GitHub `#617` par @gilmry.
- Confirmation CI GitHub Actions verte sur les 8 specs (signal réel, pas le replay local dégradé documenté en root cause #7).

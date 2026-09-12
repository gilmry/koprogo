/**
 * Où la campagne tape — en UN seul endroit.
 *
 * ── Ce qui est arrivé, et qui n'a tenu qu'à un redirect ────────────────────
 *
 * `const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1"`
 * était recopié dans **93 fichiers**. Sur l'hôte qui porte la démo, ce défaut
 * ne désigne pas « le backend local » : le port 80 y est tenu par le Traefik
 * de la démo, qui route vers `Host(api.koprogo.com)`.
 *
 * Mesuré le 2026-09-12, au premier lancement de la pile de recette : `make
 * test-e2e` n'exportait que `PLAYWRIGHT_BASE_URL`. Les 93 fichiers sont donc
 * retombés sur le port 80 et **57 specs sur 106 ont échoué**, toutes à
 * l'amorçage.
 *
 * Ce qui a sauvé la démo n'est pas une garde, c'est un hasard de
 * configuration : le port 80 rend `301 → https://localhost`, qui n'aboutit
 * pas. Si ce Traefik avait servi la requête, la campagne aurait créé ses
 * organisations, ses immeubles et ses comptes **dans la base vivante** — et
 * `make seed-reset` aurait fait le reste.
 *
 * ── Pourquoi un module, et pas juste corriger les 93 ──────────────────────
 *
 * C'est exactement l'histoire de `identifiants.ts`, qui centralise le mot de
 * passe du superadministrateur après l'avoir vu « écrit en dur à quarante-cinq
 * endroits ». Une valeur recopiée n'a pas de lieu où être corrigée : la
 * prochaine bascule de port aurait rouvert 93 fois le même défaut.
 *
 * ── Comment surcharger ────────────────────────────────────────────────────
 *
 *     make test-e2e RECETTE=http://localhost:3000
 *     PLAYWRIGHT_API_BASE=http://localhost:8080/api/v1 npx playwright test
 *
 * La CI pose les deux variables explicitement (`ci.yml` : `:3000` pour le
 * navigateur, `:8080` pour l'API) : ce défaut ne la concerne pas.
 */

/**
 * La racine de l'API, telle que la campagne l'appelle pour AMORCER son monde
 * — organisations, immeubles, comptes, jeux d'essai.
 *
 * À distinguer de `PLAYWRIGHT_BASE_URL`, qui dit où le NAVIGATEUR va. Les
 * deux peuvent différer (en CI elles diffèrent), et n'exporter que la seconde
 * est précisément le défaut que ce module existe pour empêcher.
 */
export const API_BASE =
  process.env.PLAYWRIGHT_API_BASE || "http://localhost:8090/api/v1";

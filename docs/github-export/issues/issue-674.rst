=====================================================================================================================================
Issue #674: security(frontend): 10 vulns high npm audit résiduelles (workbox-build/PWA + @redocly/openapi-core) — nécessitent --force
=====================================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: security
:Assignees: Unassigned
:Created: 2026-07-26
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/674>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Suite à la PR #673 (fix CI feature/dev), `npm audit fix` (sans `--force`) a réduit les vulnérabilités frontend de 11 (1 low, 1 moderate, 9 high) à 10 (toutes high). Les 10 restantes ne peuvent plus être corrigées sans bump majeur (`--force`), non tenté dans #673 pour ne pas élargir le scope d'un fix ciblé CI.
   
   ## Vulnérabilités restantes (10 high)
   
   Deux chaînes de dépendances distinctes, toutes **dev-only** (outils de build, pas de code livré au navigateur en prod) :
   
   **Chaîne 1 — PWA build (`workbox-build`)** :
   - `workbox-build` → `@trickfilm400/rollup-plugin-off-main-thread` → `ejs` → `jake` → `filelist` → `minimatch` → `brace-expansion` (GHSA-mh99-v99m-4gvg, DoS via unbounded expansion)
   - `@babel/plugin-transform-modules-systemjs` (GHSA-fv7c-fp4j-7gwp, génération de code arbitraire sur input malicieux) — même chaîne de build PWA
   
   **Chaîne 2 — Génération OpenAPI (`@redocly/openapi-core`)** :
   - `@redocly/openapi-core` → `js-yaml` (GHSA-52cp-r559-cp3m, DoS quadratique via merge-key chains) + `minimatch`/`brace-expansion` (partagé avec chaîne 1)
   
   ## Pourquoi "dev-only" atténue le risque (mais ne l'annule pas)
   
   Ces packages tournent au build-time (génération du service worker PWA, génération de la doc OpenAPI) — sur inputs contrôlés par l'équipe (le code source du repo), pas sur des inputs utilisateur en prod. Risque réel faible en l'état (cf. `project_koprogo-current-state` : v0.1.0 non-prod). Mais `npm audit --audit-level=high` bloque le pre-push hook local (`make ci`) tant que ce n'est pas résolu ou explicitement accepté quelque part.
   
   ## Recette proposée
   
   1. Tenter `npm audit fix --force` sur une branche dédiée, isolée du reste.
   2. Vérifier après coup :
      - [ ] `npm run build` (build PWA complet, vérifier que le service worker généré fonctionne)
      - [ ] `make openapi-export` (génération OpenAPI toujours correcte)
      - [ ] `npx vitest run` (344 tests) + `npx astro check` (0 erreur) restent verts
   3. Si `--force` casse quelque chose (bump majeur de `workbox-build` ou `@redocly/openapi-core` a de bonnes chances de changer des APIs), évaluer une alternative : `overrides` npm ciblés sur `js-yaml`/`brace-expansion`/`minimatch` sans bumper les packages parents, ou accepter/documenter le risque quelque part (pas d'équivalent npm au `.cargo/audit.toml` du backend actuellement — à créer si on choisit cette voie).
   
   ## Notes
   
   - `npm audit fix` non-force est **safe et déjà appliqué** dans #673 — seule cette 2e étape (bump majeur) reste.
   - Distinct de #432 (vulnérabilités Dependabot sur `main`, périmètre différent — `main` très en retard sur `feature/dev`) et #636 (vulnérabilités **Rust**, déjà closes par #673).
   
   Refs: #634, #673

.. raw:: html

   </div>


====================================================================================================================
Issue #634: Frontend cassé par 4 bumps Dependabot majeurs (astro 7 / svelte 9 / astrojs-node 11 / @astrojs-svelte 9)
====================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug
:Assignees: Unassigned
:Created: 2026-06-25
:Updated: 2026-06-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/634>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Depuis l'auto-merge de 10 PR Dependabot (#623–#633) sur \`feature/dev\`, le **frontend est cassé**. Surgi au premier run CI Pipeline complet (commit \`5ff2608\`, Track H H12) — les commits Dependabot eux-mêmes étaient en \`action_required\` (CI Pipeline **n'a jamais tourné** dessus avant merge).
   
   Jobs rouges (frontend uniquement — **backend vert** : Unit/oasdiff/E2E ✓) :
   - ❌ `Frontend Check & Build`
   - ❌ `Frontend Unit Tests (vitest)`
   - ❌ `Contract Types Check (end-to-end anti-drift)`
   - ❌ `Docker Build and Push to GHCR` (job `build-and-push-frontend`)
   
   ## Cause
   
   4 bumps **majeurs** simultanés, non testés avant merge (gap `action_required` sur les runs Dependabot) :
   - **astro 6.4.8 → 7.0.2** (#630)
   - **@astrojs/svelte 8.1.2 → 9.0.0** (#629)
   - **@astrojs/node 10.1.4 → 11.0.0** (#626)
   - **svelte 9** (groupe npm)
   
   Symptôme bloquant `npm ci` (Docker prod) : override obsolète dans `frontend/package.json` (l. 73–75) —
   `"@vite-pwa/astro": { "astro": "^6.1.2" }` force le peer astro à 6 alors que la racine est en astro ^7 → ERESOLVE.
   Au-delà du `npm ci`, le **build astro réel** + **vitest** + **contract-types** cassent aussi (breaking changes astro 7 / adapter node 11 / @astrojs/svelte 9).
   
   ## Recette proposée
   
   Décision PO @gilmry (2026-06-25) : **laisser le frontend rouge à part**, ne pas bloquer Track H (backend). Deux options pour la résolution dédiée :
   1. **Revert** des 4 bumps majeurs (restaure le vert immédiatement ; bumps **fonctionnels**, pas sécurité → aucune perte sécu), puis migration délibérée et testée plus tard.
   2. **Fix-forward** : migration frontend astro 7 (config + adapter node 11 + svelte 5 runes + override @vite-pwa + régénération lock + contract-types).
   
   Gap process à corriger : l'auto-merge Dependabot doit attendre que **CI Pipeline** (pas seulement les checks de PR) soit vert — les majeures front contournent le gate via `action_required`.
   
   ## Critères d'acceptation
   - [ ] `Frontend Check & Build`, `Frontend Unit Tests`, `Contract Types Check`, `Docker Build (frontend)` verts sur `feature/dev`.
   - [ ] `npm ci` (Dockerfile prod) passe sans `--legacy-peer-deps`.
   - [ ] Override `@vite-pwa/astro`→astro cohérent avec la version d'astro retenue.
   - [ ] Décision tracée : revert OU migration (avec la version cible de chaque dépendance).
   - [ ] Garde-fou : auto-merge Dependabot conditionné au vert de CI Pipeline (pas seulement PR checks).
   
   Refs: #623 #624 #625 #626 #628 #629 #630 #631 #632 #633 · commit 5ff2608

.. raw:: html

   </div>


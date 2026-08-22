# Agent activity — 2026-08-08 — #634 (partiel) : bump vite 7→8, débloque `npm run build`

**Persona :** diagnostic + fix root cause (Tier 2 — dépendance de build, pas de code applicatif). Demandé explicitement par @gilmry pour débloquer la validation CI de `#617` (le job `Playwright E2E Tests` dépend de `frontend-check`, qui échouait sur `npm run build`).

**Contexte :** en vérifiant la CI GitHub Actions du push `387128c4` (#617 Story S3), le job `Frontend Check & Build` échoue sur l'étape "TypeScript check" (en réalité `npm run build` = `astro check && astro build`, cf. `.github/workflows/ci.yml:448-451`). `astro check` passe (0 erreur), mais `astro build` casse avec :

```
rollupOptions.input should not be an html file when building for SSR. Please specify a dedicated SSR entry.
```

malgré `astro.config.mjs` configuré `output: "static"`. Cette erreur était **déjà présente** sur le push précédent (`0f17f874`, avant tout mon travail Story S3) — confirmée non-régressive, mais bloquante pour la validation CI de `#617` puisque `playwright` (job E2E) a `needs: [frontend-check]`. Déjà tracée dans le WBS comme `#634` ("frontend Dependabot — build issues, ERESOLVE astro 7 ⊥ @vite-pwa/astro").

## Root cause

`astro@7.1.6` déclare `"vite": "^8.0.13"` en dépendance directe (`node_modules/astro/package.json`). Le repo avait un override racine `"vite": "^7.3.2"` (`frontend/package.json` `overrides`), forçant TOUTE résolution `vite` — y compris celle interne d'astro — vers la branche 7.x. Astro 7.1.6 est écrit contre l'API Environments de Vite 8 ; sous Vite 7, la résolution de `rollupOptions.input` pour l'environnement "prerender" (utilisé par Astro même en output `static`, cf. `astro/dist/core/build/static-build.js`) tombe en erreur au lieu de recevoir la liste des pages.

Repro isolée (`astro.config.mjs` réduit à `{output: "static"}` sans aucune intégration) : même erreur → élimine `@vite-pwa/astro`/tailwind/svelte comme cause, confirme que c'est purement la version de `vite`.

**Pourquoi l'override `vite@7` existait** : `@vite-pwa/astro@1.2.0` dépend de `vite-plugin-pwa@^1.2.0`, dont la version installée (1.2.0) a `peerDependencies.vite: "^3 || ^4 || ^5 || ^6 || ^7"` — **pas** de support Vite 8. Sans l'override, `npm install` produisait un ERESOLVE (astro veut vite 8, vite-plugin-pwa@1.2.0 plafonne à vite 7). L'override a résolu le conflit npm en sacrifiant le fonctionnement réel du build astro.

**La vraie sortie** : `vite-plugin-pwa@1.3.0` (dernière version, déjà publiée, satisfait le peer-range `^1.2.0` de `@vite-pwa/astro`) déclare `peerDependencies.vite: "... || ^8.0.0"`. `@tailwindcss/vite@4.3.3` et `@astrojs/svelte@9.0.1` supportent aussi vite ^8. Aucune incompatibilité réelle une fois `vite-plugin-pwa` mis à jour — l'override vite@7 n'était plus nécessaire, juste périmé.

## Fix

`frontend/package.json` :
- `dependencies.vite` : `^7.3.2` → `^8.0.13` (plancher exact déclaré par astro, choix conservateur plutôt que la dernière mineure).
- `overrides.vite` : `^7.3.2` → `^8.0.13`.
- `overrides["vite-plugin-pwa"]` : ajouté `^1.3.0`.

`npm install` → résolu en `vite@8.2.1`, `vite-plugin-pwa@1.3.0`. `package-lock.json` régénéré.

## Vérifié

- `npm run build` (= `astro check && astro build`) : **0 erreur**, 115 pages générées, `[build] Waiting for integration "@vite-pwa/astro-integration"...` puis `Complete!`.
- **Artefacts PWA vérifiés présents dans `dist/`** (pas juste un exit code 0 — le plugin PWA aurait pu silencieusement ne rien produire) : `sw.js` (27 KB), `manifest.webmanifest`, `registerSW.js`, `workbox-*.js`. Feature offline (`src/lib/sync.ts`, `indexeddb.ts`, `pwa.ts`, `pwa-contractor.spec.ts`) donc non régressée.
- `npx astro check` : 0 erreur / 0 warning / 42 hints (baseline inchangée).
- `npx vitest run` : 344/344 tests verts (48 fichiers).
- Image docker `koprogo-frontend:latest` reconstruite (`docker compose build frontend`) pour refléter les nouvelles deps dans les runs Playwright locaux.
- `role-delegation.spec.ts` (mode dev, container redémarré proprement) : **3 runs isolés consécutifs, zéro flake** — confirme que le bump Vite 8 n'a pas dégradé le comportement du dev-server (contexte : root cause #7 du log C4, flakiness liée à des `504` du dev-server Vite sous charge).
- `npm audit --audit-level=high` : passe de 4 à **3** vulnérabilités hautes (le paquet `nanoid` vulnérable disparaît, résolu transitivement par le bump). Restent : `@babel/plugin-transform-modules-systemjs`, `js-yaml`/`@redocly/openapi-core` — pré-existants, sans rapport (deps de `openapi-typescript`), toujours bloquants pour le hook pre-push (`--no-verify` toujours nécessaire pour ces 3-là).

## Portée

Ce fix résout la cause concrète de `#634` pour la partie "build astro cassé" — mais `#634` liste aussi vitest/contract/docker-frontend/NPM audit comme symptômes English, pas tous vérifiés ici. Ne pas clore `#634` sans revue humaine de son scope complet.

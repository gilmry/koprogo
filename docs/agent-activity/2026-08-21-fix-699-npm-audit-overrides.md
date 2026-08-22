# Agent activity — 2026-08-21 — Fix #699 : npm audit 3 vulns high via overrides ciblés

**Persona :** correction de bug (Tier 2, code non-prod, dépendances dev-only).

**Contexte :** suite au fix #695 (même session, même round), poursuite de l'avancée WBS avec plusieurs fixes groupés par round plutôt qu'un fix isolé à la fois (retour explicite utilisateur).

## Constat au 2026-08-09 (état documenté par l'issue)

`npm audit` (frontend) : 3 vulns high — `@babel/plugin-transform-modules-systemjs` (chaîne `@vite-pwa/astro`→`workbox-build`) + `js-yaml`/CVE-2026-59870 (chaîne `@redocly/openapi-core`). `npm audit fix --dry-run` ne résolvait rien à l'époque (pas de version non-breaking disponible en amont).

## Re-vérification 2026-08-21

12 jours plus tard, des patch releases amont existent désormais :

- `@babel/plugin-transform-modules-systemjs` : plage vulnérable `7.12.0-7.29.0` (GHSA-fv7c-fp4j-7gwp), pinné exactement à `7.29.0`. `7.29.7`/`7.29.8` publiés depuis, hors de la plage vulnérable. Le parent (`@babel/preset-env`) déclare `^7.29.0`, qui autorise déjà `7.29.8` — aucun conflit de range.
- `js-yaml` : plage vulnérable `4.0.0-4.3.0` (CVE-2026-59870), pinné exactement à `4.3.0` par `@redocly/openapi-core@1.34.18` (pin exact, pas un range). `4.3.1` publié depuis, hors de la plage vulnérable. Les autres consommateurs (`astro`, `@astrojs/internal-helpers`) déclarent `^4.3.0`, satisfait par `4.3.1`.

Un simple `npm update`/`npm install` ne suffisait pas à faire remonter ces versions (le lockfile figeait des résolutions exactes plus anciennes). Ajout d'`overrides` ciblés dans `frontend/package.json` (même pattern que les overrides existants `esbuild`/`vite`/etc., cf. #634) :

```json
"@babel/plugin-transform-modules-systemjs": "^7.29.7",
"js-yaml": "^4.3.1"
```

Lockfile régénéré via `npm install --package-lock-only --no-audit --no-fund`.

## Vérification

- `npm audit` (frontend) : **0 vulnérabilité** (avant : 3 high).
- Diff `package-lock.json` scopé : uniquement les 5 paquets babel-tooling (dev-only, patch bumps `7.29.0/7.28.6→7.29.7/7.29.8`) + `js-yaml` (`4.3.0→4.3.1`). Aucun paquet non lié modifié.
- `npm ci` réel (accès registry.npmjs.org disponible dans cette session, contrairement à github.com) : 785 paquets installés sans erreur.
- `npm run build` : 115 pages générées, aucune erreur.
- `npx prettier --check package.json package-lock.json` : propre.

## Ce qui reste ouvert

- **#696** — instabilité pré-existante de la suite smoke Playwright CI. Évalué non-actionnable dans ce round : l'issue elle-même dit servir uniquement à dater une occurrence, renvoie le fond du sujet à #550 (strate 3, déjà tracée), et porte une hypothèse (dégradation Vite dev sous charge soutenue) non confirmée sur plusieurs runs. Pas de fix « comme ça » sans comprendre la cause — laissé en l'état.

---
persona: astro-svelte-expert
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `astro-svelte-expert`

## Anti-patterns recurrents trouvés (audit 2026-04-29)

### 🔴 1. JWT en `localStorage` (XSS-exploitable)

- **Localisation** : `frontend/src/stores/auth.ts:16-18, 37`
- **Code actuel** :
  ```typescript
  const storedToken = localStorage.getItem("koprogo_token");
  localStorage.setItem("koprogo_token", token);
  ```
- **Cible** : Cookie httpOnly + SameSite=Strict + Secure côté backend, frontend ne voit jamais le JWT.
- **Migration** : nécessite changements backend (handler `/auth/login` pose le cookie) + frontend (retirer localStorage, vérifier session via `/auth/me`).

### 🔴 2. 88 pages Astro toutes en `client:load`

- **Conséquence** : SSG/SSR contournés, JS bundle énorme, perf dégradée.
- **Audit** : la plupart des composants pourraient passer en `client:idle` ou `client:visible`.
- **Stratégie** : audit case-by-case dans une issue dédiée.

### 🟠 3. Mix Svelte 4 / Svelte 5 (en cours de migration)

- **État actuel** : migration runes en cours (cf. commits récents `refactor(svelte5):*`).
- **Reliquat à terminer** : ~10 composants encore en transition partielle (NotificationBell, ResolutionVotePanel, etc.).
- **Anti-pattern récurrent corrigé** (commit `f89954e`) : commentaire `// Svelte 5 runes mode` placé HORS du `<script>` → crash silencieux.

### 🟠 4. God components

- `InvoiceWorkflow.svelte` 887 LOC → découper en 4 composants.
- `AdminGdprPanel.svelte` 660 LOC + `payload: any`.
- `InvoiceForm.svelte` 633 LOC.
- `UserForm.svelte` 591 LOC + `payload: any`.

### 🟠 5. 18 modals dupliqués

- Pattern : `UserCreateModal`, `OwnerCreateModal`, `UnitCreateModal`, `TicketCreateModal`...
- **Cible** : composant abstrait `<EntityCrudModal entity={...} mode="create|edit|delete" />`.
- **Impact** : élimine ~12 fichiers de code dupliqué.

### 🟠 6. 16 `fetch()` directs dans composants

- **Anti-pattern** : composant fait `fetch('/api/v1/...')` au lieu d'importer `lib/api.ts`.
- **Conséquence** : pas de gestion d'erreur centralisée, pas d'auth automatique, duplication.

### 🟠 7. `: any` / `as any` en TS strict

- **5 occurrences** trouvées : `BuildingForm.svelte`, `OrganizationForm.svelte`, `UserForm.svelte`, `AdminGdprPanel.svelte`, `api.ts`.
- **Cible** : types explicites ou `unknown` + validation runtime.

### 🟠 8. i18n strings hardcodées

- **23+ occurrences détectées** par scan.
- **Politique** : toute string user-facing doit utiliser `$t('clé')` + clé présente dans 4 locales (`frontend/src/locales/{fr,nl,en,de}.json`).

### 🟠 9. Tests Vitest = 13/181 composants ≈ 7 %

- **Composants critiques sans tests** : `InvoiceWorkflow`, `AdminGdprPanel`, `UserForm`.
- **Cible** : ≥ 30 % couverture composants en sprint S1, focus prioritaire sur composants critiques.

## Conventions Svelte 5 + Astro acceptées

- **Svelte 5 runes** : `$state`, `$derived`, `$effect`, `$props`, `$bindable` partout. Pas de mix avec legacy `let`/`$:`/`export let`.
- **Astro client directives** :
  - `client:idle` par défaut pour widgets non-critiques.
  - `client:visible` pour below-the-fold (lazy).
  - `client:load` réservé aux composants critiques au-dessus du fold.
  - Pas de `client:*` (statique) pour composants 100% server-rendered.
- **TypeScript strict** : pas de `: any`, pas de `as any`, props typées exhaustivement.
- **Sécurité** : pas de `{@html}` sauf cas justifié (et sanitizé).
- **A11y** : focus management dans modals, aria-*, keyboard navigation.
- **i18n** : 4 locales obligatoires, scan automatique des clés manquantes.

## Décisions en attente

- ADR : migration JWT vers cookie httpOnly (impact backend + frontend).
- ADR : composant `<EntityCrudModal>` abstrait (élimine ~12 modals).
- RFC : audit `client:load` → décision case-by-case `client:idle`/`visible`/`statique`.
- RFC : politique tests Vitest (cibles couverture par composant).

## Lessons learned

- La migration Svelte 5 runes est en cours mais incomplète et a généré des anti-patterns (commentaires hors `<script>` qui crashent silencieusement).
- L'audit a révélé que les composants admin sont les plus laxistes (`payload: any` toléré). C'est le périmètre prioritaire pour le strict TS.
- **Tooling via docker compose** (cf. memory `feedback_use-docker-compose-for-tooling.md`) : avant toute review/PR frontend, lancer `docker compose run --rm frontend npm run check` (astro check + svelte-check). Idem `npm run lint` et `npx playwright test`. Le shell hôte peut avoir node mais le container assure l'environnement reproductible.

## Liens

- [`.claude/agents/astro-svelte-expert.md`](astro-svelte-expert.md)
- Issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#427](https://github.com/gilmry/koprogo/issues/427), [#428](https://github.com/gilmry/koprogo/issues/428)
- Existing : `docs/MIGRATION_SVELTE5_RUNES.md`, `docs/ACCESSIBILITY_WCAG.rst`, `docs/I18N_GUIDE.md`, `docs/TESTING_SVELTE5.md`

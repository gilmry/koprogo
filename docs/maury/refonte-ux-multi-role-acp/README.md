# Refonte UX multi-rôle + modèle ACP — Pipeline Maury

> Refonte FE conséquente issue de la session live testing 2026-05-20 (cf. issues [#553](https://github.com/gilmry/koprogo/issues/553) / [#554](https://github.com/gilmry/koprogo/issues/554) / [#555](https://github.com/gilmry/koprogo/issues/555) et règles d'agent `admin-publishes-conform-buildings` / `validate-before-compute` / `world-model-seed`).
>
> Pilotée par la **Méthode Maury** (`Maury/README.md`) avec gates humains de signature à chaque phase.

## État du pipeline

| Phase | Document | Agent | Statut | GATE |
|---|---|---|---|---|
| **1 — Brief** | [`brief.md`](brief.md) | Mary (Analyste TOGAF) | ✅ Draft posé 2026-05-20 | ⏳ **Sign-off humain @gilmry attendu** |
| **2 — PRD** | [`prd.md`](prd.md) | John (PM) | ⛔ Blocked phase 1 | — |
| **3 — Architecture** | [`architecture.md`](architecture.md) | Winston (Architecte hexagonal) | ⛔ Blocked phase 2 | — |
| **4 — Stories** | [`stories.md`](stories.md) | Bob (Scrum Master) | ⛔ Blocked phase 3 | — |
| **5 — Validation** | _à créer_ | Product Owner | ⛔ Blocked phase 4 | — |
| **6 — Exécution** | (PRs + commits) | dev / qa / release-manager | ⛔ Blocked phase 5 | — |

## Aval — issues GitHub + WBS

Conformément à la décision humaine 2026-05-20 :

> *« quand ce sera validé, on créera les issues github qu'on intègrera dans le wbs golive 0.1.0 »*

Les artefacts suivants seront produits **après signature du brief** (Phase 1) puis itérativement à chaque gate :

- **Phase 2 (PRD signé)** → 1 issue Epic GitHub avec frontmatter Maury, pointant vers ce dossier
- **Phase 4 (Stories signées)** → N sous-issues GitHub (1 par story S1-S4 et plus si découpage), liées à l'Epic
- **Mise à jour WBS `docs/WBS_GO_LIVE_v0.1.0.md`** : intégration des stories priorisées comme bloqueurs go-live (notamment S1 refacto ACP si jugé bloquant légal). Cette intégration sera proposée en Phase 4 ou validation, pas avant.

Tant que ces gates ne sont pas franchis, **aucune issue GitHub n'est créée** et **le WBS n'est pas modifié**. La structure Maury vit indépendamment ici en `docs/maury/`.

## Mémoires d'agent applicables

Cette refonte respecte (sans dérogation) les mémoires transverses suivantes — toute story IA doit les charger :

- `project_admin-publishes-conform-buildings.md`
- `project_validate-before-compute.md`
- `project_world-model-seed.md`
- `project_a11y-wcag-aa-baseline.md`
- `project_data-testid-systematic.md`
- `project_fe-refactor-test-driven.md` (3 niveaux : caractérisation + RED-GREEN-BLUE + multi-rôle E2E)
- `feedback_multirole-narrative-scenarios.md`
- `project_no-f64-in-money.md`

## Conventions du dossier

- 1 fichier par phase Maury (snake_case)
- Frontmatter YAML obligatoire (cf. `brief.md` pour le pattern)
- Sections imposées par phase (cf. `Maury/Méthode Maury.md`)
- Statut explicite : `Draft awaiting human sign-off` → `Signed by @gilmry YYYY-MM-DD` → `Blocked next phase` → `Completed`
- Les liens vers les mémoires d'agent utilisent la syntaxe `[[memory-name]]` (compatible Obsidian-like)

# Refonte UX multi-rôle + modèle ACP — Pipeline Maury

> Refonte FE conséquente issue de la session live testing 2026-05-20 (cf. issues [#553](https://github.com/gilmry/koprogo/issues/553) / [#554](https://github.com/gilmry/koprogo/issues/554) / [#555](https://github.com/gilmry/koprogo/issues/555) et règles d'agent `admin-publishes-conform-buildings` / `validate-before-compute` / `world-model-seed`).
>
> Pilotée par la **Méthode Maury** (`Maury/README.md`) avec gates humains de signature à chaque phase.

## État du pipeline

| Phase | Document | Agent | Statut | GATE |
|---|---|---|---|---|
| **1 — Brief** | [`brief.md`](brief.md) | Mary (Analyste TOGAF) | ✅ **SIGNÉ par @gilmry 2026-05-20** (v1.0) | ✔ Franchi |
| **2 — PRD** | [`prd.md`](prd.md) | John (PM) | ✅ **SIGNÉ par @gilmry 2026-05-20** (v1.0) | ✔ Franchi |
| **3 — Architecture** | [`architecture.md`](architecture.md) | Winston (Architecte hexagonal) | ✅ **SIGNÉE par @gilmry 2026-05-20** (v1.0) | ✔ Franchi |
| **4 — Stories** | [`stories.md`](stories.md) | Bob (Scrum Master) | ✅ **SIGNÉES par @gilmry 2026-05-20** (v1.0 — 31 stories en 6 slices) | ✔ Franchi |
| **5 — Validation** | [`validation.md`](validation.md) | Product Owner (@gilmry) | ✅ **VALIDÉE par @gilmry 2026-05-20** (v1.0) | ✔ Franchi |
| **6 — Exécution** | (PRs + commits) | dev / qa / release-manager | 🟢 **Ready to start** — création issues GH + WBS Track H | ⏳ Démarrage à programmer |

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

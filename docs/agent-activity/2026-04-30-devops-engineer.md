---
date: 2026-04-30
persona: devops-engineer
session: PR-E (chore/ci-gitflow-alignment)
tier: 2  # proposal via PR ; merge = Tier 1 humain
---

# Activity log — devops-engineer 2026-04-30

## Context

Plan approuvé `C:\Users\gilmr\.claude\plans\toasty-cooking-snowflake.md` — Axe 5 « GitHub Actions GitFlow alignment ». Cette PR-E corrige les lacunes CI bloquant le pipeline GitOps multi-topologie :

1. `ci.yml` ne tournait jamais sur `dev`/`integration`/`staging`/`production` (typo `develop`).
2. `security.yml` idem.
3. `docker-build-push.yml` publiait `{branch}` mais les helm-values + `gitops-deploy.sh` consomment `{branch}-latest` + `{branch}-{sha7}`.
4. Aucun `release-tag.yml` pour automatiser les semver tags après merge `production → main`.
5. Aucun `GitHub Environment production` pour gate humain sur le build production.

## Actions (Tier 2)

| Action | Fichier | Lignes |
|---|---|---|
| Triggers GitFlow | `.github/workflows/ci.yml:3-22` | +18 -3 |
| Triggers GitFlow | `.github/workflows/security.yml:3-19` | +13 -2 |
| Tag scheme | `.github/workflows/docker-build-push.yml:46-78,109-140` | +30 -8 (2 jobs) |
| Nouveau workflow | `.github/workflows/release-tag.yml` | +84 (création) |
| Documentation | `docs/ci-cd/GITFLOW_CI.md` | +95 (création) |

## Décisions justifiées

### `release-tag.yml` en `workflow_dispatch` only

CRITICAL.md règle 2 : `gh release create` est *deny* en autonome. Le workflow tourne avec `GITHUB_TOKEN` (pas l'agent), mais pour rester safe :
- Trigger = `workflow_dispatch` uniquement (humain lance via `gh workflow run` ou UI Actions)
- Création de release en `--draft` (humain édite + publie manuellement)

Cohérent avec règle 11 Tier 1 (création doc publique = humain valide).

### Pas de `:latest` dans helm-values production/staging — tracé en PR-A

CRITICAL.md « Lignes rouges » : `:latest` un image tag = interdit. Mais les helm-values actuelles utilisent `production-latest`/`staging-latest` qui sont mutables. Tension noted, fix tracé en PR-A (cluster-profiles refactor) qui pinnera staging+production sur `{env}-{sha7}` ou digest.

PR-E publie les nouveaux tags (`{branch}-{sha7}`) **sans casser** les anciennes references — backward compat préservée.

### GitHub Environment `production` non créé par l'agent

Création d'un Environment GitHub = Tier 1 (config repo settings via UI ou `gh api PATCH`). Documenté dans `docs/ci-cd/GITFLOW_CI.md` avec procédure manuelle pour `@gilmry`.

### Branches `feature/**`, `story/**`, `chore/**` ajoutées au trigger ci.yml

Justification : la convention KoproGo utilise ces patterns. Avant cette PR, seules `claude/**` et `release/**` étaient triggerées. Toutes les branches de travail méritent CI verte avant merge dans `dev`.

## Risques identifiés

| Risque | Mitigation appliquée |
|---|---|
| `{{sha}}` token de metadata-action = full SHA (40 chars) ≠ `gitops-deploy.sh` qui utilise short=7 | Ajout step explicite `${GITHUB_SHA::7}` puis raw tag avec `steps.vars.outputs.short_sha` |
| `release-tag.yml` push tag puis trigger en cascade `docker-build-push.yml` (semver) | Comportement voulu : le tag `v0.X.Y` déclenche un build avec tags semver immutables |
| Documentation `GITFLOW_CI.md` non auto-générée | Acceptable — 1 fichier, low maintenance ; pas un risk de drift critique |
| Augmentation workload GH Actions (CI sur 4 nouvelles branches) | OK — KoproGo reste dans free tier ; mesurer après 1 mois sur Insights → Actions |

## Étapes suivantes (humain)

1. Review + merge PR-E vers `feature/dev`.
2. Configurer GitHub Environment `production` (procédure dans `docs/ci-cd/GITFLOW_CI.md`).
3. Ouvrir PR de suivi : ajouter `environment: production` au job docker-build-push.yml production-only.
4. Démarrer **PR-A (cluster-profiles)** en parallèle (peut commencer dès que PR-E est mergée).

## Métriques DORA (post-PR estimation)

- Lead time : aucun changement (CI ne ralentit pas la vélocité)
- Deploy frequency : +∞ → en pratique 1-2/jour selon promotions
- Change failure rate : devrait baisser (CI tournera enfin sur dev/integration avant promotion vers staging/production)
- MTTR : N/A (pas en prod)

## Sortie

PR-E ready pour review humaine. Aucune mutation prod. Aucun secret écrit. Hooks pre-commit appliqueront le format Yaml.

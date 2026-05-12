---
date: 2026-04-30
persona: support-agent
session: PR-D (docs/gitops-runbook)
tier: 2  # création doc référence ; merge = Tier 1 humain
---

# Activity log — support-agent 2026-04-30

## Context

Plan approuvé `C:\Users\gilmr\.claude\plans\toasty-cooking-snowflake.md` — finition runbook GitOps multi-topologie après merge PR-A (cluster-profiles), PR-C (env-dev poller) et PR-E (CI alignment, en review).

## Actions (Tier 2)

| Action | Fichier | Lignes |
|---|---|---|
| Création runbook | `infrastructure/GITOPS.md` | +185 |
| Pointer | `infrastructure/README.md` | +2 |

## Décisions

### Format unifié (5 cas d'usage)

`GITOPS.md` couvre les 5 topologies dans une matrice unique plutôt que 5 docs séparées :
1. env-dev local poller (Mode A)
2. VPS poller (Mode A)
3. Docker Desktop K8s sandbox (Mode B)
4. K3s self-hosted (Mode B)
5. K8s managé Koprogo Cloud (Mode B)

Justification : un opérateur ou self-hoster lit GITOPS.md une fois, identifie son cas, et exécute. Pas besoin de naviguer entre fichiers.

### Compat matrix + image tag scheme + Day1/Day2 dans le même doc

3 sections critiques co-localisées :
- **Compat matrix** par cluster profile (storage/ingress/TLS/secrets)
- **Image tag scheme** (cohérent `docker-build-push.yml` PR-E + helm-values + `gitops-deploy.sh`)
- **Day 1 / Day 2** (Terraform/Ansible vs Helm/ArgoCD)

→ une seule URL à donner aux nouveaux agents/opérateurs.

### Liens vers fichiers concrets, pas vers descriptions

Chaque section pointe vers les fichiers réels (cluster profiles, AppSet, scripts) plutôt que de réécrire leur contenu. Token-frugale, pas de drift entre runbook et code.

## Vérifications

- Liens internes vérifiés (cluster-profiles README, GITFLOW_CI.md, scripts/, helm/koprogo/)
- Cohérence avec PR #450/#451/#452 mergées
- Aucune mention de `--no-verify` ou `helm:latest` (CRITICAL.md respecté)

## Hors scope

- Diagrammes Mermaid (acceptable en V1, ASCII art lisible)
- Vidéos/screencasts (impossible à générer côté agent)
- ADR séparé pour le choix Mode A vs Mode B (déjà capturé dans `toasty-cooking-snowflake.md`)

## Étapes suivantes

1. Review + merge PR-D vers `feature/dev`.
2. Plan toasty-cooking-snowflake : 5/5 axes complétés (PR-E + PR-A + PR-B-attente + PR-C + PR-D).
3. **PR-B (bootstrap script + AppSet templatisée)** — encore à faire, dépend de PR-A mergée (✅).

## Sortie

PR-D ready pour review humaine. Aucune mutation prod. Doc exhaustive co-localisée avec l'infra.

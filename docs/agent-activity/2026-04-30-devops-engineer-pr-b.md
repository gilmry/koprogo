---
date: 2026-04-30
persona: devops-engineer
session: PR-B (chore/gitops-bootstrap-script)
tier: 2
---

# Activity log — devops-engineer 2026-04-30 (PR-B)

## Context

Plan toasty-cooking-snowflake — **Axes 2 & 3** : ApplicationSet templatisée + bootstrap script. Consomme PR-A (#451 mergé) cluster-profiles. Dernier axe restant du plan.

## Actions (Tier 2)

| Action | Fichier | Lignes |
|---|---|---|
| AppSet templatisée | `_shared/argocd/applicationset.yaml.tpl` | +148 |
| Bootstrap script | `_shared/scripts/gitops-bootstrap.sh` | +135 |
| Prereqs installer | `_shared/scripts/install-prereqs.sh` | +110 |
| Teardown helper | `_shared/scripts/gitops-teardown.sh` | +47 |
| Hosts file template | `_shared/scripts/hosts-local-gitops.txt` | +30 |
| Makefile targets | `Makefile.infra` | +14 |

## Décisions

### `envsubst '${CLUSTER_TYPE}'` (scoped)

Premier essai `envsubst < tpl` consommait `$values` (Helm multi-source ref) en plus de `${CLUSTER_TYPE}`. Fix : passer la liste explicite à substituer comme premier argument.
```bash
envsubst '${CLUSTER_TYPE}' < tpl   # ← seul ${CLUSTER_TYPE} remplacé, $values intact
```

Vérifié : rendu produit bien `$values/infrastructure/_shared/cluster-profiles/docker-desktop.yaml` (path Helm valide).

### Bootstrap **interactif** (Tier 1 safeguard)

Le script demande confirmation explicite avant tout `kubectl apply` :
```
Cluster context : docker-desktop
Cluster type    : docker-desktop
About to apply kubectl + helm changes to this cluster.
Type 'yes' to proceed: _
```

Cohérent avec CRITICAL.md règle 11 — même si l'humain lance le script, on ajoute un dernier garde-fou. Pas de `--yes` flag (volontairement absent — modifier le script si automation CI plus tard).

### `install-prereqs.sh` séparé du bootstrap

Pour permettre re-installation idempotente d'un seul composant (ex: `install-prereqs.sh k3s-self-hosted` re-installe juste sealed-secrets sans toucher ArgoCD existant). Aussi : facilite test unitaire du flow par profil.

### Teardown préserve les workloads

`gitops-down` supprime ArgoCD + ApplicationSets + AppProject mais **PAS** les namespaces `koprogo-{env}`. Re-bootstrap → ArgoCD re-sync les namespaces existants → aucune perte de data. Important pour le sandbox local : on peut couper/rebrancher la control plane sans rebuilder Postgres + MinIO + uploads.

### `applicationset.yaml` legacy gardé en parallèle

Le fichier original (sans cluster-profile stacking) reste dans `_shared/argocd/`. Le bootstrap applique `applicationset.yaml.tpl` (rendu) qui a les mêmes noms (`koprogo-infra`, `koprogo-app`) → écrase l'ancien à l'apply. Pas de breakage, transition progressive. Suppression de l'ancien dans une PR de suivi après validation.

### Cible Makefile `gitops-bootstrap CLUSTER=...`

Délégué au script bash plutôt que tout en Makefile (logique trop riche pour Makefile). Le pattern `make gitops-bootstrap CLUSTER=docker-desktop` reste idiomatique.

## Vérifications

- `bash -n` syntax check OK pour 3 scripts
- `CLUSTER_TYPE=docker-desktop envsubst '${CLUSTER_TYPE}' < tpl` produit YAML valide avec `$values` préservé
- Imports cluster-profile `docker-desktop` confirmé dans rendu (annotation, label, valueFiles)

## Risques

| Risque | Mitigation |
|---|---|
| `envsubst` non installé sur Mac (gettext) | `command -v envsubst` check au début du script |
| Run sur wrong kubectl context | Confirmation interactive affiche le context avant apply |
| ArgoCD UI ouvert sur réseau public via port-forward | Doc précise `localhost:8080` (binding 0.0.0.0 désactivé par défaut) |
| Legacy AppSet + nouveau template avec mêmes noms → race | apply atomique kubectl ; rollback via gitops-down |
| Self-hosted user oublie `KOPROGO_DOMAIN` | Profile k3s a `domainOverride: ""` → admin doit fournir via `-f override.yaml` (warn dans bootstrap) |

## Statut plan toasty-cooking-snowflake

Tous les 5 axes couverts :

| Axe | PR | Status |
|---|---|---|
| 1 — Cluster profiles | #451 | ✅ mergé |
| 2 — AppSet templatisée | **#(this PR)** | open |
| 3 — Bootstrap scripts + Makefile | **#(this PR)** | open |
| 4 — env-dev local poller | #452 | ✅ mergé |
| 5 — CI alignment | #450 | open |
| Doc | #457 (PR-D) | open |

Auxiliaires découverts :
- #454 openapi.json regen
- #456 EXP-003-complete (Decimal cascade)

## Sortie

PR-B ready pour review humaine. Aucune mutation prod (script Tier 1 — l'humain l'invoque). Cohabitation avec legacy AppSet préservée.

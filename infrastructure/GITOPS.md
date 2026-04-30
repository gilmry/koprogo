# KoproGo GitOps Runbook

> Pipeline GitOps multi-topologie : `feature/dev` (docker-compose `make dev`) → branches officielles **dev / integration / staging / production / main** suivies par GitOps déployé.

## Vue d'ensemble

```
feature/dev → dev → integration → staging → production → main
   docker      │       │             │           │
   compose     └───────┴─────────────┴───────────┴──→ trigger GitOps déployé
   (make dev)                                          (auto-suivi par les pollers)
```

**2 modes GitOps coexistent**, sélectionnés selon la **topologie cible** (pas la branche) :

| Mode | Topologies cibles | Mécanisme | Cas d'usage |
|---|---|---|---|
| **A — Cron poller** | docker-compose | [`gitops-deploy.sh`](_shared/scripts/gitops-deploy.sh) + systemd / Task Scheduler | (a) `env-dev` local supervisor (b) VPS dev/integration/staging/production |
| **B — ArgoCD** | Kubernetes | ArgoCD ApplicationSet + Helm/Kustomize | (a) Docker Desktop K8s sandbox (b) K3s self-hosted (c) K8s managé Koprogo Cloud |

## 5 cas d'usage

### Cas 1 — `env-dev` local poller (Mode A) — supervisor's machine

Topologie docker-compose tournant sur la machine du superviseur, suit la branche `dev`.

**Ports isolés de `make dev`** : 8082 / 5433 / 9002 / 9003 (vs 80 / 5432 / 9000 / 9001).

Setup :
```bash
cp infrastructure/monosite/local/env-dev/.env.example \
   infrastructure/monosite/local/env-dev/.env
# Éditer .env : rotate JWT_SECRET et TOTP_ENCRYPTION_KEY à des valeurs aléatoires

# /etc/hosts (Linux/Mac) ou C:\Windows\System32\drivers\etc\hosts (Windows)
echo "127.0.0.1 envdev.koprogo.local api-envdev.koprogo.local" | sudo tee -a /etc/hosts

# Linux/Mac (systemd user mode)
./infrastructure/_shared/scripts/install-systemd-poller.sh
systemctl --user enable --now koprogo-gitops-env-dev

# Windows (Task Scheduler)
.\infrastructure\_shared\scripts\windows-task-poller.ps1 -Action Install
Start-ScheduledTask -TaskName 'KoproGo-GitOps-EnvDev'
```

Vérifier :
```bash
journalctl --user -u koprogo-gitops-env-dev -f
curl http://envdev.koprogo.local:8082/health
```

→ détails [`monosite/local/env-dev/README.md`](monosite/local/env-dev/README.md).

### Cas 2 — VPS poller (Mode A) — dev / integration / staging / production

Topologie docker-compose sur VPS OVH provisionné par Terraform/Ansible.

```bash
# Sur le VPS, suite à provisioning Ansible (Day 1)
BRANCH=dev ENV_NAME=dev TOPOLOGY=vps \
  /opt/koprogo/infrastructure/_shared/scripts/gitops-deploy.sh watch
```

Le rôle Ansible installe le systemd unit `koprogo-gitops.service` (system-mode, run as root).

Layout attendu : `infrastructure/monosite/vps/${ENV_NAME}/{docker-compose.override.yml,.env}`.

### Cas 3 — Docker Desktop K8s sandbox (Mode B) — supervisor's machine

Reproduit en local le pipeline GitFlow K8s complet (4 envs en namespaces séparés sur le même cluster Docker Desktop) avant Terraform/Ansible vrai.

Setup (à venir dans **PR-B** après merge PR-A cluster-profiles) :
```bash
kubectl config use-context docker-desktop
make -C infrastructure gitops-bootstrap CLUSTER=docker-desktop
# → ingress-nginx + ArgoCD installés
# → ConfigMap argocd-cluster-config posé avec clusterType=docker-desktop
# → 4 ApplicationSets créées (koprogo-app-{dev,integration,staging,production})

# /etc/hosts pour les 4 envs
sudo cat infrastructure/_shared/scripts/hosts-local-gitops.txt >> /etc/hosts

# UI ArgoCD
make -C infrastructure gitops-ui   # → https://localhost:8080

# Vérifier
kubectl get applications -n argocd
```

Cluster profile [`_shared/cluster-profiles/docker-desktop.yaml`](_shared/cluster-profiles/docker-desktop.yaml) :
- Storage `hostpath` (Docker Desktop default)
- Ingress `nginx`
- TLS off (sandbox uniquement)
- Secrets `raw` (Helm values en clair — sandbox uniquement)

### Cas 4 — K3s self-hosted (Mode B) — communauté open-source

Pour un syndic ou self-hoster qui veut faire tourner Koprogo sur son propre k3s.

```bash
# Sur la machine cible (k3s installed)
curl -sfL https://get.k3s.io | sh -
KOPROGO_DOMAIN=ma-copro.be \
  ./infrastructure/_shared/scripts/gitops-bootstrap.sh k3s-self-hosted
```

Cluster profile [`_shared/cluster-profiles/k3s-self-hosted.yaml`](_shared/cluster-profiles/k3s-self-hosted.yaml) :
- Storage `local-path` (k3s default)
- Ingress `traefik` (k3s default)
- TLS `letsencrypt-prod` (cert-manager installé par bootstrap)
- Secrets `sealed-secrets` (controller installé par bootstrap)

Domain : l'admin fournit `KOPROGO_DOMAIN` au bootstrap. Le DNS doit pointer vers le node k3s.

### Cas 5 — K8s managé Koprogo Cloud (Mode B) — SaaS officiel

OVH Managed K8s + cert-manager + External-Secrets+Vault.

```bash
# Operator runs (after Terraform provisioning)
./infrastructure/_shared/scripts/gitops-bootstrap.sh k8s-managed
```

Cluster profile [`_shared/cluster-profiles/k8s-managed.yaml`](_shared/cluster-profiles/k8s-managed.yaml) :
- Storage `csi-cinder-high-speed` (OVH SSD)
- Ingress `nginx` (ingress-nginx-controller)
- TLS `letsencrypt-prod`
- Secrets `external-secrets-vault` (Vault Kubernetes auth method)
- Domain `koprogo.be`

## Image tag scheme

[`docker-build-push.yml`](../.github/workflows/docker-build-push.yml) (PR-E #450) publie pour chaque push :

| Tag | Mutabilité | Consommateur |
|---|---|---|
| `{branch}` | mutable | humain debug |
| `{branch}-latest` | mutable | ArgoCD helm-values (sandbox/dev/integration uniquement) |
| `{branch}-{sha7}` | **immutable** | `gitops-deploy.sh`, staging+production helm-values |
| `{sha7}`, `sha-{sha7}` | immutable | override manuel |
| `latest` | mutable | publié uniquement sur main (default branch) |
| `v{semver}`, `v{major}.{minor}` | immutable | release officielle (post-tag manuel via [release-tag.yml](../.github/workflows/release-tag.yml)) |

> **CRITICAL.md** : staging/production doivent **pinner sur `{branch}-{sha7}` ou digest** (`*-latest` interdit en prod).

## Compat matrix

| Profil | Storage | Ingress | TLS | Secrets backend | Domain | Resources preset |
|---|---|---|---|---|---|---|
| docker-desktop | hostpath | nginx | off | raw | `*.koprogo.local` | small |
| k3s-self-hosted | local-path | traefik | letsencrypt-prod | sealed-secrets | user-defined | medium |
| k8s-managed | csi-cinder | nginx | letsencrypt-prod | external-secrets-vault | `koprogo.be` | large |

## GitFlow CI workflows

Triggers post-PR-E (#450) :

| Workflow | feature/story/chore | dev | integration | staging | production | main | tag v*.*.* |
|---|---|---|---|---|---|---|---|
| [`ci.yml`](../.github/workflows/ci.yml) | push, PR | push, PR | push, PR | push, PR | push, PR | push, PR | — |
| [`security.yml`](../.github/workflows/security.yml) | — | push, PR | push, PR | push, PR | push, PR | push, PR | — |
| [`docker-build-push.yml`](../.github/workflows/docker-build-push.yml) | — | push, PR | push, PR | push, PR | push, PR | push | push (semver) |
| [`docs.yml`](../.github/workflows/docs.yml) | — | — | — | — | — | push | — |
| [`release-tag.yml`](../.github/workflows/release-tag.yml) | — | — | — | — | — | manual only | — |

Détails complets : [`docs/ci-cd/GITFLOW_CI.md`](../docs/ci-cd/GITFLOW_CI.md).

## Day 1 / Day 2 — découplage

```
DAY 1 — INFRA PROVISIONING (lent, rare)              DAY 2 — APP DEPLOYMENT (rapide, fréquent)
       Terraform + Ansible                                  GitOps (ArgoCD ou cron poller)
              │                                                       │
              ▼                                                       ▼
  ┌──────────────────────────┐                       ┌──────────────────────────┐
  │  Cluster vide + tagué    │ ──── lit le tag ────▶ │  Helm chart + cluster   │
  │  (clusterType ConfigMap) │                       │  profile + env values   │
  └──────────────────────────┘                       └──────────────────────────┘
```

Le **cluster profile** est le contrat entre les 2 couches. Day 1 le pose, Day 2 le lit. Aucun manifest applicatif ne sait sur quoi il tourne ; aucun module Terraform ne sait quelle version de l'app va tourner dessus.

→ détails [`_shared/cluster-profiles/README.md`](_shared/cluster-profiles/README.md).

## Promotion GitFlow — multi-topologie

Test d'acceptation : sur la machine supervisor avec **les 2 modes actifs en parallèle** (env-dev poller systemd + Docker Desktop K8s ArgoCD).

1. Merge `feature/dev → dev`, push.
2. Observer en parallèle :
   - `journalctl --user -u koprogo-gitops-env-dev -f` → poller détecte commit, `compose pull` + `up -d` (~3 min).
   - `kubectl get app koprogo-app-dev -n argocd -w` → ArgoCD `OutOfSync` → `Syncing` → `Synced` (~30s).
3. Tester les 2 endpoints :
   - `curl http://envdev.koprogo.local:8082/api/v1/health` (env-dev compose)
   - `curl -H "Host: dev.koprogo.local" http://localhost` (K8s dev namespace)
4. Si les 2 répondent avec le nouveau commit hash → **GitFlow multi-topologie validé**.

## Tier 1 / Tier 2 (CRITICAL.md règle 11)

| Action | Tier | Auteur |
|---|---|---|
| Édition cluster-profiles, Helm templates, scripts, docs | **Tier 2** | Agent IA via PR |
| `kubectl apply` / `helm install argocd` / `gitops-bootstrap.sh` | **Tier 1** | Humain (mainteneur ou ops) |
| Configuration GitHub Environment `production` | **Tier 1** | Humain admin repo |
| `gh release create` | **Tier 1** | Humain (workflow_dispatch + draft) |

L'agent ne lance **jamais** ces commandes — il livre les fichiers, l'humain bootstrappe.

## Troubleshooting

| Symptôme | Cause probable | Fix |
|---|---|---|
| Cron poller : `manifest unknown` | Tag `{branch}-{sha7}` pas encore publié (CI en cours) | poller retry 10× automatique, fallback `{branch}-latest` |
| Cron poller : ports collision avec `make dev` | Volumes/containers/ports non isolés | env-dev override utilise 8082/5433/9002, container names suffixés `-envdev` |
| ArgoCD : Application stuck `Progressing` | Image pull failed (private registry, auth) | `imagePullSecrets` dans helm-values + secret `regcred` dans namespace |
| ArgoCD : `OutOfSync` constant sur Helm release | `ignoreDifferences` mal configuré | voir AppSet template `ignoreDifferences` (déjà : Deployment.spec.replicas) |
| Hosts file résoud pas | Cache DNS local | `sudo systemd-resolve --flush-caches` ou redémarrer le navigateur |
| Docker Desktop K8s : pods `Pending` | Storage class `hostpath` pas dispo | `kubectl get sc` puis vérifier `is-default-class=true` sur hostpath |
| K3s self-host : cert-manager Order failed | DNS pas encore propagé pour `KOPROGO_DOMAIN` | attendre 5-10 min ou vérifier `dig +short {domain}` |

## Limites sandbox local

- Docker Desktop K8s : storage `hostpath` perd les data au reset Docker Desktop. Backup avant reset si nécessaire.
- 4 envs simultanés sur < 16 Go RAM = OOMKilled garanti. Scaler à 0 les envs non testés : `kubectl scale --replicas=0 deploy/koprogo-backend -n koprogo-{env}`.
- Pas de TLS local (cert-manager désactivé par profil docker-desktop). Pour le tester localement, utiliser `mkcert` (Phase 2).
- Pas de monitoring (Prometheus/Grafana). Pour observer, `kubectl top pods` + `kubectl logs`.

## Liens

- Plan complet : [`.claude/plans/toasty-cooking-snowflake.md`](../.claude/plans/toasty-cooking-snowflake.md) (5 axes)
- ApplicationSet ArgoCD : [`_shared/argocd/applicationset.yaml`](_shared/argocd/applicationset.yaml)
- Helm chart : [`_shared/helm/koprogo/`](_shared/helm/koprogo/)
- Cluster profiles : [`_shared/cluster-profiles/`](_shared/cluster-profiles/)
- Cron poller script : [`_shared/scripts/gitops-deploy.sh`](_shared/scripts/gitops-deploy.sh)
- CI workflows runbook : [`docs/ci-cd/GITFLOW_CI.md`](../docs/ci-cd/GITFLOW_CI.md)

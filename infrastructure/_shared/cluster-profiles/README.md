# Cluster Profiles

Helm values overlays describing **cluster-specific** runtime overrides (storage class, ingress class, TLS issuer, secrets backend, resources preset).

The profile is the contract between **Day 1 (provisioning)** and **Day 2 (deployment)** :
- Day 1 (Terraform/Ansible / `apt install k3s` / Docker Desktop K8s enable) installs a cluster and tags it with a type.
- Day 2 (ArgoCD ApplicationSet + Helm chart) reads the corresponding profile and injects the right values.

The profile contains **only** cluster-level concerns. Per-environment business config (replicas, log level, feature flags) lives in [`infrastructure/monosite/k3s/{env}/helm-values.yaml`](../../monosite/k3s/).

## Helm values stacking order

ArgoCD `koprogo-app` ApplicationSet feeds Helm in this order (later overrides earlier):

```
1. infrastructure/_shared/helm/koprogo/values.yaml         (chart defaults)
2. infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml  (cluster overrides) ← THIS DIR
3. infrastructure/monosite/k3s/${ENV}/helm-values.yaml     (env business config)
```

## Available profiles

| Profile | Storage | Ingress | TLS | Secrets backend | Use case |
|---|---|---|---|---|---|
| [`docker-desktop.yaml`](docker-desktop.yaml) | `hostpath` | `nginx` | off | `raw` (Helm values) | Local sandbox supervisor (Mac/Windows) |
| [`k3s-self-hosted.yaml`](k3s-self-hosted.yaml) | `local-path` | `traefik` | letsencrypt-prod | `sealed-secrets` | Community open-source AGPL deployments |
| [`k8s-managed.yaml`](k8s-managed.yaml) | `csi-cinder-high-speed` (OVH) | `nginx` | letsencrypt-prod | `external-secrets-vault` | Koprogo Cloud (managed K8s) |

## How a cluster picks its profile

The `CLUSTER_TYPE` is set at ArgoCD bootstrap time (PR-B), via `gitops-bootstrap.sh` which:
1. Auto-detects from `kubectl config current-context` (`docker-desktop` → docker-desktop, `*k3s*` → k3s-self-hosted, default → k8s-managed)
2. Can be overridden via CLI: `gitops-bootstrap.sh k3s-self-hosted`
3. Is persisted in a ConfigMap `argocd-cluster-config` in the `argocd` namespace

The ApplicationSet template (`applicationset.yaml.tpl`) substitutes `${CLUSTER_TYPE}` via `envsubst` at install time.

## Adding a new profile

1. Copy an existing profile as starting point.
2. Override only the cluster-specific keys (don't duplicate env-level values).
3. Add a row to the table above.
4. Add a case to `gitops-bootstrap.sh` auto-detection (PR-B) if applicable.
5. Test the rendered Helm output:
   ```bash
   helm template infrastructure/_shared/helm/koprogo \
     -f infrastructure/_shared/cluster-profiles/<NEW>.yaml \
     -f infrastructure/monosite/k3s/dev/helm-values.yaml
   ```

## What does NOT go into a profile

- **Per-environment values** (replicas dev=1 vs prod=3, log level, feature flags) → `monosite/k3s/{env}/helm-values.yaml`
- **Per-tenant values** (organization slug, custom domain) → tenant-specific Helm release values (out of scope for PR-A)
- **Secrets values** (passwords, API keys, JWT signing keys) → secrets backend (sealed-secrets / Vault), never in Git

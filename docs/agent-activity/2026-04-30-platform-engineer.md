---
date: 2026-04-30
persona: platform-engineer
session: PR-A (chore/gitops-cluster-profiles)
tier: 2  # proposal via PR ; merge = Tier 1 humain
---

# Activity log — platform-engineer 2026-04-30

## Context

Plan approuvé `C:\Users\gilmr\.claude\plans\toasty-cooking-snowflake.md` — Axe 1 « Cluster profiles ». Cette PR-A pose la première brique du GitOps cluster-agnostic : un dossier `cluster-profiles/` avec 3 overlays Helm (docker-desktop, k3s-self-hosted, k8s-managed) qui décrivent ce qui change *par type de cluster* (storage, ingress, TLS, secrets backend, resources preset), indépendamment de l'environnement GitFlow (dev/integration/staging/production).

## Actions (Tier 2)

| Action | Fichier | Lignes |
|---|---|---|
| Création dossier + README | `infrastructure/_shared/cluster-profiles/README.md` | +56 |
| Profil sandbox supervisor | `infrastructure/_shared/cluster-profiles/docker-desktop.yaml` | +37 |
| Profil community AGPL | `infrastructure/_shared/cluster-profiles/k3s-self-hosted.yaml` | +35 |
| Profil Koprogo Cloud | `infrastructure/_shared/cluster-profiles/k8s-managed.yaml` | +34 |
| Schema global.* | `infrastructure/_shared/helm/koprogo/values.yaml` | +21 -2 |
| Binding storageClass | `infrastructure/_shared/helm/koprogo/templates/postgres-statefulset.yaml` | +2 -1 |
| Binding storageClass | `infrastructure/_shared/helm/koprogo/templates/postgres-cluster.yaml` | +2 -1 |
| Binding storageClass | `infrastructure/_shared/helm/koprogo/templates/minio-deployment.yaml` | +2 -1 |

## Décisions justifiées

### Approche additive, backward-compat préservée

Le binding `coalesce` :
```
{{- $storageClass := .Values.global.storageClassName | default .Values.postgres.storageClass }}
```
préserve le comportement actuel : si on déploie *sans* profil (l'AppSet existante non templatisée), le rendu Helm est strictement identique à avant. Vérifié par `helm template` sans `-f cluster-profiles/...`.

→ Aucune migration ArgoCD prod requise. Le profil n'est appliqué que si la PR-B (bootstrap script + AppSet templatisée) ou un override manuel l'inclut.

### Champ `secretsBackend` advisory dans cette PR

PR-A pose juste le champ ; la logique conditionnelle (raw values vs SealedSecret vs ExternalSecret) sera implémentée en **PR de suivi** (probablement après PR-B). Aujourd'hui les `Secret` du chart sont écrits via `secrets.yaml` (raw values) — risk acceptable pour le moment puisque dev/integration uniquement. Production passera à external-secrets-vault avant Day 0 prod réel (CRITICAL.md règle 1).

### Pas de `templates/ingress.yaml` dans cette PR

Le plan listait la création d'un Ingress template Helm pour remplacer les patches kustomize per-env. À l'inspection ([`infrastructure/_shared/kustomize/base/ingress.yaml`](../../infrastructure/_shared/kustomize/base/ingress.yaml)), la base kustomize contient déjà :
- 3 Middlewares Traefik (security headers, HSTS, rate limit)
- TLS spec hardcodé pour `app.koprogo.be`/`api.koprogo.be`
- Annotations Traefik

Les remplacer par un template Helm cluster-agnostic (nginx + traefik + alb selon `global.ingressClassName`) demande de :
- Convertir les Middlewares Traefik en annotations équivalentes nginx-ingress (différent par contrôleur)
- Conditionner les blocs TLS sur `global.tls.enabled`
- Préserver les middlewares de sécurité

C'est une PR à part entière. **Tracé pour PR de suivi `chore/helm-ingress-unified`** après merge PR-A et PR-B.

### Profil k8s-managed assumé OVH/csi-cinder

CRITICAL.md règle 5 (itération sur les directives) — l'utilisateur a explicitement parlé de "VPS OVH" pour le provisioning Terraform. Le profil k8s-managed assume donc OVH Managed Kubernetes (csi-cinder-high-speed). Si Koprogo Cloud bouge sur AWS EKS / GKE / AKS, il suffira d'ajouter un profil `eks-managed.yaml` etc. — pas un breaking change.

## Vérification effectuée

Helm template rendering testé via `docker run --rm alpine/helm:3.16.2`:

```bash
# Cas 1: docker-desktop + dev
$ helm template ... -f cluster-profiles/docker-desktop.yaml -f monosite/k3s/dev/helm-values.yaml \
  | grep storageClassName
  storageClassName: hostpath          ✓
        storageClassName: hostpath    ✓

# Cas 2: k8s-managed + production
$ helm template ... -f cluster-profiles/k8s-managed.yaml -f monosite/k3s/production/helm-values.yaml \
  | grep storageClassName
  storageClassName: csi-cinder-high-speed   ✓

# Cas 3: backward-compat (no profile)
$ helm template ... -f monosite/k3s/dev/helm-values.yaml \
  | grep storageClassName
  (no output — fallback to component-level, same as before this PR)   ✓
```

## Risques identifiés

| Risque | Mitigation appliquée |
|---|---|
| Helm template casse pour utilisateurs avec `postgres.storageClass` set explicitement | Binding utilise `default` → component-level garde la priorité quand `global.storageClassName` est vide |
| Profils AGPL exposent une opinion forte (k3s = traefik + sealed-secrets) | Documenté en commentaires du fichier ; admin peut override via `-f override.yaml` |
| `global.domainOverride` non encore consommé par les templates | OK — sera utilisé par le futur Ingress template (PR follow-up) ; aujourd'hui c'est un champ documenté, no-op |
| `resources.preset` advisory uniquement | Acceptable Phase 1 ; si on veut rendre actif, créer un helper `_helpers.tpl` qui mappe preset → resources block |

## Étapes suivantes

1. Review + merge PR-A vers `feature/dev`.
2. **PR-B** (bootstrap script + AppSet templatisée) en suite immédiate — consomme les profils créés ici.
3. **PR follow-up `chore/helm-ingress-unified`** — convertir kustomize ingress + middlewares en template Helm conditionnel par `global.ingressClassName`.
4. **PR follow-up `chore/secretsBackend-conditional-rendering`** — implémenter le switch raw/SealedSecret/ExternalSecret dans `templates/secrets.yaml` selon `global.secretsBackend`.

## Sortie

PR-A ready pour review humaine. Aucune mutation prod. Backward-compat préservée (vérifié helm template).

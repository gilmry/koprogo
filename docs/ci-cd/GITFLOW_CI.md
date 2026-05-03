# GitFlow CI/CD — Workflows par branche

## Modèle de promotion

```
feature/* → story/* → chore/*  (PR review humaine)
                ↓
              dev  (auto-merge possible si CI verte)
                ↓
          integration  (PR review humaine)
                ↓
            staging  (PR review humaine + audit security)
                ↓
          production  (PR review humaine + GitHub Environment approval)
                ↓
              main  (merge depuis production uniquement)
                ↓
        workflow_dispatch release-tag.yml (humain manuel)
```

## Triggers par workflow

| Workflow | feature/story/chore | dev | integration | staging | production | main | tag v*.*.* |
|---|---|---|---|---|---|---|---|
| [`ci.yml`](../../.github/workflows/ci.yml) — lint + tests | push, PR | push, PR | push, PR | push, PR | push, PR | push, PR | — |
| [`security.yml`](../../.github/workflows/security.yml) — cargo audit | — | push, PR | push, PR | push, PR | push, PR | push, PR | — |
| [`docker-build-push.yml`](../../.github/workflows/docker-build-push.yml) — GHCR images | — | push, PR | push, PR | push, PR | push, PR | push | push (semver) |
| [`docs.yml`](../../.github/workflows/docs.yml) — GitHub Pages | — | — | — | — | — | push | — |
| [`release-tag.yml`](../../.github/workflows/release-tag.yml) — semver release | — | — | — | — | — | manual only | — |

## Image tag scheme (GHCR)

Pour chaque push sur les branches officielles, `docker-build-push.yml` publie :

| Tag | Mutabilité | Consommateur | Exemple |
|---|---|---|---|
| `{branch}` | mutable | humain (debug, tests manuels) | `dev`, `integration`, `staging`, `production`, `main` |
| `{branch}-latest` | mutable | ArgoCD `helm-values.yaml` (sandbox/dev/integration uniquement) | `dev-latest` |
| `{branch}-{sha7}` | **immutable** | `gitops-deploy.sh` (VPS/local poller), staging+production helm-values (cible PR-A) | `dev-1de6c2e` |
| `{sha7}` | immutable | override manuel `helm install --set image.tag=...` | `1de6c2e` |
| `sha-{sha7}` | immutable | rétrocompat | `sha-1de6c2e` |
| `latest` | mutable | **uniquement publié sur main** (default branch) | `latest` |
| `v{semver}` + `v{major}.{minor}` | immutable | release officielle, post-tag manuel | `v0.2.0`, `v0.2` |

> **CRITICAL.md (lignes rouges)** : `:latest` est un tag mutable interdit en prod. Production et staging doivent **pinner sur `{branch}-{sha7}`** ou un digest. Cette migration est tracée dans **PR-A (cluster-profiles)** — la PR-E (cette PR CI) se contente d'aligner les tags publiés avec ce que les helm-values attendent aujourd'hui.

## Setup GitHub Environment `production` (Tier 1, manuel humain)

> Cette étape **ne peut PAS être automatisée** par un agent IA — elle nécessite un humain avec droits admin sur le repo (CRITICAL.md règle 11).

### Procédure (à faire 1 fois par le mainteneur)

1. Aller sur https://github.com/gilmry/koprogo/settings/environments
2. Cliquer **New environment** → nom : `production`
3. Cocher **Required reviewers** → ajouter `@gilmry` (au minimum)
4. **Wait timer** (optionnel) : 0 minutes (l'approval suffit)
5. **Deployment branches and tags** : restreindre à `production` et `main` uniquement
6. Sauver

### Activer le gate sur `docker-build-push.yml` job production

Une fois l'environment créé, ajouter à `docker-build-push.yml` (PR séparée pour limiter le blast radius) :

```yaml
build-and-push-backend:
  runs-on: ubuntu-latest
  environment: ${{ github.ref_name == 'production' && 'production' || '' }}
  # ...
```

Cela bloque le build de l'image production tant que `@gilmry` n'a pas approuvé le déploiement dans l'UI GitHub Actions.

> **Pourquoi pas dans cette PR ?** Le gate sans Environment configuré côté repo settings ferait échouer le workflow. Donc : (1) cette PR aligne les triggers + tags, (2) le mainteneur configure l'Environment via UI, (3) une PR de suivi ajoute le `environment:` au YAML.

## Cohérence avec ArgoCD / GitOps

- **ArgoCD ApplicationSet** suit les branches `dev`, `integration`, `staging`, `production` ([`infrastructure/_shared/argocd/applicationset.yaml`](../../infrastructure/_shared/argocd/applicationset.yaml)).
- Les helm-values ([`infrastructure/monosite/k3s/{env}/helm-values.yaml`](../../infrastructure/monosite/k3s/)) référencent `tag: {env}-latest` → matchent les tags publiés par `docker-build-push.yml`.
- Le poller VPS / local env-dev (`gitops-deploy.sh`) construit `IMAGE_TAG="${BRANCH}-${current_sha}"` → matche les tags `{branch}-{sha7}` publiés.

## Vérification post-merge PR-E

Après merge de cette PR vers `feature/dev` puis vers `dev` :

1. **CI doit tourner sur `dev`** :
   ```bash
   gh run list --branch dev --workflow ci.yml --limit 3
   # → Workflow runs présents (auparavant : 0 run, jamais déclenché)
   ```

2. **Image `ghcr.io/gilmry/koprogo/backend:dev-latest` doit exister** :
   ```bash
   gh api /users/gilmry/packages/container/koprogo%2Fbackend/versions \
     --jq '.[] | select(.metadata.container.tags[] | contains("dev-latest"))' | head
   ```

3. **Image `dev-{sha7}` doit exister pour chaque commit** :
   ```bash
   sha=$(git rev-parse --short=7 origin/dev)
   docker manifest inspect ghcr.io/gilmry/koprogo/backend:dev-${sha}
   ```

4. **Security audit doit tourner sur dev** :
   ```bash
   gh run list --branch dev --workflow security.yml --limit 3
   ```

## Checklist humain post-merge

- [ ] Vérifier que la 1ère push sur `dev` post-merge déclenche `ci.yml` (auparavant ne se déclenchait pas)
- [ ] Vérifier que `docker-build-push.yml` produit les tags `dev-latest` et `dev-{sha7}`
- [ ] Configurer GitHub Environment `production` (procédure ci-dessus)
- [ ] Ouvrir une PR de suivi pour ajouter `environment: production` au job production de `docker-build-push.yml`
- [ ] Mettre à jour [`infrastructure/monosite/k3s/{staging,production}/helm-values.yaml`](../../infrastructure/monosite/k3s/) pour pinner sur `{env}-{sha}` au lieu de `{env}-latest` (PR-A cluster-profiles)

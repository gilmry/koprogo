=============================================================================================================
Issue #515: infra: ArgoCD GitOps fresh-cluster deployment fails on 5 gaps (dry-run Docker Desktop 2026-05-12)
=============================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,phase:k3s track:infrastructure,priority:high
:Assignees: Unassigned
:Created: 2026-05-12
:Updated: 2026-05-12
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/515>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue GH — draft
   
   **Titre proposé** :
   `infra: ArgoCD GitOps fresh-cluster deployment fails on 5 gaps (dry-run Docker Desktop 2026-05-12)`
   
   **Labels suggérés** : `infrastructure`, `gitops`, `bug`, `priority:high`
   
   **Milestone** : la jalon sécurité/infra en cours (Jalon 1 ou #429 runtime ops)
   
   ---
   
   ## Contexte
   
   Dry-run du déploiement prod (`monosite/k3s/production` × cluster-profile `docker-desktop`) sur un cluster Docker Desktop K8s vierge, suivant le runbook GitOps officiel :
   
   1. `kubectl create namespace argocd` + `helm install argocd argo/argo-cd` ✅
   2. `kubectl apply` AppProject + ApplicationSet (rendus avec `CLUSTER_TYPE=docker-desktop`) ✅
   3. ArgoCD pull `github.com/gilmry/koprogo.git` branches `production` (helm) + `infra-prod` (kustomize) ✅
   4. Sync échoue partiellement → **état hybride** : postgres OK, reste cassé
   
   Résultat `kubectl get applications -n argocd` :
   ```
   koprogo-app-production      Synced       Progressing  ← partiel
   koprogo-infra-production    OutOfSync    Missing      ← cassé
   ```
   (idem dev / integration / staging)
   
   ---
   
   ## Gap 1 — Traefik CRDs absents au moment du sync infra
   
   **Constat**
   ```
   SyncFailed traefik.containo.us/Middleware/koprogo-rate-limit
   SyncFailed traefik.containo.us/Middleware/koprogo-hsts
   SyncFailed traefik.containo.us/Middleware/koprogo-security-headers
   Error: "Middleware" CRD is installed on the destination cluster.
   ```
   
   **Cause**
   Le cluster-profile [`_shared/cluster-profiles/docker-desktop.yaml`](https://github.com/gilmry/koprogo/blob/production/infrastructure/_shared/cluster-profiles/docker-desktop.yaml) déclare `ingressClassName: traefik` et commente *"Traefik installed via gitops-bootstrap.sh"*. Mais le script `gitops-bootstrap.sh` est un step manuel **non orchestré par l'ApplicationSet**. Sur un cluster qui suit uniquement le chemin `kubectl apply ApplicationSet`, Traefik n'est jamais installé, ses CRDs non plus, et la kustomize base échoue immédiatement.
   
   **Recette**
   - Option A : Wrapper le bootstrap dans un ApplicationSet "prerequisites" déployé en premier (sync wave -1), avec Traefik Helm chart + sa CRD bundle.
   - Option B : Documenter explicitement le step `gitops-bootstrap.sh` comme prérequis obligatoire avant `kubectl apply applicationset.yaml`, et faire échouer fast si CRDs absents.
   - Option C : App-of-Apps pattern — un Application racine déploie Traefik (Application 1, wave 0), puis les autres (wave 1+).
   
   **Critères d'acceptation**
   - [ ] Sur un cluster vierge, un seul `kubectl apply` de l'ApplicationSet bootstrap-anything-else doit suffire OU le runbook documente explicitement les prérequis avec test de présence des CRDs.
   - [ ] CI ajoute un test "fresh cluster from scratch" (kind + ApplicationSet only) qui doit converger sans intervention manuelle.
   
   ---
   
   ## Gap 2 — API group Traefik obsolète
   
   **Constat**
   Les manifests `_shared/kustomize/base` utilisent `traefik.containo.us/v1alpha1` (Traefik 2.x, déprécié 2023).
   
   **Cause**
   Pas migré vers Traefik 3.x. `traefik.io/v1alpha1` est le nouveau group depuis Traefik 3.0 (Apr 2024).
   
   **Recette**
   - Migrer tous les `traefik.containo.us` → `traefik.io` dans `_shared/kustomize/base/*.yaml`.
   - Vérifier dans `gitops-bootstrap.sh` quelle version de Traefik chart est installée et aligner.
   - Tester avec kubeconform contre les schémas Traefik 3.x dans `ci-infra.yml`.
   
   **Critères d'acceptation**
   - [ ] 0 occurrence `traefik.containo.us` dans `infrastructure/`.
   - [ ] kubeconform passe avec schemas Traefik 3.x.
   - [ ] Documentation `SECURITY.md` mentionne version Traefik supportée.
   
   ---
   
   ## Gap 3 — Dépendance croisée koprogo-infra → koprogo-app non orchestrée
   
   **Constat**
   ```
   FailedCreate pods "koprogo-backend-XXX" forbidden: serviceaccount "koprogo-backend" not found
   FailedCreate pods "koprogo-frontend-XXX" forbidden: serviceaccount "koprogo-frontend" not found
   ```
   Les ServiceAccounts `koprogo-backend` / `koprogo-frontend` sont créés par le layer **kustomize** (ApplicationSet `koprogo-infra`). Les Deployments backend/frontend sont créés par le layer **helm** (ApplicationSet `koprogo-app`). Rien n'attend que infra ait convergé avant que app démarre ses pods.
   
   **Cause**
   2 ApplicationSets séparés sans Sync Wave ni hook entre eux. Lorsque infra échoue (Gap 1), app reste bloqué sans signaler clairement la dépendance.
   
   **Recette**
   - Soit : annoter les Applications du generator `koprogo-app` avec `argocd.argoproj.io/sync-wave: "1"` et celles d'infra avec `"0"`, et utiliser un Sync Wave global.
   - Soit : déplacer la création des ServiceAccounts dans la chart Helm `koprogo` (avec `serviceAccount.create: true` standard pattern), supprimer cette responsabilité du layer kustomize.
   - Soit : App-of-Apps avec syncWave entre apps racines.
   
   **Critères d'acceptation**
   - [ ] Sur un cluster vierge, app ne tente pas de Deployer ses pods tant que ses dépendances (SA, ConfigMaps, Secrets) ne sont pas Synced.
   - [ ] Le manifest helm chart déclare explicitement ses dépendances RBAC.
   
   ---
   
   ## Gap 4 — MinIO entrypoint cassé
   
   **Constat**
   ```
   pod/koprogo-minio-XXX  RunContainerError  BackOff
   Error: failed to create containerd task: ...
   unable to start container process: exec: "server": executable file not found in $PATH
   ```
   
   **Cause**
   La chart Helm `_shared/helm/koprogo/templates/minio-deployment.yaml` passe `command: [server, /data]` (ou similaire). L'image `minio/minio:latest` a vraisemblablement changé d'entrypoint depuis la rédaction de la chart, et `server` n'est plus un binaire dans `$PATH` sans le wrapper officiel.
   
   **Cause secondaire (ligne rouge CRITICAL.md)**
   Tag `minio/minio:latest` mutable — interdit par les lignes rouges du projet ("Ne jamais `:latest` un image tag (digest only)"). Drift garanti à chaque pull.
   
   **Recette**
   - Pinner MinIO sur un tag stable + digest (ex. `minio/minio:RELEASE.2024-XX-XX-Z@sha256:...`).
   - Inspecter l'image cible : `docker inspect minio/minio:<tag> | jq '.[].Config.Entrypoint'` pour savoir si on doit utiliser `command:` ou `args:`.
   - Adapter `minio-deployment.yaml` selon le pattern officiel MinIO actuel.
   
   **Critères d'acceptation**
   - [ ] 0 occurrence `:latest` dans `_shared/helm/` et `monosite/`.
   - [ ] Image MinIO pinnée par digest dans `values.yaml` + commenté avec la date du pinning.
   - [ ] Pod MinIO démarre `1/1 Running` sur le dry-run Docker Desktop.
   
   ---
   
   ## Gap 5 — Aucun test "fresh cluster" en CI
   
   **Constat**
   La CI `ci-infra.yml` valide `kubectl kustomize`, `helm template`, `kubeconform`. Aucun de ces tests n'aurait détecté les Gaps 1-4 — ils valident le **rendu**, pas le **déploiement réel**.
   
   **Cause**
   Pas de job CI qui spin up un kind/k3d, applique l'ApplicationSet, attend la convergence, et fail si timeout.
   
   **Recette**
   - Ajouter un job `ci-infra-deploy` (matrix env=dev, cluster=kind) :
     1. `kind create cluster --config=...`
     2. `helm install argocd ...`
     3. `kubectl apply` les rendered ApplicationSets pour `dev` only
     4. `kubectl wait --for=condition=Healthy applications/koprogo-app-dev --timeout=10m`
     5. `kubectl wait --for=condition=Synced applications/koprogo-infra-dev --timeout=10m`
   - Marquer optionnel/non-bloquant au début pour ne pas bloquer la chaîne pendant la stabilisation.
   
   **Critères d'acceptation**
   - [ ] Job CI green sur cluster vierge dev → tous les pods Running.
   - [ ] Job documenté dans `README.md` infra section CI.
   
   ---
   
   ## Annexes
   
   - **Rendered manifests utilisés** : `c:\Users\gilmr\koprogo-local-deploy\rendered\` (rendus avec `CLUSTER_TYPE=docker-desktop`)
   - **Branche source** : `gilmry/koprogo@production` (commit `5aab471`) + `gilmry/koprogo@infra-prod` (commit `67d8e547`)
   - **Cluster cible** : Docker Desktop K8s v1.34, single-node
   - **Date du test** : 2026-05-12 23:26 → 23:31 CEST
   - **Logs détaillés disponibles** sur demande

.. raw:: html

   </div>


============================================================
Issue #268: feat(infra): GitOps pipeline avec ArgoCD sur K3s
============================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: phase:k3s,track:infrastructure priority:low
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/268>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Avec le cluster K3s provisionné (#266 Terraform, #267 Ansible), il faut un pipeline GitOps pour déployer et mettre à jour l'application KoproGo de manière déclarative et auditée.
   
   ArgoCD est le standard GitOps pour Kubernetes — léger, adapté à K3s, avec UI web et CLI.
   
   ## Objectifs
   
   Mettre en place un pipeline GitOps complet avec ArgoCD : le cluster converge automatiquement vers l'état décrit dans Git.
   
   ## Scope
   
   ### 1. Installation ArgoCD
   
   - [ ] Namespace `argocd`
   - [ ] Installation via Helm chart officiel (`argo/argo-cd`)
   - [ ] Configuration HA légère (1 replica suffisant pour K3s < 500 copros)
   - [ ] Ingress ArgoCD UI : `argocd.koprogo.be` (TLS via cert-manager)
   - [ ] CLI `argocd` installé sur le master
   - [ ] Admin password initial via Sealed Secret
   
   ### 2. Repository structure GitOps
   
   - [ ] Créer `infrastructure/k8s/` (manifests Kubernetes déclaratifs)
   - [ ] Structure par environnement :
     ```
     infrastructure/k8s/
     ├── base/                    # Manifests communs (Kustomize base)
     │   ├── kustomization.yaml
     │   ├── namespace.yaml
     │   ├── backend/
     │   │   ├── deployment.yaml
     │   │   ├── service.yaml
     │   │   ├── hpa.yaml         # HorizontalPodAutoscaler
     │   │   └── ingress.yaml
     │   ├── frontend/
     │   │   ├── deployment.yaml
     │   │   ├── service.yaml
     │   │   └── ingress.yaml
     │   ├── postgresql/
     │   │   ├── statefulset.yaml
     │   │   ├── service.yaml
     │   │   ├── pvc.yaml
     │   │   └── configmap.yaml
     │   └── monitoring/
     │       └── kustomization.yaml
     ├── overlays/
     │   ├── staging/
     │   │   ├── kustomization.yaml  # Patches staging (1 replica, debug logs)
     │   │   └── configmap-patch.yaml
     │   └── production/
     │       ├── kustomization.yaml  # Patches prod (2+ replicas, info logs)
     │       ├── configmap-patch.yaml
     │       └── sealed-secrets/     # Secrets chiffrés
     └── argocd/
         ├── projects/
         │   └── koprogo.yaml        # AppProject
         └── apps/
             ├── backend.yaml        # Application ArgoCD
             ├── frontend.yaml
             ├── postgresql.yaml
             └── monitoring.yaml
     ```
   
   ### 3. ArgoCD Applications
   
   - [ ] **AppProject** `koprogo` : restrictions RBAC (namespaces autorisés, repos autorisés)
   - [ ] **App `backend`** : Déploiement Actix-web (image Docker, 2 replicas prod, HPA 2-5)
   - [ ] **App `frontend`** : Déploiement Astro static (nginx, 1-2 replicas)
   - [ ] **App `postgresql`** : StatefulSet PostgreSQL (ou référence OVH managed DB)
   - [ ] **App `monitoring`** : kube-prometheus-stack
   - [ ] Sync policy : `automated` avec `selfHeal: true` et `prune: true`
   - [ ] Sync waves : PostgreSQL (wave 0) → Backend (wave 1) → Frontend (wave 2)
   
   ### 4. CI/CD Integration (GitHub Actions → ArgoCD)
   
   - [ ] GitHub Actions : build Docker image → push vers registry (GitHub Container Registry ou OVH Harbor)
   - [ ] Tagging : `ghcr.io/gilmry/koprogo-backend:<git-sha>` et `:latest`
   - [ ] Mise à jour automatique du tag image dans les overlays (Kustomize `images` ou ArgoCD Image Updater)
   - [ ] Workflow :
     1. Push sur `main` → CI tests + build image
     2. Image publiée → ArgoCD Image Updater détecte le nouveau tag
     3. ArgoCD sync automatique → rolling update zero-downtime
   - [ ] OU : CI met à jour le fichier kustomization.yaml et commit (GitOps pur)
   
   ### 5. Secrets Management
   
   - [ ] Sealed Secrets controller installé sur le cluster
   - [ ] `kubeseal` pour chiffrer les secrets avant commit
   - [ ] Secrets à gérer :
     - `DATABASE_URL` (PostgreSQL connection string)
     - `JWT_SECRET` (min 32 chars, cf. validation existante)
     - `STRIPE_SECRET_KEY` (paiements)
     - Credentials SMTP (notifications email)
     - OVH API keys (DNS, backup S3)
   - [ ] Rotation des secrets documentée
   
   ### 6. Environnements
   
   - [ ] **Staging** (`koprogo-staging`) :
     - 1 replica backend, 1 frontend
     - PostgreSQL dédié (ou schéma séparé)
     - Sync automatique sur push `develop`/`staging`
     - Debug logs activés
   - [ ] **Production** (`koprogo-prod`) :
     - 2+ replicas backend (HPA), 1-2 frontend
     - PostgreSQL avec backups
     - Sync automatique sur push `main` (après CI vert)
     - Approval manuel optionnel (ArgoCD sync window)
   
   ### 7. Observabilité GitOps
   
   - [ ] Notifications ArgoCD → Slack/Discord (sync success/failure)
   - [ ] ArgoCD metrics → Prometheus (sync duration, app health)
   - [ ] Dashboard Grafana "GitOps" (deploy frequency, lead time, MTTR)
   - [ ] Alertes : sync failed, app degraded, image pull errors
   
   ### 8. Rollback & Disaster Recovery
   
   - [ ] Rollback via `argocd app rollback` ou git revert
   - [ ] Historique des syncs dans ArgoCD UI
   - [ ] Procédure documentée de DR :
     1. `terraform apply` (recréer infra)
     2. `ansible-playbook site.yml` (provisionner cluster)
     3. ArgoCD auto-sync depuis Git (restaurer apps)
     4. PostgreSQL restore depuis backup S3
   
   ## Dépendances
   
   - #266 (Terraform — cluster OVH)
   - #267 (Ansible — K3s installé + cert-manager + Longhorn)
   - GitHub Actions CI existant (à étendre)
   - Container Registry (GHCR ou OVH)
   
   ## Estimation
   
   8-10h
   
   ## Critères d'acceptation
   
   - [ ] ArgoCD UI accessible sur `argocd.koprogo.be` (TLS)
   - [ ] Push sur `main` déclenche un déploiement automatique (< 5 min)
   - [ ] `argocd app list` montre toutes les apps `Healthy` + `Synced`
   - [ ] Rollback fonctionnel via git revert → ArgoCD auto-sync
   - [ ] Secrets chiffrés dans Git (aucun secret en clair)
   - [ ] Staging et Production déployés et fonctionnels
   - [ ] Zero-downtime rolling updates vérifiés
   - [ ] Documentation opérationnelle complète (runbook)
   
   ## Références
   
   - [ArgoCD Getting Started](https://argo-cd.readthedocs.io/en/stable/getting_started/)
   - [ArgoCD Image Updater](https://argocd-image-updater.readthedocs.io/)
   - [Sealed Secrets](https://sealed-secrets.netlify.app/)
   - [Kustomize](https://kustomize.io/)
   
   ---
   
   ## Considérations Green IT (Mars 2026)
   
   > **Priorité abaissée à low** — le scale-up VPS est plus écologique à court/moyen terme.
   
   ### Contexte performance actuelle
   - Backend : **5.5 MB** RAM, **287 req/s**, **0.12g CO2/req**
   - Capacité VPS actuel : **2,000-3,000 copros** (8€/mois)
   - Scale-up VPS d2-8 : **10,000+ copros** (~32€/mois, ~40W)
   
   ### Impact ArgoCD sur l'empreinte
   ArgoCD seul ajoute un overhead permanent :
   - **~300 MB RAM** (server + repo-server + application-controller)
   - **CPU polling** continu (sync toutes les 3 min par défaut)
   - Justifié uniquement si le bénéfice opérationnel (GitOps, rollback, multi-env) compense le coût énergétique
   
   ### Alternative légère pour le VPS
   En attendant K3s, un GitOps simplifié sur VPS peut couvrir 80% des besoins :
   - GitHub Actions → SSH deploy (script Ansible existant)
   - Git tag → Docker pull → docker compose up
   - Pas d'overhead d'orchestrateur
   
   ### Seuil de bascule recommandé
   ArgoCD + GitOps complet devient pertinent quand :
   - **Cluster K3s déployé** (#266, #267)
   - **Multiple environnements** (staging + prod) à gérer
   - **Équipe > 1 dev** avec besoin d'audit trail des déploiements
   - **Rollback fréquents** nécessaires (> 1/mois)

.. raw:: html

   </div>


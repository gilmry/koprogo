=====================================================================================
Issue #266: feat(infra): Provisionning cluster K3s sur OVH avec Terraform + OpenStack
=====================================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: phase:k3s,track:infrastructure priority:low
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/266>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Migration VPS → K3s pour supporter 200-500 copropriétés (cf. #231).
   OVH Public Cloud utilise OpenStack comme couche IaaS — Terraform dispose d'un provider OpenStack mature.
   
   ## Objectifs
   
   Créer l'infrastructure K3s sur OVH Public Cloud via Terraform, reproductible et versionnable (IaC).
   
   ## Scope
   
   ### 1. Setup Terraform + Backend
   
   - [ ] Initialiser le projet Terraform dans `infrastructure/terraform/`
   - [ ] Configurer le backend remote (Swift object storage OVH ou S3-compatible)
   - [ ] Configurer le provider OpenStack pour OVH Public Cloud
   - [ ] Variables d'environnement OVH : `OS_AUTH_URL`, `OS_TENANT_ID`, `OS_REGION_NAME`, etc.
   - [ ] Fichier `terraform.tfvars.example` avec documentation
   
   ### 2. Réseau (OpenStack Neutron)
   
   - [ ] Création du réseau privé (`koprogo-net`)
   - [ ] Sous-réseau avec DHCP (`10.0.0.0/24`)
   - [ ] Routeur connecté au réseau externe (Ext-Net OVH)
   - [ ] Security groups :
     - `k3s-master` : 6443 (API), 2379-2380 (etcd), 10250 (kubelet)
     - `k3s-worker` : 10250, 30000-32767 (NodePort)
     - `common` : 22 (SSH), 8472/UDP (VXLAN/Flannel), ICMP
   - [ ] Floating IP pour le master (accès API externe)
   
   ### 3. Compute (OpenStack Nova)
   
   - [ ] Instance master K3s : `b2-7` (2 vCPU, 7GB RAM) — suffisant pour < 500 copros
   - [ ] Instances workers K3s (x2) : `b2-7` chacune
   - [ ] Image : Ubuntu 22.04 LTS (OVH marketplace)
   - [ ] SSH keypair provisionnée via Terraform
   - [ ] Cloud-init minimal (user creation, SSH hardening, packages de base)
   - [ ] Anti-affinity server group (workers sur hyperviseurs différents)
   
   ### 4. Storage (OpenStack Cinder)
   
   - [ ] Volume block storage pour PostgreSQL data (`50GB`, SSD haute performance)
   - [ ] Volume pour Longhorn distributed storage (workers)
   - [ ] Backup policy automatique (snapshots Cinder)
   
   ### 5. DNS (OVH API)
   
   - [ ] Enregistrement DNS A pour `api.koprogo.be` → floating IP master
   - [ ] Enregistrement wildcard `*.koprogo.be` → ingress IP
   - [ ] Module Terraform OVH DNS (provider `ovh`)
   
   ### 6. Outputs & Documentation
   
   - [ ] Outputs : IPs, kubeconfig path, SSH commands
   - [ ] `README.md` avec procédure de déploiement
   - [ ] Estimation des coûts mensuels OVH
   
   ## Structure de fichiers
   
   ```
   infrastructure/terraform/
   ├── main.tf              # Provider config
   ├── variables.tf         # Input variables
   ├── outputs.tf           # Output values
   ├── versions.tf          # Required providers
   ├── terraform.tfvars.example
   ├── network.tf           # VPC, subnets, security groups
   ├── compute.tf           # K3s master + workers
   ├── storage.tf           # Cinder volumes
   ├── dns.tf               # OVH DNS records
   ├── cloud-init/
   │   ├── master.yaml      # Cloud-init master
   │   └── worker.yaml      # Cloud-init workers
   └── modules/             # (optionnel, si refactoring)
   ```
   
   ## Dépendances
   
   - Compte OVH Public Cloud avec projet OpenStack
   - Clés API OVH (pour DNS)
   - #231 (R&D stratégie scaling — décisions architecture)
   
   ## Estimation
   
   6-8h
   
   ## Critères d'acceptation
   
   - [ ] `terraform plan` s'exécute sans erreur
   - [ ] `terraform apply` crée le cluster complet (réseau + 3 VMs + storage + DNS)
   - [ ] `terraform destroy` nettoie tout proprement
   - [ ] SSH fonctionnel vers toutes les instances
   - [ ] State Terraform stocké en remote (pas de state local commité)
   - [ ] Aucun secret dans le code (variables ou env uniquement)
   
   ---
   
   ## Considérations Green IT (Mars 2026)
   
   > **Priorité abaissée à low** — le scale-up VPS est plus écologique à court/moyen terme.
   
   ### Load tests actuels (VPS d2-2, 2 vCPU / 4GB RAM, 8€/mois)
   - Throughput : **287 req/s** soutenu
   - Mémoire backend : **5.5 MB** (pic)
   - Mémoire PostgreSQL : **94 MB**
   - CO2/requête : **0.12g** (7-25× moins que concurrents)
   - Capacité estimée : **2,000-3,000 copros** par VPS
   
   ### Comparaison énergétique
   | Config | Watts | CO2/an | Copros | Coût/mois |
   |--------|-------|--------|--------|-----------|
   | VPS d2-4 (scale-up) | ~20W | ~350 kg | ~5,000 | ~16€ |
   | K3s 3× b2-7 | ~60W | ~525 kg | ~5,000-8,000 | ~30-50€ |
   
   Le cluster K3s consomme **~3× plus d'énergie** pour la même charge utile car :
   - Overhead orchestration K3s incompressible : ~1.5 GB RAM + 0.5 vCPU (etcd, kubelet ×3, Traefik, Longhorn)
   - 3 machines = 3× l'idle power
   - Réseau inter-nœuds VXLAN ajoute latence + CPU
   
   ### Seuil de bascule recommandé
   Cette issue devient pertinente quand :
   - **> 5,000 copros** avec charge soutenue
   - **Besoin HA** (SLA > 99.9%, le VPS est un SPOF)
   - **Zero-downtime deployments** contractuellement obligatoires
   - **Scaling horizontal** nécessaire (pics AG annuelles)
   
   Jusque-là, un simple scale-up VPS (d2-4 → d2-8) est plus green, moins cher, et suffisant.

.. raw:: html

   </div>


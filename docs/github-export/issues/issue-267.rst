==================================================================================
Issue #267: feat(infra): Provisionning K3s avec Ansible (installation + hardening)
==================================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: phase:k3s,track:infrastructure priority:low
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/267>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Une fois les VMs créées par Terraform (#266), Ansible provisionne et configure le cluster K3s complet.
   L'infrastructure existante (`infrastructure/ansible/`) contient déjà des playbooks de sécurité VPS — on étend pour K3s.
   
   ## Objectifs
   
   Automatiser l'installation, la configuration et le hardening du cluster K3s sur les VMs OVH via Ansible.
   
   ## Scope
   
   ### 1. Inventaire dynamique
   
   - [ ] Inventaire dynamique OpenStack (`openstack_inventory.py` ou plugin `openstack.cloud.openstack`)
   - [ ] Groupes : `[k3s_master]`, `[k3s_workers]`, `[k3s:children]`
   - [ ] Variables de groupe : versions K3s, options réseau, token join
   - [ ] Fallback : inventaire statique `hosts.ini` pour debug
   
   ### 2. Playbook : Common (tous les nœuds)
   
   - [ ] Mise à jour système + packages essentiels
   - [ ] Configuration NTP (chrony)
   - [ ] SSH hardening (réutilisation du rôle existant `security-monitoring.yml`)
   - [ ] fail2ban configuration
   - [ ] Kernel hardening (sysctl — réutilisation rôle existant)
   - [ ] Firewall (ufw/nftables) avec règles K3s-specifiques
   - [ ] Utilisateur dédié `koprogo` (sans mot de passe, sudo limité)
   - [ ] Log rotation + envoi vers Loki
   
   ### 3. Playbook : K3s Master
   
   - [ ] Installation K3s server (`curl -sfL https://get.k3s.io | sh -`)
   - [ ] Options : `--disable traefik` (on installe Traefik v2 via Helm)
   - [ ] Options : `--write-kubeconfig-mode 644`
   - [ ] Récupération du token join (`/var/lib/rancher/k3s/server/node-token`)
   - [ ] Récupération du kubeconfig et adaptation (IP externe)
   - [ ] Installation `kubectl` + `helm` sur le master
   - [ ] Vérification santé du cluster (`kubectl get nodes`)
   
   ### 4. Playbook : K3s Workers
   
   - [ ] Installation K3s agent avec token join du master
   - [ ] Options : `--server https://<master-ip>:6443`
   - [ ] Labeling des nœuds (`node-role.kubernetes.io/worker=true`)
   - [ ] Vérification rattachement au cluster
   
   ### 5. Playbook : Post-installation cluster
   
   - [ ] Installation Longhorn (storage distribué)
   - [ ] Installation cert-manager (Let's Encrypt)
   - [ ] Installation Traefik v2 via Helm (IngressRoute CRDs)
   - [ ] Installation MetalLB ou kube-vip (LoadBalancer pour bare-metal)
   - [ ] Namespaces : `koprogo-prod`, `koprogo-staging`, `monitoring`, `argocd`
   - [ ] RBAC : ServiceAccounts pour ArgoCD, monitoring, app
   - [ ] Sealed Secrets ou SOPS pour gestion des secrets
   
   ### 6. Playbook : Monitoring migration
   
   - [ ] Migration Prometheus depuis VPS vers K3s (Helm chart `kube-prometheus-stack`)
   - [ ] Migration Grafana avec dashboards existants
   - [ ] Migration Loki + Promtail pour logs
   - [ ] Alertmanager avec alertes existantes
   - [ ] Dashboards K3s-specific (node health, pod metrics, etc.)
   
   ### 7. Playbook : PostgreSQL
   
   - [ ] Déploiement PostgreSQL 15 (StatefulSet avec PVC Longhorn ou volume Cinder)
   - [ ] OU : PostgreSQL externe (OVH Managed DB) — décision à prendre
   - [ ] PgBouncer connection pooling
   - [ ] Backups automatisés (pg_dump + GPG + S3, réutilisation script existant)
   - [ ] Monitoring PostgreSQL (pg_exporter)
   
   ## Structure de fichiers
   
   ```
   infrastructure/ansible/
   ├── inventory/
   │   ├── openstack.yml         # Inventaire dynamique
   │   └── hosts.ini             # Fallback statique
   ├── group_vars/
   │   ├── all.yml               # Variables communes
   │   ├── k3s_master.yml        # Variables master
   │   └── k3s_workers.yml       # Variables workers
   ├── roles/
   │   ├── common/               # Hardening + packages
   │   ├── k3s-master/           # Installation K3s server
   │   ├── k3s-worker/           # Installation K3s agent
   │   ├── k3s-post-install/     # Helm charts, namespaces, RBAC
   │   ├── monitoring/           # Prometheus/Grafana/Loki migration
   │   └── postgresql/           # DB deployment
   ├── playbooks/
   │   ├── site.yml              # Playbook principal (tout)
   │   ├── k3s-install.yml       # Cluster K3s seul
   │   ├── k3s-post-install.yml  # Post-installation
   │   └── monitoring.yml        # Monitoring seul
   └── security-monitoring.yml   # (existant, à adapter)
   ```
   
   ## Dépendances
   
   - #266 (Terraform — VMs doivent exister)
   - #231 (R&D — décisions PostgreSQL managed vs self-hosted)
   - Playbooks de sécurité existants (`infrastructure/ansible/security-monitoring.yml`)
   
   ## Estimation
   
   8-12h
   
   ## Critères d'acceptation
   
   - [ ] `ansible-playbook playbooks/site.yml` provisionne le cluster complet from scratch
   - [ ] `kubectl get nodes` montre 1 master + 2 workers `Ready`
   - [ ] Traefik Ingress fonctionnel (test avec un Ingress basique)
   - [ ] Longhorn storage provisionné et fonctionnel
   - [ ] cert-manager émet des certificats Let's Encrypt
   - [ ] PostgreSQL accessible depuis les pods applicatifs
   - [ ] Prometheus + Grafana accessibles avec dashboards
   - [ ] Playbook idempotent (re-run sans casse)
   - [ ] Aucun secret en clair dans le repo (vault-encrypted ou Sealed Secrets)
   
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
   
   L'overhead Ansible/K3s est significatif : etcd, kubelet ×3, Longhorn, cert-manager, monitoring = **~1.5 GB RAM + 0.5 vCPU** permanents, soit plus que l'app elle-même (5.5 MB backend).
   
   ### Seuil de bascule recommandé
   Cette issue devient pertinente quand :
   - **> 5,000 copros** avec charge soutenue
   - **Besoin HA** (SLA > 99.9%)
   - **Zero-downtime deployments** contractuellement obligatoires
   
   Les playbooks Ansible existants (sécurité VPS) restent utilisables en attendant. La migration K3s est une question de **résilience**, pas de performance ou d'écologie.

.. raw:: html

   </div>


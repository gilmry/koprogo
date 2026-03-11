Infrastructure - Terraform + Ansible
====================================

Infrastructure as Code (IaC) pour déploiement automatisé de KoproGo sur OVH Public Cloud.

**Localisation** : ``infrastructure/``

Vue d'ensemble
--------------

Le déploiement KoproGo utilise une approche **GitOps** complète :

1. **Terraform** : Provisionne le VPS OVH (compute, réseau, SSH keys)
2. **Ansible** : Configure le serveur (Docker, firewall, GitOps service)
3. **GitOps systemd** : Auto-update continu depuis GitHub (check toutes les 3 minutes)

**Coût Total** : ~8€/mois TTC (VPS + Domaine + Backups)

Stack Technologique
-------------------

.. list-table::
   :header-rows: 1
   :widths: 30 30 40

   * - Composant
     - Technologie
     - Rôle
   * - **Provisioning**
     - Terraform 1.0+
     - Création VPS OVH
   * - **Configuration**
     - Ansible 2.9+
     - Setup serveur automatisé
   * - **Container Runtime**
     - Docker 24+ (Compose V2)
     - Orchestration containers
   * - **Reverse Proxy**
     - Traefik 3.0
     - SSL automatique (Let's Encrypt)
   * - **GitOps**
     - systemd service
     - Auto-update toutes les 3 min
   * - **Monitoring**
     - Scripts shell + cron
     - Health checks + alertes
   * - **Backups**
     - Scripts shell + cron
     - Backup DB quotidien

Structure
---------

.. code-block:: text

   infrastructure/
   ├── terraform/              # Infrastructure as Code (Terraform)
   │   ├── main.tf             # Configuration principale VPS
   │   ├── variables.tf        # Variables configurables
   │   └── load-env.sh         # Chargement variables .env
   │
   └── ansible/                # Configuration Management (Ansible)
       ├── playbook.yml        # Playbook principal
       ├── fix-env.yml         # Fix permissions .env
       ├── files/              # Fichiers statiques
       │   └── configure-ovh-dns.py  # Script config DNS OVH
       └── templates/          # Templates Jinja2
           ├── env-production.j2          # Template .env production
           ├── koprogo-gitops.service.j2  # Service systemd GitOps
           ├── health-check.sh.j2         # Script monitoring
           └── backup.sh.j2               # Script backups

Modules
-------

.. toctree::
   :maxdepth: 2

   terraform/index
   ansible/index

Déploiement Complet (TL;DR)
----------------------------

**Depuis la racine du projet** :

.. code-block:: bash

   # 🚀 Commande unique (20-30 minutes)
   make setup-infra

   # Ou manuellement:
   cd infrastructure
   ./terraform-apply.sh
   cd ansible
   ansible-playbook -i inventory.ini playbook.yml

**Ce que fait setup-infra** :

1. ✅ Provisionne VPS OVH avec Terraform (d2-2: 2 vCPU, 4GB RAM)
2. ✅ Configure serveur avec Ansible (Docker, firewall, fail2ban)
3. ✅ Déploie Docker Compose (Traefik + Backend + Frontend + PostgreSQL)
4. ✅ Configure DNS automatique (A records frontend + API)
5. ✅ Active GitOps (systemd service, check toutes les 3 minutes)
6. ✅ Configure backups quotidiens (PostgreSQL dump)
7. ✅ Configure health checks (monitoring toutes les 5 minutes)

Infrastructure Provisionnée
----------------------------

**VPS d2-2 (OVH Public Cloud)** :

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Ressource
     - Spécification
   * - **vCPUs**
     - 2 vCPUs Intel Xeon
   * - **RAM**
     - 4 GB DDR4
   * - **Stockage**
     - 25 GB SSD NVMe
   * - **Réseau**
     - 100 Mbps (illimité)
   * - **OS**
     - Ubuntu 22.04 LTS
   * - **Région**
     - GRA11 (Gravelines, France - bas carbone 60g CO2/kWh)
   * - **Prix**
     - ~7€ TTC/mois

**Services Déployés** :

- **Traefik** : Reverse proxy + SSL Let's Encrypt automatique
- **Backend API** : Actix-web Rust (port 8080 interne)
- **Frontend** : Astro SSG + Svelte (port 3000 interne)
- **PostgreSQL 15** : Base de données (port 5432 interne)

**Exposition Publique** :

- ``https://koprogo.com`` → Frontend (Traefik → port 3000)
- ``https://api.koprogo.com`` → Backend (Traefik → port 8080)
- Ports 80/443 : Traefik (SSL termination)
- Port 22 : SSH (accès maintenance)

Flux de Déploiement
--------------------

.. code-block:: text

   ┌─────────────────────────────────────────────────────────────┐
   │                    DEVELOPER MACHINE                        │
   │                                                             │
   │  1. make setup-infra                                        │
   │     ↓                                                       │
   │  2. Terraform provisionne VPS OVH                           │
   │     ├─ Crée compute instance (d2-2)                         │
   │     ├─ Configure réseau public (Ext-Net)                    │
   │     └─ Upload SSH key                                       │
   │     ↓                                                       │
   │  3. Ansible configure serveur                               │
   │     ├─ Install Docker + Docker Compose                      │
   │     ├─ Configure UFW (firewall)                             │
   │     ├─ Clone repo GitHub                                    │
   │     ├─ Déploie Docker Compose                               │
   │     ├─ Configure DNS OVH (A records)                        │
   │     ├─ Install GitOps systemd service                       │
   │     └─ Configure backups + monitoring                       │
   └─────────────────────────────────────────────────────────────┘
                              ↓
   ┌─────────────────────────────────────────────────────────────┐
   │                      VPS OVH (GRA11)                        │
   │                                                             │
   │  ┌─────────────────────────────────────────────────────┐   │
   │  │         GitOps Systemd Service                      │   │
   │  │  (check GitHub toutes les 3 minutes)                │   │
   │  │                                                     │   │
   │  │  1. git fetch origin main                           │   │
   │  │  2. Si commit différent:                            │   │
   │  │     ├─ git pull                                     │   │
   │  │     ├─ docker compose down                          │   │
   │  │     ├─ docker compose pull                          │   │
   │  │     ├─ docker compose up -d                         │   │
   │  │     └─ Health check API                             │   │
   │  │  3. Si health check échoue: rollback automatique    │   │
   │  └─────────────────────────────────────────────────────┘   │
   │                                                             │
   │  ┌─────────────────────────────────────────────────────┐   │
   │  │              Docker Compose Stack                   │   │
   │  │                                                     │   │
   │  │  Traefik (80/443) ← SSL Let's Encrypt              │   │
   │  │    ├→ Frontend (3000)                               │   │
   │  │    └→ Backend API (8080)                            │   │
   │  │         └→ PostgreSQL (5432)                        │   │
   │  └─────────────────────────────────────────────────────┘   │
   └─────────────────────────────────────────────────────────────┘

Sécurité
--------

**Firewall UFW** :

.. code-block:: bash

   # Ports autorisés
   22/tcp   → SSH (maintenance)
   80/tcp   → HTTP (redirect vers HTTPS)
   443/tcp  → HTTPS (Traefik)

   # Tout le reste: DENY

**Fail2ban** :

Protection contre bruteforce SSH (5 tentatives = ban 10 minutes).

**SSL/TLS** :

- Certificats Let's Encrypt automatiques (Traefik)
- Renouvellement automatique tous les 60 jours
- HTTPS uniquement (HTTP redirect)

**Isolation Containers** :

- Réseau Docker privé (backend ↔ postgres)
- Seul Traefik exposé publiquement
- Pas de port mapping direct (sauf Traefik)

**Secrets Management** :

.. code-block:: bash

   # Fichier .env (permissions 0600)
   DATABASE_URL=postgresql://...
   JWT_SECRET=<256-bit-random>

   # Jamais commité dans Git
   # Généré par Ansible template

GitOps Auto-Update
------------------

**Service systemd** : ``koprogo-gitops.service``

.. code-block:: ini

   [Unit]
   Description=KoproGo GitOps Auto-Update
   After=docker.service network-online.target
   Requires=docker.service

   [Service]
   Type=simple
   User=root
   WorkingDirectory=/home/koprogo/koprogo
   ExecStart=/home/koprogo/koprogo/deploy/production/gitops-deploy.sh monitor
   Restart=always
   RestartSec=180  # 3 minutes

   [Install]
   WantedBy=multi-user.target

**Workflow** :

1. Service check Git toutes les 3 minutes
2. Si nouveau commit sur ``main`` :
   - Pull automatique
   - Docker Compose down
   - Docker Compose pull (nouvelles images)
   - Docker Compose up -d
   - Health check API (``https://api.koprogo.com/api/v1/health``)
3. Si health check échoue :
   - Rollback automatique (``git reset --hard ORIG_HEAD``)
   - Docker Compose up avec ancienne version
   - Alerte log

**Logs** :

.. code-block:: bash

   # Voir logs GitOps
   sudo tail -f /var/log/koprogo-gitops.log

   # Status service
   sudo systemctl status koprogo-gitops

Backups
-------

**Backup PostgreSQL Quotidien** : 2h du matin (cron)

.. code-block:: bash

   # Script: /home/koprogo/koprogo/scripts/backup.sh

   # Backup DB
   docker exec koprogo-postgres pg_dumpall -U postgres \
       > /home/koprogo/backups/postgres_$(date +%Y%m%d).sql

   # Rotation: garde 7 derniers jours
   find /home/koprogo/backups -name "postgres_*.sql" \
       -mtime +7 -delete

**Logs** :

.. code-block:: bash

   tail -f /var/log/koprogo-backup.log

Monitoring
----------

**Health Check toutes les 5 minutes** : cron

.. code-block:: bash

   # Script: /home/koprogo/koprogo/scripts/health-check.sh

   # Check API health
   curl -f https://api.koprogo.com/api/v1/health || \
       echo "ALERT: API down!" | mail -s "KoproGo Alert" admin@example.com

**Métriques Docker** :

.. code-block:: bash

   # Stats containers
   docker stats

   # Logs en temps réel
   cd /home/koprogo/koprogo/deploy/production
   docker compose logs -f

**Métriques Système** :

.. code-block:: bash

   # CPU/RAM/Disk
   htop

   # Disk usage
   ncdu /

Coûts Infrastructure
--------------------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Ressource
     - Coût Mensuel
     - Coût Annuel
   * - **VPS d2-2 (OVH)**
     - 6,96€ TTC
     - 83,52€
   * - **Domaine .be**
     - 0,83€ TTC
     - 10€
   * - **Backups (optionnel)**
     - 0€ (local VPS)
     - 0€
   * - **SSL Let's Encrypt**
     - 0€ (gratuit)
     - 0€
   * - **TOTAL**
     - **~8€ TTC**
     - **~94€**

**Capacité** : 2,000-3,000 copropriétés cloud sur ce VPS.

Empreinte Carbone
-----------------

**Datacenter GRA11** (Gravelines, France) :

- **Intensité carbone** : 60g CO2/kWh (mix électrique français)
- **Consommation VPS** : ~10W constant
- **Émissions mensuelles** : ~0.43 kg CO2
- **Émissions annuelles** : ~5.3 kg CO2

**Par utilisateur** (2,000 copropriétés) :

- **2.6g CO2/copro/an** (infrastructure)
- **< 0.5g CO2/requête** (objectif atteint ✅)

**Comparaison** :

- KoproGo cloud : 5.3 kg CO2/an (2,000 copros)
- WordPress typique : 120 kg CO2/an (1 site)
- **96% de réduction** grâce à Rust + architecture efficace

Scalabilité
-----------

**Scaling Vertical** (augmenter VPS) :

.. code-block:: bash

   # Passer à d2-4 (4 vCPU, 8GB RAM) : ~14€/mois
   # Modifier infrastructure/terraform/main.tf:
   flavor_name = "d2-4"

   # Appliquer
   terraform apply

**Scaling Horizontal** (multi-VPS) :

Pour > 3,000 copropriétés :

1. Load Balancer OVH (2€/mois)
2. Plusieurs VPS d2-2 derrière LB
3. PostgreSQL centralisé (Managed Database 15€/mois)
4. Redis cluster (session sharing)

**Architecture Cible 10,000+ copros** :

.. code-block:: text

   Load Balancer OVH
        ↓
   ┌────┴────┬────────┬────────┐
   VPS 1     VPS 2     VPS 3     VPS N
   (API)     (API)     (API)     (API)
        ↓
   PostgreSQL Managed (Master-Replica)
        ↓
   Redis Cluster (session)

Maintenance
-----------

**Commandes Utiles** :

.. code-block:: bash

   # SSH vers VPS
   ssh ubuntu@<VPS_IP>

   # Voir logs GitOps
   sudo tail -f /var/log/koprogo-gitops.log

   # Redémarrer services
   cd /home/koprogo/koprogo/deploy/production
   docker compose restart

   # Forcer update manuelle
   sudo systemctl restart koprogo-gitops

   # Backup manuel
   /home/koprogo/koprogo/scripts/backup.sh

   # Restaurer backup
   cat backup.sql | docker exec -i koprogo-postgres psql -U postgres

**Mise à jour système** :

.. code-block:: bash

   sudo apt update && sudo apt upgrade -y
   sudo reboot  # Si kernel update

Troubleshooting
---------------

Voir documentation complète : ``docs/deployment/troubleshooting.md``

**Problèmes Communs** :

.. code-block:: bash

   # Service GitOps ne démarre pas
   sudo systemctl status koprogo-gitops
   sudo journalctl -u koprogo-gitops -n 50

   # API ne répond pas
   docker compose logs backend
   curl https://api.koprogo.com/api/v1/health

   # SSL non configuré
   docker compose logs traefik
   # Vérifier DNS: A records pointent vers VPS IP

CI/CD GitHub Actions
--------------------

**Workflow** : ``.github/workflows/deploy.yml`` (futur)

.. code-block:: yaml

   name: Deploy to Production

   on:
     push:
       branches: [main]

   jobs:
     deploy:
       runs-on: ubuntu-latest
       steps:
         - name: Checkout
           uses: actions/checkout@v3

         - name: Run tests
           run: make test

         - name: Build Docker images
           run: |
             docker build -t koprogo-backend:${{ github.sha }} backend/
             docker build -t koprogo-frontend:${{ github.sha }} frontend/

         - name: Push to registry
           run: |
             docker tag koprogo-backend:${{ github.sha }} registry/backend:latest
             docker push registry/backend:latest

         # GitOps service sur VPS pull automatiquement

Références
----------

- Terraform OVH Provider : https://registry.terraform.io/providers/ovh/ovh/
- Ansible Docs : https://docs.ansible.com/
- Docker Compose : https://docs.docker.com/compose/
- Traefik : https://doc.traefik.io/traefik/
- Let's Encrypt : https://letsencrypt.org/

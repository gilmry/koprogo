
Terraform + Ansible
===================

Détails techniques du provisionnement VPS (Terraform) et de la configuration serveur (Ansible).

----

🏗️ Architecture de Déploiement
------------------------------

.. code-block::

   ┌─────────────────── INTERNET ────────────────────┐
   │                                                  │
   │         HTTPS (Port 443)                        │
   │               ↓                                 │
   │    ┌──────────────────────────┐               │
   │    │      Traefik Proxy       │               │
   │    │  (Let's Encrypt SSL)     │               │
   │    └───────────┬──────────────┘               │
   │                ↓                                │
   │    ┌──────────────────────────┐               │
   │    │    Docker Compose        │               │
   │    │                           │               │
   │    │  ┌────────┐  ┌────────┐ │               │
   │    │  │Frontend│  │Backend │ │               │
   │    │  │ :3000  │  │ :8080  │ │               │
   │    │  └───┬────┘  └────┬───┘ │               │
   │    │      │            │      │               │
   │    │      └────────────┼────┐ │               │
   │    │                   │    │ │               │
   │    │            ┌──────▼────▼┐│               │
   │    │            │  PostgreSQL││               │
   │    │            │   :5432    ││               │
   │    │            └────────────┘│               │
   │    └──────────────────────────┘               │
   │                                                  │
   │         VPS OVH d2-2                            │
   │  2 vCPU, 4GB RAM, 25GB SSD                     │
   │  Gravelines GRA9 (France)                       │
   └──────────────────────────────────────────────────┘

----

1. Terraform : Provisionnement VPS
----------------------------------

1.1 Ressources créées
^^^^^^^^^^^^^^^^^^^^^

Le fichier ``infrastructure/terraform/main.tf`` provisionne :


#. 
   **VPS** : Instance Compute OpenStack (d2-2)


   * 2 vCPU
   * 4GB RAM
   * 25GB SSD
   * Ubuntu 22.04 LTS

#. 
   **Réseau** : Configuration réseau automatique


   * IP publique fixe
   * Règles de sécurité (ports 22, 80, 443)

#. 
   **DNS** (si activé) : Enregistrements via API OVH


   * Type A pour domaine principal
   * Type A pour sous-domaines (api, app)

1.2 Variables Terraform
^^^^^^^^^^^^^^^^^^^^^^^

Définies dans ``infrastructure/terraform/.env`` :

.. code-block:: bash

   # OpenStack (REQUIS)
   OS_PROJECT_ID=xxxxx
   OS_USERNAME=user-xxxxx
   OS_PASSWORD=xxxxx
   OS_REGION_NAME=GRA9

   # OVH API (OPTIONNEL - pour DNS)
   OVH_APPLICATION_KEY=xxxxx
   OVH_APPLICATION_SECRET=xxxxx
   OVH_CONSUMER_KEY=xxxxx
   OVH_ENDPOINT=ovh-eu

   # Configuration VPS
   VPS_FLAVOR=d2-2
   VPS_IMAGE=Ubuntu 22.04
   SSH_PUBLIC_KEY=/home/user/.ssh/id_rsa.pub

   # Domaine (OPTIONNEL)
   DOMAIN=koprogo.com
   API_DOMAIN=api.koprogo.com
   APP_DOMAIN=app.koprogo.com

1.3 Commandes Terraform
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd infrastructure/terraform

   # Charger variables d'environnement
   source ./load-env.sh

   # Initialiser (première fois)
   terraform init

   # Prévisualiser changements
   terraform plan

   # Appliquer (provisionner VPS)
   terraform apply

   # Outputs
   terraform output vps_ip
   terraform output vps_id

   # Détruire infrastructure
   terraform destroy

----

2. Ansible : Configuration Serveur
----------------------------------

2.1 Rôles Ansible
^^^^^^^^^^^^^^^^^

Le playbook ``infrastructure/ansible/playbook.yml`` configure :

Rôle 1 : **Système de Base**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Mise à jour packages (\ ``apt update && apt upgrade``\ )
* Installation utilitaires (curl, wget, git, htop, etc.)
* Configuration timezone (Europe/Paris)
* Configuration locale (fr_FR.UTF-8)

Rôle 2 : **Sécurité**
~~~~~~~~~~~~~~~~~~~~~~~~~


* 
  **Firewall UFW**


  * Port 22 (SSH) ✅
  * Port 80 (HTTP) ✅
  * Port 443 (HTTPS) ✅
  * Reste bloqué ❌

* 
  **Fail2ban**


  * Protection brute-force SSH
  * Ban après 5 tentatives échouées
  * Ban 10 minutes

* 
  **Durcissement SSH**


  * Désactivation root login
  * Désactivation password authentication
  * Clé SSH uniquement

Rôle 3 : **Docker**
~~~~~~~~~~~~~~~~~~~~~~~


* Installation Docker Engine (version latest)
* Installation Docker Compose v2 (plugin)
* Ajout utilisateur ``koprogo`` au groupe docker
* Configuration Docker daemon (logs rotation)

Rôle 4 : **Utilisateur koprogo**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Création utilisateur système ``koprogo``
* Home directory : ``/home/koprogo``
* Shell : ``/bin/bash``
* Accès sudo sans password (pour GitOps)
* Clé SSH autorisée

Rôle 5 : **Repository Git**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Clone repository GitHub : ``github.com/gilmry/koprogo``
* Checkout branche : ``main``
* Permissions : ``koprogo:koprogo``
* SSH key GitHub configurée

Rôle 6 : **Configuration Application**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Génération fichier ``.env`` depuis template
* Variables :
  .. code-block:: bash

     DATABASE_URL=postgresql://koprogo:${DB_PASSWORD}@postgres:5432/koprogo_db
     API_URL=https://${API_DOMAIN}
     FRONTEND_URL=https://${APP_DOMAIN}
     CORS_ALLOWED_ORIGINS=https://${APP_DOMAIN}
     JWT_SECRET=${JWT_SECRET}

Rôle 7 : **Docker Compose**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Déploiement stack complète
* Services :

  * ``traefik`` : Reverse proxy + SSL
  * ``postgres`` : Base de données
  * ``backend`` : API Rust
  * ``frontend`` : Application Astro/Svelte

* Réseaux Docker privés
* Volumes persistants

Rôle 8 : **GitOps**
~~~~~~~~~~~~~~~~~~~~~~~


* Installation service systemd : ``koprogo-gitops.service``
* Configuration timer : vérification toutes les 3 minutes
* Script : ``/home/koprogo/koprogo/deploy/production/gitops-deploy.sh``
* Activation automatique au démarrage

Rôle 9 : **Backups**
~~~~~~~~~~~~~~~~~~~~~~~~


* Script backup PostgreSQL : ``/home/koprogo/koprogo/scripts/backup.sh``
* Cron job : tous les jours à 2h du matin
* Rétention : 7 jours
* Destination : ``/home/koprogo/koprogo/backups/``

Rôle 10 : **Health Check**
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Vérification finale : ``https://${API_DOMAIN}/api/v1/health``
* 10 retries (délai 10s entre chaque)
* Timeout total : 100 secondes
* Validation SSL désactivée (cert Let's Encrypt peut ne pas être immédiat)

2.2 Templates Ansible
^^^^^^^^^^^^^^^^^^^^^

Fichiers générés dynamiquement :

.. list-table::
   :header-rows: 1

   * - Template
     - Destination
     - Description
   * - ``env-production.j2``
     - ``/home/koprogo/koprogo/.env``
     - Variables d'environnement application
   * - ``gitops-deploy.sh.j2``
     - ``/home/koprogo/koprogo/deploy/production/gitops-deploy.sh``
     - Script GitOps
   * - ``koprogo-gitops.service.j2``
     - ``/etc/systemd/system/koprogo-gitops.service``
     - Service systemd
   * - ``backup.sh.j2``
     - ``/home/koprogo/koprogo/scripts/backup.sh``
     - Script backup PostgreSQL


2.3 Commandes Ansible
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd infrastructure/ansible

   # Dry-run (simulation)
   ansible-playbook -i inventory.ini playbook.yml --check

   # Exécution complète
   ansible-playbook -i inventory.ini playbook.yml

   # Exécution avec verbosité
   ansible-playbook -i inventory.ini playbook.yml -vvv

   # Exécuter seulement certains rôles
   ansible-playbook -i inventory.ini playbook.yml --tags "docker,app"

----

3. Traefik : Reverse Proxy + SSL
--------------------------------

3.1 Configuration Traefik
^^^^^^^^^^^^^^^^^^^^^^^^^

Fichier : ``deploy/production/traefik.yml``

**Entrypoints** :


* ``web`` : Port 80 (HTTP) → Redirect vers HTTPS
* ``websecure`` : Port 443 (HTTPS)

**Certificats** :


* Let's Encrypt ACME
* Challenge HTTP-01
* Email : défini dans ``.env``
* Stockage : ``/home/koprogo/koprogo/deploy/production/letsencrypt/acme.json``

**Providers** :


* Docker (détection automatique containers)
* Labels sur services backend/frontend pour routing

3.2 Labels Docker
^^^^^^^^^^^^^^^^^

Backend (\ ``docker-compose.yml``\ ) :

.. code-block:: yaml

   backend:
     labels:
       - "traefik.enable=true"
       - "traefik.http.routers.api.rule=Host(`api.${DOMAIN}`)"
       - "traefik.http.routers.api.entrypoints=websecure"
       - "traefik.http.routers.api.tls.certresolver=letsencrypt"
       - "traefik.http.services.api.loadbalancer.server.port=8080"

Frontend :

.. code-block:: yaml

   frontend:
     labels:
       - "traefik.enable=true"
       - "traefik.http.routers.app.rule=Host(`app.${DOMAIN}`)"
       - "traefik.http.routers.app.entrypoints=websecure"
       - "traefik.http.routers.app.tls.certresolver=letsencrypt"
       - "traefik.http.services.app.loadbalancer.server.port=3000"

----

4. Docker Compose
-----------------

4.1 Services
^^^^^^^^^^^^

Traefik
~~~~~~~

.. code-block:: yaml

   traefik:
     image: traefik:v2.10
     ports:
       - "80:80"
       - "443:443"
     volumes:
       - /var/run/docker.sock:/var/run/docker.sock:ro
       - ./traefik.yml:/etc/traefik/traefik.yml:ro
       - ./letsencrypt:/letsencrypt

PostgreSQL
~~~~~~~~~~

.. code-block:: yaml

   postgres:
     image: postgres:15-alpine
     environment:
       POSTGRES_DB: koprogo_db
       POSTGRES_USER: koprogo
       POSTGRES_PASSWORD: ${DB_PASSWORD}
     volumes:
       - postgres_data:/var/lib/postgresql/data

Backend
~~~~~~~

.. code-block:: yaml

   backend:
     image: ghcr.io/gilmry/koprogo-backend:latest
     environment:
       DATABASE_URL: ${DATABASE_URL}
       JWT_SECRET: ${JWT_SECRET}
       CORS_ALLOWED_ORIGINS: ${CORS_ALLOWED_ORIGINS}
     depends_on:
       - postgres

Frontend
~~~~~~~~

.. code-block:: yaml

   frontend:
     image: ghcr.io/gilmry/koprogo-frontend:latest
     environment:
       PUBLIC_API_URL: ${API_URL}

4.2 Volumes
^^^^^^^^^^^


* ``postgres_data`` : Données PostgreSQL persistantes
* ``./letsencrypt`` : Certificats SSL Let's Encrypt

4.3 Réseaux
^^^^^^^^^^^


* ``koprogo-network`` : Réseau privé interne

  * Backend ↔ PostgreSQL
  * Frontend ↔ Backend (via Traefik)

----

5. Workflow Complet
-------------------

Ordre d'exécution
^^^^^^^^^^^^^^^^^


#. **Terraform** provisionne VPS (~5 min)
#. **Ansible** configure serveur (~10 min)

   * Installation Docker
   * Clone repository
   * Génération ``.env``
   * Démarrage Docker Compose

#. **Traefik** génère certificat SSL (~1-2 min)
#. **Health check** valide déploiement
#. **GitOps** activé (vérification toutes les 3 min)

Timeline typique
^^^^^^^^^^^^^^^^

.. code-block::

   T+0min    : terraform apply
   T+5min    : VPS provisionné, IP assignée
   T+5min    : ansible-playbook démarre
   T+7min    : Docker installé
   T+10min   : Repository cloné
   T+12min   : Docker Compose up
   T+14min   : Traefik génère certificat SSL
   T+15min   : Health check réussi ✅
   T+18min   : Premier check GitOps

----

6. Vérification Post-Déploiement
--------------------------------

Sur votre machine locale
^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Vérifier outputs Terraform
   cd infrastructure/terraform
   terraform output

   # Health check API
   curl https://api.votre-domaine.com/api/v1/health
   # {"status":"healthy","timestamp":"..."}

Sur le VPS
^^^^^^^^^^

.. code-block:: bash

   # SSH
   ssh ubuntu@$(cd infrastructure/terraform && terraform output -raw vps_ip)

   # Passer en utilisateur koprogo
   sudo su - koprogo
   cd ~/koprogo/deploy/production

   # Vérifier services Docker
   docker compose ps

   # Logs en temps réel
   docker compose logs -f

   # Status GitOps
   sudo systemctl status koprogo-gitops.service
   sudo journalctl -u koprogo-gitops.service -f

----

7. Mises à Jour Infrastructure
------------------------------

Modifier configuration Terraform
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd infrastructure/terraform

   # Éditer variables
   nano .env

   # Prévisualiser changements
   terraform plan

   # Appliquer
   terraform apply

Modifier configuration Ansible
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd infrastructure/ansible

   # Éditer playbook
   nano playbook.yml

   # Réexécuter
   ansible-playbook -i inventory.ini playbook.yml

Recréer containers Docker
^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Sur le VPS
   cd ~/koprogo/deploy/production

   # Rebuild avec nouvelles images
   docker compose down
   docker compose pull
   docker compose up -d

   # Ou forcer rebuild local (dev)
   docker compose up -d --build

----

📚 Fichiers Importants
----------------------

.. list-table::
   :header-rows: 1

   * - Fichier
     - Description
   * - ``infrastructure/terraform/main.tf``
     - Définition infrastructure Terraform
   * - ``infrastructure/terraform/.env``
     - Variables d'environnement (gitignored)
   * - ``infrastructure/ansible/playbook.yml``
     - Playbook Ansible principal
   * - ``infrastructure/ansible/inventory.ini``
     - Inventaire serveurs (généré par Terraform)
   * - ``deploy/production/docker-compose.yml``
     - Stack Docker
   * - ``deploy/production/traefik.yml``
     - Configuration Traefik
   * - ``deploy/production/.env``
     - Variables application (généré par Ansible)


----

🔗 Prochaine Étape
------------------

Configuration terminée ? Découvrir le fonctionnement de **\ `GitOps <gitops.md>`_\ **

----

**Dernière mise à jour** : Octobre 2025

**KoproGo ASBL** 🚀

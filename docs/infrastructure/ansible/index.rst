Ansible - Configuration Management
===================================

Ansible automatise la configuration complète du VPS : installation Docker, firewall, déploiement KoproGo, et setup GitOps.

**Localisation** : ``infrastructure/ansible/``

Vue d'ensemble
--------------

**Ansible** permet de configurer automatiquement un serveur vierge :

- ✅ Déclaratif : Playbooks YAML lisibles
- ✅ Idempotent : Peut être réexécuté sans risque
- ✅ Agentless : Pas d'agent à installer (SSH uniquement)
- ✅ Templating : Jinja2 pour fichiers dynamiques
- ✅ Extensible : Modules pour tous les services

**Durée** : ~5-10 minutes pour configuration complète

Structure
---------

.. code-block:: text

   infrastructure/ansible/
   ├── playbook.yml            # Playbook principal (12 sections)
   ├── fix-env.yml             # Playbook fix permissions .env
   ├── inventory.ini           # Inventaire des serveurs (git ignored)
   ├── inventory.ini.example   # Template inventaire
   ├── files/                  # Fichiers statiques
   │   └── configure-ovh-dns.py  # Script Python config DNS OVH
   └── templates/              # Templates Jinja2
       ├── env-production.j2          # Template .env production
       ├── koprogo-gitops.service.j2  # Service systemd GitOps
       ├── health-check.sh.j2         # Script monitoring
       └── backup.sh.j2               # Script backups

Prérequis
---------

**1. Ansible CLI**

.. code-block:: bash

   # Linux (Ubuntu/Debian)
   sudo apt update
   sudo apt install ansible

   # macOS
   brew install ansible

   # Vérifier
   ansible --version
   # Output: ansible [core 2.15.x]

**2. VPS Provisionné**

VPS créé via Terraform avec :

- Ubuntu 22.04 LTS
- IP publique
- SSH key configurée

**3. Clé SSH Locale**

.. code-block:: bash

   # Vérifier clé SSH
   ls ~/.ssh/id_rsa

   # Si absente, créer
   ssh-keygen -t rsa -b 4096 -C "you@example.com"

Configuration
-------------

Inventory
^^^^^^^^^

**inventory.ini** : Liste des serveurs cibles.

.. code-block:: bash

   cd infrastructure/ansible
   cp inventory.ini.example inventory.ini
   nano inventory.ini

**Exemple inventory.ini** :

.. code-block:: ini

   [koprogo]
   51.178.12.34 ansible_user=ubuntu ansible_ssh_private_key_file=~/.ssh/id_rsa

   [koprogo:vars]
   ansible_python_interpreter=/usr/bin/python3

**Variables** :

- ``ansible_user`` : Utilisateur SSH (ubuntu pour Ubuntu)
- ``ansible_ssh_private_key_file`` : Chemin clé privée
- ``ansible_python_interpreter`` : Python3 requis

Variables d'environnement
^^^^^^^^^^^^^^^^^^^^^^^^^^

**Définir variables Ansible** :

.. code-block:: bash

   export KOPROGO_DOMAIN=koprogo.be
   export KOPROGO_FRONTEND_DOMAIN=koprogo.be
   export KOPROGO_API_DOMAIN=api.koprogo.be
   export ACME_EMAIL=admin@koprogo.be

   # Credentials OVH (pour DNS automatique)
   export OVH_ENDPOINT=ovh-eu
   export OVH_APPLICATION_KEY=your_app_key
   export OVH_APPLICATION_SECRET=your_app_secret
   export OVH_CONSUMER_KEY=your_consumer_key

**Variables optionnelles** :

.. code-block:: bash

   export GITHUB_REPOSITORY=gilmry/koprogo
   export IMAGE_TAG=latest
   export RUST_LOG=info
   export ACTIX_WORKERS=2
   export DB_POOL_MAX_CONNECTIONS=10

Playbook Principal
------------------

Structure playbook.yml
^^^^^^^^^^^^^^^^^^^^^^^

**playbook.yml** : 12 sections automatisées

.. code-block:: yaml

   ---
   - name: Configure KoproGo VPS with GitOps Auto-Update
     hosts: koprogo
     become: yes
     vars:
       koprogo_user: koprogo
       koprogo_repo: https://github.com/gilmry/koprogo.git
       koprogo_branch: main
       koprogo_dir: /home/{{ koprogo_user }}/koprogo
       koprogo_domain: "{{ lookup('env', 'KOPROGO_DOMAIN') | default('', true) }}"
       frontend_domain: "{{ lookup('env', 'KOPROGO_FRONTEND_DOMAIN') | default('', true) }}"
       api_domain: "{{ lookup('env', 'KOPROGO_API_DOMAIN') | default('', true) }}"
       enable_ssl: "{{ koprogo_domain != '' }}"
       acme_email: "{{ lookup('env', 'ACME_EMAIL') | default('admin@example.com', true) }}"

     tasks:
       # 1. Mise à jour système
       # 2. Installation dépendances
       # 3. Installation Docker
       # 4. Création utilisateur KoproGo
       # 5. Configuration firewall (UFW)
       # 6. Clone dépôt GitHub
       # 7. Configuration environnement (.env)
       # 8. Configuration DNS OVH (optionnel)
       # 9. Déploiement Docker Compose
       # 10. Configuration GitOps (systemd service)
       # 11. Configuration backups (cron)
       # 12. Monitoring (health checks)
       # 13. Vérifications finales

Section par Section
^^^^^^^^^^^^^^^^^^^^

**1. Mise à jour système**

.. code-block:: yaml

   - name: Update apt cache
     apt:
       update_cache: yes
       cache_valid_time: 3600

   - name: Upgrade all packages
     apt:
       upgrade: dist
       autoremove: yes

**2. Installation dépendances**

.. code-block:: yaml

   - name: Install required packages
     apt:
       name:
         - apt-transport-https
         - ca-certificates
         - curl
         - gnupg
         - lsb-release
         - git
         - ufw
         - fail2ban
         - htop
         - ncdu
       state: present

**3. Installation Docker**

.. code-block:: yaml

   - name: Add Docker GPG key
     apt_key:
       url: https://download.docker.com/linux/ubuntu/gpg
       state: present

   - name: Add Docker repository
     apt_repository:
       repo: "deb [arch=amd64] https://download.docker.com/linux/ubuntu {{ ansible_distribution_release }} stable"
       state: present

   - name: Install Docker
     apt:
       name:
         - docker-ce
         - docker-ce-cli
         - containerd.io
         - docker-compose-plugin
       state: present

   - name: Start and enable Docker
     systemd:
       name: docker
       state: started
       enabled: yes

**4. Création utilisateur koprogo**

.. code-block:: yaml

   - name: Create koprogo user
     user:
       name: "{{ koprogo_user }}"
       shell: /bin/bash
       create_home: yes
       groups: docker
       append: yes

**5. Configuration firewall UFW**

.. code-block:: yaml

   - name: Configure UFW defaults
     ufw:
       direction: "{{ item.direction }}"
       policy: "{{ item.policy }}"
     loop:
       - { direction: 'incoming', policy: 'deny' }
       - { direction: 'outgoing', policy: 'allow' }

   - name: Allow SSH
     ufw:
       rule: allow
       port: '22'
       proto: tcp

   - name: Allow HTTP
     ufw:
       rule: allow
       port: '80'
       proto: tcp

   - name: Allow HTTPS
     ufw:
       rule: allow
       port: '443'
       proto: tcp

   - name: Enable UFW
     ufw:
       state: enabled

**Résultat** :

.. code-block:: text

   22/tcp   → SSH (maintenance)
   80/tcp   → HTTP (redirect HTTPS)
   443/tcp  → HTTPS (Traefik)
   * → DENY (défaut)

**6. Clone dépôt GitHub**

.. code-block:: yaml

   - name: Remove existing KoproGo directory if present
     file:
       path: "{{ koprogo_dir }}"
       state: absent

   - name: Clone KoproGo repository
     git:
       repo: "{{ koprogo_repo }}"
       dest: "{{ koprogo_dir }}"
       version: "{{ koprogo_branch }}"

   - name: Set ownership
     file:
       path: "{{ koprogo_dir }}"
       owner: "{{ koprogo_user }}"
       group: "{{ koprogo_user }}"
       recurse: yes

**7. Configuration environnement (.env)**

.. code-block:: yaml

   - name: Create .env file for production
     template:
       src: env-production.j2
       dest: "{{ koprogo_dir }}/deploy/production/.env"
       owner: "{{ koprogo_user }}"
       group: "{{ koprogo_user }}"
       mode: '0600'

**Template env-production.j2** génère ``.env`` dynamique :

.. code-block:: bash

   # GitHub Container Registry
   GITHUB_REPOSITORY=gilmry/koprogo
   IMAGE_TAG=latest

   # Traefik Configuration
   ACME_EMAIL=admin@koprogo.be
   API_DOMAIN=api.koprogo.be
   FRONTEND_DOMAIN=koprogo.be

   # Database (password auto-généré)
   POSTGRES_DB=koprogo_db
   POSTGRES_USER=koprogo
   POSTGRES_PASSWORD=<random-32-chars>

   # Backend (JWT secret auto-généré)
   JWT_SECRET=<random-64-chars>
   CORS_ALLOWED_ORIGINS=https://koprogo.be,https://api.koprogo.be

   # Frontend
   PUBLIC_API_URL=https://api.koprogo.be/api/v1

**8. Configuration DNS OVH (optionnel)**

.. code-block:: yaml

   - name: Install Python OVH module
     pip:
       name: ovh
       state: present

   - name: Copy OVH DNS configuration script
     copy:
       src: configure-ovh-dns.py
       dest: /tmp/configure-ovh-dns.py
       mode: '0755'

   - name: Configure OVH DNS for frontend
     command: python3 /tmp/configure-ovh-dns.py
     environment:
       DOMAIN: "{{ frontend_domain }}"
       TARGET_IP: "{{ ansible_host }}"
       OVH_ENDPOINT: "{{ lookup('env', 'OVH_ENDPOINT') }}"
       OVH_APPLICATION_KEY: "{{ lookup('env', 'OVH_APPLICATION_KEY') }}"
       OVH_APPLICATION_SECRET: "{{ lookup('env', 'OVH_APPLICATION_SECRET') }}"
       OVH_CONSUMER_KEY: "{{ lookup('env', 'OVH_CONSUMER_KEY') }}"

**Script configure-ovh-dns.py** :

- Crée/update A record pour frontend (koprogo.be → VPS_IP)
- Crée/update A record pour API (api.koprogo.be → VPS_IP)
- Utilise API OVH pour modification DNS

**9. Déploiement Docker Compose**

.. code-block:: yaml

   - name: Make gitops-deploy.sh executable
     file:
       path: "{{ koprogo_dir }}/deploy/production/gitops-deploy.sh"
       mode: '0755'

   - name: Deploy KoproGo using gitops-deploy.sh
     command: "{{ koprogo_dir }}/deploy/production/gitops-deploy.sh deploy"
     environment:
       REPO_DIR: "{{ koprogo_dir }}"

**Ce que fait deploy** :

.. code-block:: bash

   cd /home/koprogo/koprogo/deploy/production
   docker compose pull    # Pull images GitHub Container Registry
   docker compose up -d   # Start services (Traefik, Backend, Frontend, PostgreSQL)

**10. Configuration GitOps (systemd)**

.. code-block:: yaml

   - name: Install GitOps systemd service
     template:
       src: koprogo-gitops.service.j2
       dest: /etc/systemd/system/koprogo-gitops.service
       owner: root
       group: root
       mode: '0644'

   - name: Enable and start GitOps service
     systemd:
       name: koprogo-gitops
       enabled: yes
       state: started

**Template koprogo-gitops.service.j2** :

.. code-block:: ini

   [Unit]
   Description=KoproGo GitOps Auto-Update Service
   After=docker.service
   Requires=docker.service

   [Service]
   Type=simple
   User=koprogo
   WorkingDirectory=/home/koprogo/koprogo
   ExecStart=/home/koprogo/koprogo/deploy/production/gitops-deploy.sh watch
   Restart=always
   RestartSec=10
   StandardOutput=append:/var/log/koprogo-gitops.log

   # Security
   NoNewPrivileges=true
   PrivateTmp=true

   [Install]
   WantedBy=multi-user.target

**Workflow GitOps** :

.. code-block:: bash

   # Service systemd appelle gitops-deploy.sh watch
   # Ce script:
   1. Vérifie Git toutes les 3 minutes
   2. Si nouveau commit sur main:
      - git pull
      - docker compose down
      - docker compose pull
      - docker compose up -d
      - Health check API
   3. Si health check échoue:
      - Rollback (git reset --hard ORIG_HEAD)
      - docker compose up -d (ancienne version)

**11. Configuration backups (cron)**

.. code-block:: yaml

   - name: Create backup script
     template:
       src: backup.sh.j2
       dest: "{{ koprogo_dir }}/scripts/backup.sh"
       owner: "{{ koprogo_user }}"
       mode: '0755'

   - name: Setup daily backup cron job
     cron:
       name: "KoproGo Daily Backup"
       minute: "0"
       hour: "2"
       job: "{{ koprogo_dir }}/scripts/backup.sh >> /var/log/koprogo-backup.log 2>&1"
       user: "{{ koprogo_user }}"

**Template backup.sh.j2** :

.. code-block:: bash

   #!/bin/bash
   # Backup PostgreSQL quotidien (2h du matin)

   BACKUP_DIR=/home/koprogo/backups
   DATE=$(date +%Y%m%d)

   mkdir -p $BACKUP_DIR

   # Backup PostgreSQL
   docker exec koprogo-postgres pg_dumpall -U postgres \
       > $BACKUP_DIR/postgres_$DATE.sql

   # Rotation (garde 7 derniers jours)
   find $BACKUP_DIR -name "postgres_*.sql" -mtime +7 -delete

**12. Monitoring (health checks)**

.. code-block:: yaml

   - name: Create health check script
     template:
       src: health-check.sh.j2
       dest: "{{ koprogo_dir }}/scripts/health-check.sh"
       owner: "{{ koprogo_user }}"
       mode: '0755'

   - name: Setup health check cron job (every 5 min)
     cron:
       name: "KoproGo Health Check"
       minute: "*/5"
       job: "{{ koprogo_dir }}/scripts/health-check.sh >> /var/log/koprogo-health.log 2>&1"
       user: "{{ koprogo_user }}"

**Template health-check.sh.j2** :

.. code-block:: bash

   #!/bin/bash
   # Health check API toutes les 5 minutes

   API_URL="https://{{ api_domain }}/api/v1/health"

   if ! curl -f -s $API_URL > /dev/null; then
       echo "$(date): ALERT - API down!"
       # Optionnel: envoyer email/SMS
   fi

**13. Vérifications finales**

.. code-block:: yaml

   - name: Wait for services to start
     pause:
       seconds: 10

   - name: Check API health (public HTTPS endpoint)
     uri:
       url: "https://{{ api_domain }}/api/v1/health"
       method: GET
       status_code: 200
       validate_certs: no
     register: health_check
     retries: 10
     delay: 10
     until: health_check.status == 200

   - name: Display deployment info
     debug:
       msg:
         - "KoproGo déployé avec succès !"
         - "URL Frontend: https://{{ frontend_domain }}"
         - "URL API: https://{{ api_domain }}/api/v1"
         - "GitOps Service: systemctl status koprogo-gitops"

Templates Jinja2
----------------

env-production.j2
^^^^^^^^^^^^^^^^^

Template ``.env`` production avec variables dynamiques.

**Variables auto-générées** :

.. code-block:: jinja

   # Database password (aléatoire 32 chars)
   POSTGRES_PASSWORD={{ postgres_password | default(lookup('password', '/dev/null chars=ascii_letters,digits length=32')) }}

   # JWT secret (aléatoire 64 chars)
   JWT_SECRET={{ jwt_secret | default(lookup('password', '/dev/null chars=ascii_letters,digits length=64')) }}

**Conditionnels** :

.. code-block:: jinja

   # Si domaine fourni: HTTPS
   {% if api_domain and api_domain != '' %}
   API_DOMAIN={{ api_domain }}
   PUBLIC_API_URL=https://{{ api_domain }}/api/v1
   {% else %}
   # Sinon: HTTP + IP
   API_DOMAIN={{ ansible_host }}
   PUBLIC_API_URL=http://{{ ansible_host }}:8080/api/v1
   {% endif %}

koprogo-gitops.service.j2
^^^^^^^^^^^^^^^^^^^^^^^^^^

Template service systemd GitOps.

.. code-block:: jinja

   [Unit]
   Description=KoproGo GitOps Auto-Update Service
   After=docker.service
   Requires=docker.service

   [Service]
   User={{ koprogo_user }}
   WorkingDirectory={{ koprogo_dir }}
   ExecStart={{ koprogo_dir }}/deploy/production/gitops-deploy.sh watch
   Restart=always
   RestartSec=10

backup.sh.j2
^^^^^^^^^^^^

Template script backup PostgreSQL.

.. code-block:: jinja

   #!/bin/bash
   BACKUP_DIR={{ koprogo_dir }}/backups
   DATE=$(date +%Y%m%d)

   docker exec koprogo-postgres pg_dumpall -U postgres \
       > $BACKUP_DIR/postgres_$DATE.sql

   find $BACKUP_DIR -name "postgres_*.sql" -mtime +7 -delete

health-check.sh.j2
^^^^^^^^^^^^^^^^^^

Template script health check API.

.. code-block:: jinja

   #!/bin/bash
   API_URL="https://{{ api_domain }}/api/v1/health"

   if ! curl -f -s $API_URL > /dev/null; then
       echo "$(date): API down!"
   fi

Commandes Ansible
-----------------

Ping
^^^^

Tester connexion SSH.

.. code-block:: bash

   ansible koprogo -i inventory.ini -m ping

**Output** :

.. code-block:: text

   51.178.12.34 | SUCCESS => {
       "ping": "pong"
   }

Check Mode (Dry-Run)
^^^^^^^^^^^^^^^^^^^^

Prévisualiser changements sans les appliquer.

.. code-block:: bash

   ansible-playbook -i inventory.ini playbook.yml --check

Apply Playbook
^^^^^^^^^^^^^^

Exécuter playbook complet.

.. code-block:: bash

   ansible-playbook -i inventory.ini playbook.yml

**Durée** : ~5-10 minutes.

**Output** :

.. code-block:: text

   PLAY [Configure KoproGo VPS] ************************************

   TASK [Update apt cache] *****************************************
   ok: [51.178.12.34]

   TASK [Install Docker] *******************************************
   changed: [51.178.12.34]

   ...

   PLAY RECAP ******************************************************
   51.178.12.34               : ok=45   changed=12   unreachable=0    failed=0

Apply avec verbose
^^^^^^^^^^^^^^^^^^

Afficher détails exécution.

.. code-block:: bash

   ansible-playbook -i inventory.ini playbook.yml -v   # Verbose
   ansible-playbook -i inventory.ini playbook.yml -vv  # Plus verbose
   ansible-playbook -i inventory.ini playbook.yml -vvv # Debug complet

Tags
^^^^

Exécuter uniquement certaines tâches.

.. code-block:: bash

   # Exemple avec tags (si définis dans playbook)
   ansible-playbook -i inventory.ini playbook.yml --tags "docker,firewall"

Limit
^^^^^

Exécuter sur serveurs spécifiques.

.. code-block:: bash

   ansible-playbook -i inventory.ini playbook.yml --limit 51.178.12.34

Idempotence
-----------

**Principe** : Ansible est idempotent. Réexécuter playbook n'applique que changements nécessaires.

**Exemple** :

.. code-block:: bash

   # 1ère exécution
   ansible-playbook -i inventory.ini playbook.yml
   # Output: ok=10 changed=10

   # 2ème exécution
   ansible-playbook -i inventory.ini playbook.yml
   # Output: ok=10 changed=0  (aucun changement)

**Modules idempotents** :

- ``apt`` : Installe uniquement si absent
- ``file`` : Crée uniquement si absent
- ``systemd`` : Start uniquement si stopped
- ``template`` : Update uniquement si contenu différent

Troubleshooting
---------------

Connexion SSH Failed
^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   UNREACHABLE! => {"msg": "Failed to connect to the host via ssh"}

**Solutions** :

.. code-block:: bash

   # Tester SSH manuellement
   ssh ubuntu@51.178.12.34

   # Vérifier clé SSH
   ls ~/.ssh/id_rsa

   # Vérifier inventory.ini
   cat inventory.ini

   # Vérifier firewall VPS (port 22 ouvert?)
   ansible koprogo -i inventory.ini -m shell -a "sudo ufw status"

Permission Denied
^^^^^^^^^^^^^^^^^

.. code-block:: text

   FAILED! => {"msg": "Missing sudo password"}

**Solutions** :

.. code-block:: bash

   # Demander password sudo
   ansible-playbook -i inventory.ini playbook.yml --ask-become-pass

   # Ou configurer sudo sans password (sudoers)
   echo "ubuntu ALL=(ALL) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/ubuntu

Docker Installation Failed
^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   FAILED! => {"msg": "Unable to install docker-ce"}

**Solutions** :

.. code-block:: bash

   # Vérifier Ubuntu version
   ansible koprogo -i inventory.ini -m shell -a "lsb_release -a"

   # Vérifier architecture
   ansible koprogo -i inventory.ini -m shell -a "uname -m"
   # Doit être x86_64

Health Check Failed
^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   FAILED! => {"msg": "API health check failed after 10 retries"}

**Solutions** :

.. code-block:: bash

   # SSH vers VPS
   ssh ubuntu@51.178.12.34

   # Vérifier logs Docker
   cd /home/koprogo/koprogo/deploy/production
   docker compose logs backend

   # Vérifier containers running
   docker compose ps

   # Vérifier DNS pointent vers VPS
   dig koprogo.be
   dig api.koprogo.be

Gestion Secrets
---------------

Ansible Vault
^^^^^^^^^^^^^

Chiffrer fichiers sensibles (inventory, vars).

.. code-block:: bash

   # Créer vault (inventaire chiffré)
   ansible-vault create inventory.vault.ini

   # Éditer vault
   ansible-vault edit inventory.vault.ini

   # Chiffrer fichier existant
   ansible-vault encrypt inventory.ini

   # Déchiffrer
   ansible-vault decrypt inventory.ini

**Utiliser vault** :

.. code-block:: bash

   # Demander password vault
   ansible-playbook -i inventory.vault.ini playbook.yml --ask-vault-pass

   # Ou via fichier password
   echo "mypassword" > .vault_pass
   chmod 600 .vault_pass
   ansible-playbook -i inventory.vault.ini playbook.yml --vault-password-file .vault_pass

Variables chiffrées
^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Créer fichier vars chiffré
   ansible-vault create vars/secrets.yml

**secrets.yml** :

.. code-block:: yaml

   ---
   postgres_password: super_secret_password
   jwt_secret: super_secret_jwt
   ovh_consumer_key: secret_key

**Utiliser dans playbook** :

.. code-block:: yaml

   - name: Configure KoproGo
     hosts: koprogo
     vars_files:
       - vars/secrets.yml
     tasks:
       - name: Create .env
         template:
           src: env-production.j2
           dest: /home/koprogo/.env

CI/CD Ansible
-------------

GitHub Actions Example
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: yaml

   # .github/workflows/ansible.yml
   name: Ansible Deploy

   on:
     push:
       branches: [main]
       paths:
         - 'infrastructure/ansible/**'

   jobs:
     deploy:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3

         - name: Install Ansible
           run: sudo apt-get install ansible

         - name: Create inventory
           run: |
             cat > infrastructure/ansible/inventory.ini <<EOF
             [koprogo]
             ${{ secrets.VPS_IP }} ansible_user=ubuntu ansible_ssh_private_key_file=./ssh_key
             EOF

         - name: Add SSH key
           run: |
             echo "${{ secrets.SSH_PRIVATE_KEY }}" > ./ssh_key
             chmod 600 ./ssh_key

         - name: Run Ansible playbook
           working-directory: infrastructure/ansible
           run: ansible-playbook -i inventory.ini playbook.yml

**Secrets GitHub** :

.. code-block:: bash

   VPS_IP (51.178.12.34)
   SSH_PRIVATE_KEY (contenu ~/.ssh/id_rsa)
   OVH_APPLICATION_KEY
   OVH_APPLICATION_SECRET
   OVH_CONSUMER_KEY

Best Practices
--------------

1. **Toujours tester en check mode** :

   .. code-block:: bash

      ansible-playbook -i inventory.ini playbook.yml --check

2. **Utiliser tags pour tâches réutilisables** :

   .. code-block:: yaml

      - name: Install Docker
        apt:
          name: docker-ce
        tags: [docker]

3. **Handlers pour services** :

   .. code-block:: yaml

      handlers:
        - name: restart docker
          systemd:
            name: docker
            state: restarted

      tasks:
        - name: Update Docker config
          template:
            src: daemon.json.j2
            dest: /etc/docker/daemon.json
          notify: restart docker

4. **Variables dans fichiers séparés** :

   .. code-block:: text

      vars/
      ├── common.yml
      ├── production.yml
      └── secrets.yml (vault)

5. **Roles pour réutilisabilité** :

   .. code-block:: text

      roles/
      ├── docker/
      │   ├── tasks/main.yml
      │   └── handlers/main.yml
      └── firewall/
          └── tasks/main.yml

Références
----------

- Ansible Docs : https://docs.ansible.com/
- Ansible Best Practices : https://docs.ansible.com/ansible/latest/user_guide/playbooks_best_practices.html
- Ansible Vault : https://docs.ansible.com/ansible/latest/user_guide/vault.html
- Jinja2 Templating : https://jinja.palletsprojects.com/

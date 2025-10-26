# Troubleshooting Déploiement

Résolution des problèmes courants rencontrés lors du déploiement de KoproGo.

---

## 🔍 Diagnostic Rapide

Avant de chercher la solution, identifier la phase qui échoue :

| Phase | Outil | Symptôme |
|-------|-------|----------|
| **Terraform** | `terraform apply` | Erreur création VPS, région, permissions |
| **Ansible** | `ansible-playbook` | Erreur SSH, Docker, configuration |
| **Docker** | `docker compose up` | Erreur images, network, volumes |
| **Traefik** | Logs Traefik | Erreur SSL, certificat, routing |
| **GitOps** | `systemctl status` | Erreur auto-update, permissions Git |

---

## 1. Problèmes Terraform

### 1.1 "No suitable endpoint could be found"

**Symptôme** :
```
Error: No suitable endpoint could be found in the service catalog

  with provider["registry.terraform.io/terraform-provider-openstack/openstack"]
```

**Cause** : Région incorrecte ou non compatible avec votre fichier OpenRC

**Solution** :
```bash
# 1. Télécharger le fichier OpenRC depuis OVH Manager
#    Public Cloud → Users & Roles → ... → Download OpenRC file

# 2. Ouvrir le fichier et identifier la région EXACTE
grep OS_REGION_NAME openrc.sh
# export OS_REGION_NAME="GRA9"

# 3. Utiliser cette région dans make setup-infra
make setup-infra
# Quand demandé: Région OpenRC → GRA9 (exactement comme dans le fichier)
```

**⚠️ Ne PAS deviner la région !** Toujours utiliser celle du fichier OpenRC.

---

### 1.2 "Insufficient permissions" / "Forbidden"

**Symptôme** :
```
Error creating openstack_compute_instance_v2: Forbidden

Error: error creating OpenStack compute instance: Expected HTTP response code [201 202]
```

**Cause** : Utilisateur OpenStack sans le rôle **Administrator**

**Solution** :
```bash
# 1. OVH Manager → Public Cloud → Users & Roles
# 2. Supprimer l'utilisateur actuel
# 3. Créer un NOUVEL utilisateur OpenStack
# 4. Cocher TOUS les rôles, SURTOUT "Administrator" (CRITIQUE !)
# 5. Noter le nouveau OS_USERNAME et OS_PASSWORD
# 6. Relancer: make setup-infra et choisir "2) Reconfigurer"
```

**Liste complète des rôles requis** :
- ☑ **Administrator** (CRITIQUE)
- ☑ Compute Operator
- ☑ Network Operator
- ☑ Network Security Operator
- ☑ Image Operator
- ☑ Volume Operator
- ☑ ObjectStore Operator
- ☑ LoadBalancer Operator
- ☑ Backup Operator
- ☑ Infrastructure Supervisor
- ☑ KeyManager Operator
- ☑ KeyManager Read

---

### 1.3 "One of 'auth_url' or 'cloud' must be specified"

**Symptôme** :
```
Error: One of 'auth_url' or 'cloud' must be specified

  with provider["registry.terraform.io/terraform-provider-openstack/openstack"].ovh
```

**Cause** : Variables d'environnement OpenStack non chargées dans le shell

**Solution** :
```bash
cd infrastructure/terraform

# ✅ CORRECT - Utiliser "source" pour charger les variables
source ./load-env.sh
terraform plan

# ✅ CORRECT - Raccourci avec "."
. ./load-env.sh
terraform apply

# ❌ FAUX - ./load-env.sh crée un sous-shell, variables perdues
./load-env.sh
terraform plan  # ❌ Erreur
```

**Détection automatique** :
Le script `load-env.sh` détecte maintenant l'erreur et affiche :
```
❌ Erreur: Ce script doit être sourcé, pas exécuté directement!

Utilisation correcte:
  source ./load-env.sh
  # ou
  . ./load-env.sh
```

---

### 1.4 "Variables not set" / "Missing required argument"

**Symptôme** :
```
Error: Missing required argument

  on main.tf line 15, in provider "openstack":
  15:   project_id = var.project_id
```

**Cause** : Fichier `.env` manquant ou incomplet

**Solution** :
```bash
cd infrastructure/terraform

# Vérifier que .env existe
ls -la .env

# Si absent, relancer setup
cd ../..
make setup-infra

# Si présent mais incomplet, éditer
nano infrastructure/terraform/.env

# Variables REQUISES:
# OS_PROJECT_ID=xxxxx
# OS_USERNAME=user-xxxxx
# OS_PASSWORD=xxxxx
# OS_REGION_NAME=GRA9
```

---

## 2. Problèmes Ansible

### 2.1 "SSH connection failed" / "Connection timed out"

**Symptôme** :
```
fatal: [koprogo-vps]: UNREACHABLE! => {
    "msg": "Failed to connect to the host via ssh: ssh: connect to host 51.210.XXX.XXX port 22: Connection timed out"
}
```

**Cause** : VPS pas encore prêt ou clé SSH incorrecte

**Solution** :
```bash
# 1. Attendre 1-2 minutes après terraform apply
sleep 120

# 2. Tester connexion SSH manuelle
VPS_IP=$(cd infrastructure/terraform && terraform output -raw vps_ip)
ssh -o StrictHostKeyChecking=no ubuntu@$VPS_IP

# 3. Si connexion échoue, vérifier clé SSH
ls -la ~/.ssh/id_rsa.pub

# Si clé absente, la générer
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"

# 4. Relancer Ansible
cd infrastructure/ansible
ansible-playbook -i inventory.ini playbook.yml
```

---

### 2.2 "Failed to set permissions" (become_user error)

**Symptôme** :
```
Failed to set permissions on the temporary files Ansible needs to create
chmod: invalid mode: 'A+user:koprogo:rx:allow'
```

**Cause** : Problème d'ACL avec Ansible 2.16+ sur Ubuntu

**Solution** :
Ce problème est **déjà corrigé** dans le playbook avec `become_method: su`.

Si l'erreur persiste :
```bash
# Vérifier version Ansible
ansible --version

# Si < 2.16, mettre à jour
pip install --upgrade ansible

# Vérifier playbook.yml contient:
grep "become_method" infrastructure/ansible/playbook.yml
# Devrait afficher: become_method: su
```

---

### 2.3 "Health check failed" pendant Ansible

**Symptôme** :
```
TASK [Check API health (public HTTPS endpoint)] ****************************
FAILED - RETRYING: [koprogo-vps]: Check API health (10 retries left).
fatal: [koprogo-vps]: FAILED! => {"status": -1, "msg": "SSL certificate problem"}
```

**Causes possibles** :
1. Certificat Let's Encrypt pas encore généré (DNS pas propagé)
2. Services Docker pas encore prêts
3. Configuration Traefik incorrecte

**Solution** :
```bash
# 1. Vérifier que DNS pointe vers VPS
VPS_IP=$(cd infrastructure/terraform && terraform output -raw vps_ip)
dig +short api.votre-domaine.com
# Devrait afficher: 51.210.XXX.XXX (IP du VPS)

# 2. Si DNS incorrect, attendre propagation (5-60 min)
# Vérifier sur DNS OVH:
dig +short api.votre-domaine.com @dns200.anycast.me

# 3. Se connecter au VPS et vérifier services
ssh ubuntu@$VPS_IP
sudo su - koprogo
cd ~/koprogo/deploy/production
docker compose ps

# 4. Vérifier logs Traefik pour certificat SSL
docker compose logs traefik | grep -i "cert"
docker compose logs traefik | grep -i "acme"

# 5. Tester health check manuellement
curl -k https://api.votre-domaine.com/api/v1/health
# (-k = ignore SSL errors si cert pas encore généré)

# 6. Si API répond, le déploiement a RÉUSSI
# Le health check Ansible a juste timeout, c'est OK
```

**Note** : Le playbook Ansible fait 10 tentatives (100 secondes total). Si le certificat SSL prend plus de temps, le health check peut échouer **mais le déploiement est quand même réussi**.

---

## 3. Problèmes DNS

### 3.1 "Propagation lente"

**Symptôme** : Le domaine ne pointe pas vers le VPS immédiatement

**Cause** : Propagation DNS normale (1-60 minutes)

**Solution** :
```bash
# Vérifier configuration DNS (peut montrer ancienne IP pendant propagation)
nslookup votre-domaine.com

# Forcer requête vers DNS OVH (plus rapide)
nslookup votre-domaine.com dns200.anycast.me

# Vérifier avec dig
dig +short api.votre-domaine.com

# Attendre 5-10 minutes et retester
```

**Timeline typique** :
- T+0: Terraform crée enregistrements DNS
- T+2min: DNS OVH à jour
- T+5-10min: DNS publics à jour (Google, Cloudflare)
- T+30-60min: Tous les DNS à jour globalement

---

### 3.2 "DNS pointe vers mauvaise IP"

**Symptôme** : `dig api.domain.com` retourne une IP différente du VPS

**Cause** : Configuration DNS manuelle incorrecte ou ancien déploiement

**Solution** :
```bash
# 1. Vérifier IP du VPS actuel
cd infrastructure/terraform
terraform output vps_ip

# 2. Vérifier enregistrements DNS OVH
# OVH Manager → Domain → DNS Zone → votre-domaine.com

# 3. Vérifier que les enregistrements A pointent vers la bonne IP:
# api.votre-domaine.com → 51.210.XXX.XXX
# app.votre-domaine.com → 51.210.XXX.XXX

# 4. Si incorrect, corriger manuellement ou relancer Terraform
cd infrastructure/terraform
source ./load-env.sh
terraform apply -refresh-only
```

---

## 4. Problèmes Docker

### 4.1 "Services ne démarrent pas"

**Symptôme** :
```bash
docker compose ps
# NAME    STATUS
# backend  Exited (1)
# frontend Exited (1)
```

**Solution** :
```bash
# Voir logs d'erreur
docker compose logs backend
docker compose logs frontend
docker compose logs postgres

# Erreurs courantes:

# → "connection refused" (PostgreSQL)
# Solution: Attendre que postgres soit prêt
docker compose restart backend

# → "permission denied" (volumes)
# Solution: Fix permissions
sudo chown -R koprogo:koprogo ~/koprogo

# → "image not found"
# Solution: Pull images manuellement
docker compose pull
docker compose up -d
```

---

### 4.2 "Out of disk space"

**Symptôme** :
```
Error response from daemon: write /var/lib/docker/...: no space left on device
```

**Solution** :
```bash
# Vérifier espace disque
df -h

# Nettoyer images Docker inutilisées
docker system prune -a
docker volume prune

# Nettoyer logs
sudo journalctl --vacuum-time=7d

# Nettoyer vieux backups
find ~/koprogo/backups -name "*.sql.gz" -mtime +30 -delete

# Si toujours plein, upgrader VPS
# OVH Manager → Public Cloud → Resize Instance
```

---

## 5. Problèmes Traefik / SSL

### 5.1 "Certificat SSL pas généré"

**Symptôme** : HTTPS ne fonctionne pas, erreur de certificat

**Cause** : DNS pas encore propagé, domaine incorrect, ou Let's Encrypt rate limit

**Solution** :
```bash
# 1. Vérifier que DNS pointe vers VPS
nslookup api.votre-domaine.com

# 2. Vérifier logs Traefik
cd ~/koprogo/deploy/production
docker compose logs traefik | grep -i "acme"
docker compose logs traefik | grep -i "certificate"

# Erreurs possibles:
# → "NXDOMAIN" : DNS pas encore propagé, attendre 10-30 min
# → "rate limit" : Trop de tentatives, attendre 1 heure
# → "CAA record" : Vérifier records CAA du domaine

# 3. Vérifier fichier acme.json
ls -la letsencrypt/acme.json
cat letsencrypt/acme.json | jq .

# Si vide ou erreurs, supprimer et relancer
rm letsencrypt/acme.json
touch letsencrypt/acme.json
chmod 600 letsencrypt/acme.json
docker compose restart traefik

# 4. Attendre 2-5 minutes et vérifier
openssl s_client -connect api.votre-domaine.com:443 -servername api.votre-domaine.com
```

---

### 5.2 "Redirect loop" / "Too many redirects"

**Symptôme** : Navigateur affiche "ERR_TOO_MANY_REDIRECTS"

**Cause** : Configuration Traefik incorrecte (redirect HTTP → HTTPS en boucle)

**Solution** :
```bash
# Vérifier configuration Traefik
cat deploy/production/traefik.yml

# Vérifier labels Docker
docker inspect koprogo-backend | grep -A 10 "Labels"

# Désactiver temporairement redirect pour debug
# Éditer traefik.yml:
# entryPoints:
#   web:
#     address: ":80"
#     # http:  # Commenter cette section
#     #   redirections:
#     #     entryPoint:
#     #       to: websecure
#     #       scheme: https

# Redémarrer
docker compose restart traefik
```

---

## 6. Problèmes GitOps

### 6.1 "Permission denied" sur .git/

**Symptôme** :
```
error: cannot open .git/FETCH_HEAD: Permission denied
error: cannot open .git/index: Permission denied
```

**Cause** : Fichiers `.git/` appartiennent à `root` au lieu de `koprogo`

**Solution** :
```bash
# Fix manuel
sudo chown -R koprogo:koprogo /home/koprogo/koprogo/.git

# Vérifier permissions
ls -la /home/koprogo/koprogo/.git/

# Le playbook Ansible fixe automatiquement ces permissions
# après chaque déploiement. Si le problème persiste,
# relancer Ansible:
cd infrastructure/ansible
ansible-playbook -i inventory.ini playbook.yml --tags "gitops"
```

---

### 6.2 "Service GitOps ne démarre pas"

**Symptôme** :
```bash
sudo systemctl status koprogo-gitops.service
# Active: failed
```

**Solution** :
```bash
# Voir logs d'erreur
sudo journalctl -u koprogo-gitops.service -n 50

# Erreurs possibles:

# → "gitops-deploy.sh: No such file"
# Solution: Vérifier script existe
ls -la /home/koprogo/koprogo/deploy/production/gitops-deploy.sh

# → "Permission denied"
# Solution: Rendre exécutable
chmod +x /home/koprogo/koprogo/deploy/production/gitops-deploy.sh

# → "docker: command not found"
# Solution: Réinstaller Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Redémarrer service
sudo systemctl restart koprogo-gitops.service
```

---

### 6.3 "GitOps ne met pas à jour"

**Symptôme** : Nouveaux commits poussés sur GitHub mais app ne se met pas à jour

**Solution** :
```bash
# 1. Vérifier que service tourne
sudo systemctl status koprogo-gitops.service
# Devrait être: Active: active (running)

# 2. Voir logs en temps réel
sudo journalctl -u koprogo-gitops.service -f

# 3. Vérifier fréquence de vérification
# Devrait montrer "Checking for updates" toutes les 3 minutes

# 4. Tester update manuel
sudo /home/koprogo/koprogo/deploy/production/gitops-deploy.sh deploy

# 5. Si erreur Git, vérifier repository
cd /home/koprogo/koprogo
git status
git remote -v
git fetch origin

# 6. Si détaché, re-checkout main
git checkout main
git pull origin main
```

---

## 7. Commandes de Debug Générales

### Vérifier tous les services

```bash
# Sur le VPS
ssh ubuntu@VPS_IP

# Passer en utilisateur koprogo
sudo su - koprogo
cd ~/koprogo/deploy/production

# Services Docker
docker compose ps

# Logs tous services
docker compose logs --tail=50

# Logs service spécifique
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f postgres
docker compose logs -f traefik

# Service GitOps
sudo systemctl status koprogo-gitops.service
sudo journalctl -u koprogo-gitops.service -n 100

# Espace disque
df -h

# Mémoire
free -h

# Processus
htop
```

### Restart complet

```bash
# Restart services Docker
cd ~/koprogo/deploy/production
docker compose restart

# Ou rebuild complet
docker compose down
docker compose pull
docker compose up -d

# Restart service GitOps
sudo systemctl restart koprogo-gitops.service
```

### Health checks manuels

```bash
# API health
curl https://api.votre-domaine.com/api/v1/health

# Avec détails
curl -v https://api.votre-domaine.com/api/v1/health

# Ignorer SSL (si certificat pas encore généré)
curl -k https://api.votre-domaine.com/api/v1/health

# Frontend
curl https://app.votre-domaine.com

# PostgreSQL (depuis VPS)
docker compose exec postgres psql -U koprogo -d koprogo_db -c "SELECT version();"
```

---

## 📚 Ressources Complémentaires

- **Lessons Learned** : [`../../infrastructure/LESSONS-LEARNED.md`](../../infrastructure/LESSONS-LEARNED.md) - Historique complet des problèmes rencontrés
- **Documentation OVH** : https://help.ovhcloud.com/
- **Terraform OpenStack** : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/latest/docs
- **Ansible Docs** : https://docs.ansible.com/
- **Docker Docs** : https://docs.docker.com/
- **Traefik Docs** : https://doc.traefik.io/

---

## 🆘 Support

Si aucune solution ci-dessus ne fonctionne :

1. Consulter [Lessons Learned](../../infrastructure/LESSONS-LEARNED.md) pour plus de détails
2. Créer une GitHub Issue : https://github.com/gilmry/koprogo/issues
3. Inclure dans l'issue :
   - Commande exécutée
   - Erreur complète (logs)
   - Terraform version, Ansible version
   - Région OVH utilisée

---

**Dernière mise à jour** : Octobre 2025

**KoproGo ASBL** 🚀

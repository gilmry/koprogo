# Ansible - Déploiement automatisé KoproGo

Déploiement automatisé de KoproGo sur VPS OVH avec Ansible.

## 🚀 Quick Start (3 commandes)

```bash
# 1. Créer inventory.ini depuis Terraform
./setup-inventory.sh

# 2. Tester la connexion
ansible koprogo -i inventory.ini -m ping

# 3. Déployer
ansible-playbook -i inventory.ini playbook.yml
```

## 📋 Ce qui est déployé

Le playbook Ansible automatise entièrement la configuration du VPS:

### Infrastructure
- ✅ Mise à jour système Ubuntu 22.04
- ✅ Installation Docker + Docker Compose
- ✅ Configuration firewall UFW (SSH, HTTP, HTTPS)
- ✅ Installation Fail2ban (sécurité SSH)
- ✅ Création utilisateur `koprogo` (non-root)

### Application
- ✅ Clone du repository Git
- ✅ Configuration `.env` depuis template
- ✅ Déploiement Docker Compose
- ✅ Backend Rust (Actix-web)
- ✅ PostgreSQL 15
- ✅ Frontend Astro/Svelte (optionnel)

### Automatisation
- ✅ **GitOps**: Auto-update quotidien (3h du matin)
- ✅ **Backups**: Backup PostgreSQL quotidien (2h du matin)
- ✅ **Monitoring**: Health check toutes les 5 minutes

## 📁 Structure

```
infrastructure/ansible/
├── README.md                 # Ce fichier
├── TESTING.md               # Guide complet de test
├── playbook.yml             # Playbook principal
├── inventory.ini.example    # Template d'inventaire
├── inventory.ini           # Votre inventaire (créé par setup-inventory.sh)
│
├── Scripts helper
├── setup-inventory.sh       # Créer inventory.ini depuis Terraform
├── quick-test.sh           # Tests rapides
│
└── templates/               # Templates Jinja2
    ├── env.j2              # Configuration .env de l'app
    ├── auto-update.sh.j2   # Script GitOps
    ├── backup.sh.j2        # Script de backup
    └── health-check.sh.j2  # Script de monitoring
```

## 🎯 Prérequis

### 1. VPS déployé avec Terraform

```bash
cd ../terraform
source ./load-env.sh
terraform apply
terraform output vps_ip  # Noter l'IP
```

### 2. Ansible installé

```bash
# Ubuntu/Debian
sudo apt install -y ansible

# macOS
brew install ansible

# Ou avec pip
pip3 install ansible
```

### 3. Clé SSH configurée

Votre clé SSH doit être sur le VPS (déjà fait par Terraform).

## 🔧 Configuration

### Option 1: Automatique (Recommandé)

```bash
# Génère inventory.ini depuis Terraform outputs
./setup-inventory.sh
```

Le script:
- Récupère l'IP depuis `terraform output`
- Crée `inventory.ini` automatiquement
- Teste la connexion SSH et Ansible

### Option 2: Manuelle

```bash
# Copier le template
cp inventory.ini.example inventory.ini

# Éditer avec votre IP
nano inventory.ini
```

Remplacez `YOUR_VPS_IP`:

```ini
[koprogo]
koprogo-vps ansible_host=X.X.X.X ansible_user=ubuntu ansible_ssh_private_key_file=~/.ssh/id_rsa

[koprogo:vars]
# Optionnel
# domain=koprogo.example.com
# acme_email=admin@example.com
```

## 🧪 Tests

### Test rapide (sans déployer)

```bash
# Script de test complet
./quick-test.sh
```

Le script teste:
- Connexion Ansible (ping)
- Uptime et ressources VPS
- Docker installé?
- Services KoproGo actifs?
- API accessible?

### Tests manuels

```bash
# Test de connexion
ansible koprogo -i inventory.ini -m ping

# Info système
ansible koprogo -i inventory.ini -m setup

# Commande shell
ansible koprogo -i inventory.ini -m shell -a "uptime"

# Dry-run du playbook
ansible-playbook -i inventory.ini playbook.yml --check
```

## 🚀 Déploiement

### Déploiement complet

```bash
# Exécuter le playbook (5-10 minutes)
ansible-playbook -i inventory.ini playbook.yml
```

### Mode verbeux (debug)

```bash
# Voir plus de détails
ansible-playbook -i inventory.ini playbook.yml -v

# Mode très verbeux
ansible-playbook -i inventory.ini playbook.yml -vvv
```

### Avec variables personnalisées

```bash
# Définir un domaine
ansible-playbook -i inventory.ini playbook.yml \
  -e "domain=koprogo.example.com" \
  -e "acme_email=admin@example.com"

# Exemple : configuration S3 externe
ansible-playbook -i inventory.ini playbook.yml \
  -e "storage_provider=s3" \
  -e "s3_bucket=mon-bucket" \
  -e "s3_endpoint=https://s3.eu-west-1.amazonaws.com" \
  -e "s3_access_key=AKIA..." \
  -e "s3_secret_key=********" \
  -e "enable_minio_bootstrap=false"

# Exemple : protection de l'endpoint /metrics
ansible-playbook -i inventory.ini playbook.yml \
  -e "metrics_auth_token=token-super-secret"

# Exemple : configuration SMTP pour notifications GDPR
ansible-playbook -i inventory.ini playbook.yml \
  -e "smtp_enabled=true" \
  -e "smtp_host=smtp.gmail.com" \
  -e "smtp_port=587" \
  -e "smtp_username=your-email@example.com" \
  -e "smtp_password=your-app-password" \
  -e "smtp_from_email=noreply@koprogo.com" \
  -e "smtp_from_name=KoproGo"
```

## ✅ Vérification post-déploiement

### 1. Health check API

```bash
VPS_IP=$(grep ansible_host inventory.ini | awk '{print $2}' | cut -d'=' -f2)
curl http://$VPS_IP:8080/api/v1/health

# Devrait retourner: {"status":"ok"}
```

### 2. Services Docker

```bash
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose ps'"
```

### 3. Logs

```bash
# Tous les services
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose logs'"

# Backend uniquement
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose logs backend'"
```

### 4. Cron jobs

```bash
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'crontab -l'"
```

Devrait afficher:
- Auto-update: Tous les jours à 3h
- Backup: Tous les jours à 2h
- Health check: Toutes les 5 min

## 🔄 Mise à jour

### Redéployer après modifications

Le playbook est **idempotent** - vous pouvez le relancer sans risque:

```bash
ansible-playbook -i inventory.ini playbook.yml
```

### Mise à jour manuelle du code

```bash
# Le système GitOps met à jour automatiquement tous les jours
# Pour forcer une mise à jour immédiate:
ssh ubuntu@$VPS_IP "sudo su - koprogo -c '/home/koprogo/koprogo/scripts/auto-update.sh'"
```

## 🔍 Dépannage

### Test de connexion SSH

```bash
VPS_IP=$(grep ansible_host inventory.ini | awk '{print $2}' | cut -d'=' -f2)
ssh ubuntu@$VPS_IP
```

### Ajouter l'hôte SSH (première connexion)

```bash
VPS_IP=$(grep ansible_host inventory.ini | awk '{print $2}' | cut -d'=' -f2)
ssh-keyscan -H $VPS_IP >> ~/.ssh/known_hosts
```

### L'API ne répond pas

```bash
# Vérifier les logs
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose logs backend'"

# Redémarrer les services
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose restart'"
```

### Relancer une tâche spécifique

```bash
# Si une tâche a échoué, relancer le playbook (idempotent)
ansible-playbook -i inventory.ini playbook.yml
```

## 📊 Fonctionnalités automatiques

### 1. GitOps Auto-Update

**Fichier**: `/home/koprogo/koprogo/scripts/auto-update.sh`
**Fréquence**: Quotidien à 3h du matin
**Action**: `git pull` + `docker compose up -d --build`

### 2. Backup PostgreSQL

**Fichier**: `/home/koprogo/koprogo/scripts/backup.sh`
**Fréquence**: Quotidien à 2h du matin
**Destination**: `/home/koprogo/backups/`

### 3. Health Check

**Fichier**: `/home/koprogo/koprogo/scripts/health-check.sh`
**Fréquence**: Toutes les 5 minutes
**Action**: Ping API + Redémarrage automatique si down

## 🔐 Sécurité

Le playbook configure:
- **Firewall UFW**: Bloque tout sauf SSH/HTTP/HTTPS
- **Fail2ban**: Protection brute-force SSH
- **Utilisateur non-root**: `koprogo` pour l'application
- **Fichier .env sécurisé**: Permissions 0600
- **Docker sans sudo**: Groupe `docker` pour l'utilisateur

## 🔗 Intégration Terraform + Ansible

Workflow complet:

```bash
# 1. Déployer l'infrastructure
cd infrastructure/terraform
source ./load-env.sh
terraform apply

# 2. Configurer Ansible
cd ../ansible
./setup-inventory.sh

# 3. Déployer l'application
ansible-playbook -i inventory.ini playbook.yml

# 4. Tester
./quick-test.sh
```

## 📚 Documentation

- **[TESTING.md](./TESTING.md)** - Guide complet de test (très détaillé)
- **[playbook.yml](./playbook.yml)** - Code du playbook (commenté)
- **[templates/](./templates/)** - Templates Jinja2

## 🎓 Commandes utiles

```bash
# Test de connexion
ansible koprogo -i inventory.ini -m ping

# Collecter des infos
ansible koprogo -i inventory.ini -m setup

# Exécuter une commande
ansible koprogo -i inventory.ini -m shell -a "df -h"

# Redémarrer services
ansible koprogo -i inventory.ini -m shell \
  -a "cd /home/koprogo/koprogo && docker compose restart" \
  --become --become-user=koprogo

# Voir les logs
ssh ubuntu@$VPS_IP "sudo su - koprogo -c 'cd koprogo && docker compose logs -f'"
```

## 📝 Checklist de déploiement

- [ ] VPS créé avec Terraform
- [ ] IP récupérée (`terraform output vps_ip`)
- [ ] Ansible installé
- [ ] `inventory.ini` configuré (`./setup-inventory.sh`)
- [ ] Test de connexion OK (`ansible koprogo -m ping`)
- [ ] Playbook exécuté (`ansible-playbook playbook.yml`)
- [ ] API accessible (`curl http://$VPS_IP:8080/api/v1/health`)
- [ ] Services actifs (`docker compose ps`)
- [ ] Cron jobs configurés (`crontab -l`)

**Bon déploiement! 🚀**

# KoproGo - Déploiement GitOps Automatique

Guide complet pour déployer KoproGo avec mises à jour automatiques depuis GitHub.

## Philosophie

**100% Self-Hosted + Auto-Update** : KoproGo ne propose AUCUN service cloud payant. Chaque utilisateur héberge sa propre instance (VPS, machine locale, ou même Raspberry Pi), et peut configurer les mises à jour automatiques depuis le dépôt GitHub officiel.

**Avantages** :
- ✅ **Gratuité totale** : 0€ pour le logiciel (coût VPS ~5-7€/mois si cloud)
- ✅ **Souveraineté des données** : Vos données restent sur VOTRE serveur
- ✅ **Updates automatiques** : Pull automatique des nouvelles versions
- ✅ **Sécurité** : Patches de sécurité appliqués automatiquement
- ✅ **Rollback facile** : Revenir à une version précédente en 1 commande
- ✅ **0 vendor lock-in** : Vous contrôlez 100% de l'infrastructure

---

## Prérequis

**Serveur** :
- VPS Linux (Ubuntu 22.04+ recommandé) ou machine locale
- 1 vCPU, 2GB RAM minimum (suffit pour 1,000-1,500 copropriétés)
- 20GB espace disque minimum
- Accès SSH root ou sudo

**Logiciels** :
- Docker 24+ (installation ci-dessous)
- Docker Compose v2+ (plugin Docker)
- Git 2.30+

**Domaine (optionnel)** :
- Nom de domaine pointant vers votre VPS (pour SSL/HTTPS automatique)
- Ou utiliser IP directe (HTTP seulement, OK pour usage local)

---

## Installation Rapide (1-Click)

### Option 1 : Script Auto-Install (Recommandé)

```bash
# Télécharger et exécuter le script d'installation
curl -fsSL https://raw.githubusercontent.com/gilmry/koprogo/main/scripts/install.sh | bash

# Ou si vous préférez inspecter avant :
curl -fsSL https://raw.githubusercontent.com/gilmry/koprogo/main/scripts/install.sh -o install.sh
cat install.sh  # Inspecter le script
bash install.sh
```

**Ce que fait le script** :
1. Installe Docker + Docker Compose si manquant
2. Clone le dépôt KoproGo
3. Crée fichier `.env` avec configuration par défaut
4. Configure auto-update via cron
5. Lance les services Docker
6. Affiche URL d'accès

**Durée** : 2-5 minutes

### Option 2 : Installation Manuelle (Contrôle Total)

#### Étape 1 : Installer Docker

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Ajouter utilisateur au groupe docker
sudo usermod -aG docker $USER
newgrp docker  # Activer immédiatement

# Vérifier
docker --version
docker compose version
```

#### Étape 2 : Cloner KoproGo

```bash
# Choisir répertoire installation
cd ~
mkdir -p apps && cd apps

# Cloner le dépôt
git clone https://github.com/gilmry/koprogo.git
cd koprogo

# Checkout version stable (tag)
git checkout $(git describe --tags --abbrev=0)
# Ou rester sur main pour dernière version
```

#### Étape 3 : Configuration

```bash
# Copier fichier env exemple
cp .env.example .env

# Éditer configuration
nano .env
```

**Variables importantes** :

```bash
# Base de données
DATABASE_URL=postgresql://koprogo:CHANGE_THIS_PASSWORD@postgres:5432/koprogo_db

# Serveur
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Logs
RUST_LOG=info

# Domaine (si vous avez un domaine)
DOMAIN=votre-domaine.com  # ou IP

# SSL (via Traefik)
ENABLE_SSL=true  # false si pas de domaine
ACME_EMAIL=your-email@example.com
```

**⚠️ IMPORTANT** : Changez le mot de passe PostgreSQL !

#### Étape 4 : Premier Lancement

```bash
# Lancer tous les services
docker compose up -d

# Vérifier que tout tourne
docker compose ps

# Voir les logs
docker compose logs -f

# Une fois OK, Ctrl+C pour quitter les logs
```

#### Étape 5 : Vérifier Installation

```bash
# Test santé API
curl http://localhost:8080/api/v1/health

# Devrait retourner : {"status":"healthy"}
```

**Accès Frontend** :
- Local : http://localhost:3000 (si frontend lancé)
- Ou avec domaine : https://votre-domaine.com

---

## Auto-Update GitOps

### Méthode 1 : Cron Job (Simple)

**Configuration auto-update quotidienne** :

```bash
# Créer script update
cat > ~/apps/koprogo/scripts/auto-update.sh << 'EOF'
#!/bin/bash
set -e

cd ~/apps/koprogo

# Fetch dernières versions
git fetch --tags

# Backup avant update
docker compose exec postgres pg_dump -U koprogo koprogo_db > backups/pre-update-$(date +%Y%m%d).sql

# Pull dernière version stable (tags)
LATEST_TAG=$(git describe --tags --abbrev=0)
git checkout $LATEST_TAG

# Rebuild et redémarrer
docker compose pull
docker compose up -d --build

# Vérifier santé
sleep 10
curl -f http://localhost:8080/api/v1/health || echo "Warning: Health check failed"

echo "Update completed to version $LATEST_TAG"
EOF

chmod +x ~/apps/koprogo/scripts/auto-update.sh
```

**Ajouter au cron** :

```bash
crontab -e

# Ajouter cette ligne (update tous les jours à 3h du matin)
0 3 * * * /home/$USER/apps/koprogo/scripts/auto-update.sh >> /var/log/koprogo-update.log 2>&1
```

### Méthode 2 : GitHub Actions Self-Hosted Runner (Avancé)

**Avantages** :
- Update immédiat après chaque release GitHub
- Notifications Discord/Slack si échec
- Rollback automatique si health check fail

**Setup** :

```bash
# 1. Installer GitHub Actions runner sur votre VPS
# Suivre : https://docs.github.com/en/actions/hosting-your-own-runners

# 2. Créer workflow dans .github/workflows/auto-deploy.yml
# (Fourni dans le dépôt KoproGo)

# 3. Runner écoute GitHub et déploie automatiquement
```

**Workflow exemple** (`.github/workflows/self-deploy.yml`) :

```yaml
name: Self-Deploy

on:
  release:
    types: [published]

jobs:
  deploy:
    runs-on: self-hosted
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          ref: ${{ github.event.release.tag_name }}

      - name: Backup Database
        run: |
          docker compose exec -T postgres pg_dump -U koprogo koprogo_db > backups/pre-${{ github.event.release.tag_name }}.sql

      - name: Pull & Rebuild
        run: |
          docker compose pull
          docker compose up -d --build

      - name: Health Check
        run: |
          sleep 10
          curl -f http://localhost:8080/api/v1/health

      - name: Notify Discord
        if: always()
        run: |
          # Webhook Discord (optionnel)
          curl -X POST ${{ secrets.DISCORD_WEBHOOK }} \
            -H 'Content-Type: application/json' \
            -d '{"content":"KoproGo updated to ${{ github.event.release.tag_name }}: ${{ job.status }}"}'
```

### Méthode 3 : Watchtower (Docker Auto-Update)

**Plus simple** : Watchtower surveille images Docker et update automatiquement.

```bash
# Ajouter Watchtower au docker-compose.yml
services:
  watchtower:
    image: containrrr/watchtower
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - WATCHTOWER_CLEANUP=true
      - WATCHTOWER_POLL_INTERVAL=86400  # Check daily
      - WATCHTOWER_INCLUDE_STOPPED=true
    restart: unless-stopped
```

**Relancer** :

```bash
docker compose up -d
```

Watchtower va maintenant vérifier quotidiennement si de nouvelles images sont disponibles sur Docker Hub et mettre à jour automatiquement.

---

## Versioning & Rollback

### Versions Stables (Tags Git)

KoproGo utilise semantic versioning : `vMAJOR.MINOR.PATCH`

```bash
# Lister versions disponibles
git tag

# Exemple output :
# v0.1.0
# v0.2.0
# v0.2.1
# v1.0.0

# Checkout version spécifique
git checkout v1.0.0
docker compose up -d --build
```

### Rollback si Problème

```bash
# Revenir à version précédente
git checkout v0.2.1  # Version qui marchait
docker compose up -d --build

# Restaurer backup DB si nécessaire
docker compose exec -T postgres psql -U koprogo -d koprogo_db < backups/pre-v1.0.0.sql
```

---

## Monitoring & Maintenance

### Vérifier Status

```bash
# Services actifs
docker compose ps

# Logs en direct
docker compose logs -f

# Logs service spécifique
docker compose logs -f backend

# Disk usage
docker system df
```

### Backups

**Backup automatique quotidien** :

```bash
# Script backup
cat > ~/apps/koprogo/scripts/backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=~/apps/koprogo/backups
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup PostgreSQL
docker compose exec -T postgres pg_dump -U koprogo koprogo_db | gzip > $BACKUP_DIR/koprogo_$DATE.sql.gz

# Garder 7 derniers jours
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete

echo "Backup created: $BACKUP_DIR/koprogo_$DATE.sql.gz"
EOF

chmod +x ~/apps/koprogo/scripts/backup.sh

# Ajouter au cron (tous les jours à 2h)
crontab -e
# Ajouter : 0 2 * * * ~/apps/koprogo/scripts/backup.sh >> /var/log/koprogo-backup.log 2>&1
```

### Monitoring Uptime

**Option 1** : UptimeRobot (gratuit, externe)
- https://uptimerobot.com
- Surveille votre instance toutes les 5 minutes
- Alerte email/SMS si down

**Option 2** : Script local

```bash
# Health check cron
crontab -e
# Ajouter : */5 * * * * curl -f http://localhost:8080/api/v1/health || echo "KoproGo DOWN" | mail -s "Alert" your-email@example.com
```

---

## Sécurité

### SSL/HTTPS (via Traefik)

KoproGo utilise Traefik pour SSL automatique avec Let's Encrypt.

**Configuration** (dans `.env`) :

```bash
DOMAIN=votre-domaine.com
ENABLE_SSL=true
ACME_EMAIL=your-email@example.com
```

**Traefik génère automatiquement** :
- Certificat SSL Let's Encrypt
- Renouvellement auto (avant expiration)
- Redirect HTTP → HTTPS

### Firewall

```bash
# Installer UFW (Ubuntu)
sudo apt install ufw

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable
sudo ufw enable

# Status
sudo ufw status
```

### Secrets

**Ne JAMAIS committer** `.env` dans Git !

```bash
# .gitignore inclut déjà .env

# Vérifier
git status  # .env ne doit PAS apparaître
```

**Changer mot de passe DB** :

```bash
# Générer password sécurisé
openssl rand -base64 32

# Éditer .env
nano .env
# DATABASE_URL=postgresql://koprogo:NEW_SECURE_PASSWORD@postgres:5432/koprogo_db

# Recréer containers
docker compose down
docker compose up -d
```

---

## Troubleshooting

### Service ne démarre pas

```bash
# Voir logs erreur
docker compose logs backend

# Vérifier .env
cat .env | grep DATABASE_URL

# Recréer containers
docker compose down
docker compose up -d --build
```

### Base de données corrompue

```bash
# Restaurer backup
docker compose exec -T postgres psql -U koprogo -d koprogo_db < backups/latest.sql

# Ou recréer DB
docker compose down -v  # ⚠️ Supprime toutes les données
docker compose up -d
```

### Out of Disk Space

```bash
# Nettoyer Docker
docker system prune -a
docker volume prune

# Nettoyer logs
sudo journalctl --vacuum-time=7d

# Nettoyer vieux backups
find ~/apps/koprogo/backups -name "*.sql.gz" -mtime +30 -delete
```

### Update échoue

```bash
# Revenir version précédente
git checkout v0.2.1  # Version stable
docker compose up -d --build

# Restaurer DB
docker compose exec -T postgres psql -U koprogo -d koprogo_db < backups/pre-update.sql

# Reporter issue GitHub
# https://github.com/gilmry/koprogo/issues
```

---

## Contribuer

KoproGo est 100% open-source (MIT). Si vous améliorez le déploiement GitOps :

1. Fork le dépôt
2. Créer branche : `git checkout -b feature/improve-deploy`
3. Commit : `git commit -m "feat: improve auto-update script"`
4. Push : `git push origin feature/improve-deploy`
5. Ouvrir Pull Request sur GitHub

**Guidelines** :
- Tester sur Ubuntu 22.04 minimum
- Documenter changements dans ce fichier
- Ajouter tests si applicable

---

## Support

**Problème d'installation ?**

1. **Discord communauté** : [à créer]
2. **GitHub Issues** : https://github.com/gilmry/koprogo/issues
3. **Documentation** : https://docs.koprogo.com (à venir)

**Auto-hébergement ≠ Support garanti** : KoproGo est gratuit et bénévole. Le support communautaire est basé sur l'entraide. Soyez patient et contribuez si vous pouvez !

---

**KoproGo ASBL** - Un bien commun numérique 🏛️🔓

*Auto-hébergement + GitOps = Souveraineté totale*

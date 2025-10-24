# Déploiement VPS KoproGo (1 vCPU / 2GB RAM)

Guide de déploiement pour un VPS à ressources limitées (1 vCPU, 2GB RAM).

## Table des matières

- [Prérequis](#prérequis)
- [Configuration du VPS](#configuration-du-vps)
- [Configuration de l'application](#configuration-de-lapplication)
- [Déploiement](#déploiement)
- [Monitoring](#monitoring)
- [Tests de charge](#tests-de-charge)
- [Optimisations appliquées](#optimisations-appliquées)
- [Maintenance](#maintenance)

## Prérequis

- VPS avec 1 vCPU et 2GB RAM minimum
- Ubuntu 22.04 LTS ou Debian 12
- Nom de domaine configuré avec DNS pointant vers le VPS
- Accès SSH root ou sudo

## Configuration du VPS

### 1. Installation des dépendances

```bash
# Mise à jour du système
apt update && apt upgrade -y

# Installation de Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Installation de Docker Compose
apt install docker-compose-plugin -y

# Installation des outils de monitoring
apt install htop iotop ncdu -y
```

### 2. Configuration du swap (recommandé pour 2GB RAM)

```bash
# Créer un fichier swap de 2GB
fallocate -l 2G /swapfile
chmod 600 /swapfile
mkswap /swapfile
swapon /swapfile

# Rendre le swap permanent
echo '/swapfile none swap sw 0 0' >> /etc/fstab

# Optimiser l'usage du swap (swappiness=10 pour limiter l'usage)
echo 'vm.swappiness=10' >> /etc/sysctl.conf
sysctl -p
```

### 3. Sécurisation SSH

```bash
# Désactiver l'accès root (après avoir créé un utilisateur sudo)
sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config

# Désactiver l'authentification par mot de passe (utiliser des clés SSH)
sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config

systemctl restart sshd
```

### 4. Configuration du pare-feu

```bash
# Installation d'ufw
apt install ufw -y

# Configuration des règles
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
```

## Configuration de l'application

### 1. Cloner le dépôt

```bash
cd /opt
git clone https://github.com/votre-org/koprogo.git
cd koprogo
```

### 2. Créer le fichier d'environnement

```bash
cp .env.example .env.vps
```

### 3. Éditer `.env.vps` avec vos valeurs

```bash
# Variables critiques à modifier
POSTGRES_PASSWORD=VOTRE_MOT_DE_PASSE_SECURISE
JWT_SECRET=VOTRE_CLE_SECRETE_LONGUE_ET_ALEATOIRE_MIN_32_CHARS

# Domaines
FRONTEND_DOMAIN=app.votredomaine.com
API_DOMAIN=api.votredomaine.com
TRAEFIK_DOMAIN=traefik.votredomaine.com

# Email pour Let's Encrypt
ACME_EMAIL=admin@votredomaine.com

# CORS
CORS_ORIGIN=https://app.votredomaine.com

# Auth Traefik Dashboard (générer avec: htpasswd -nb admin votre_mot_de_passe)
TRAEFIK_DASHBOARD_AUTH=admin:$apr1$...

# Logging (warn pour économiser les ressources)
RUST_LOG=warn
TRAEFIK_LOG_LEVEL=WARN

# Ressources (déjà optimisées dans docker-compose.vps.yml)
ACTIX_WORKERS=1
DB_POOL_MAX_CONNECTIONS=8
DB_POOL_MIN_CONNECTIONS=2
```

### 4. Générer un mot de passe sécurisé pour Traefik

```bash
# Installer apache2-utils pour htpasswd
apt install apache2-utils -y

# Générer le hash (remplacer 'votre_mot_de_passe')
htpasswd -nb admin votre_mot_de_passe
```

## Déploiement

### 1. Build des images

```bash
docker compose -f docker-compose.vps.yml --env-file .env.vps build
```

### 2. Démarrage des services

```bash
docker compose -f docker-compose.vps.yml --env-file .env.vps up -d
```

### 3. Vérifier les logs

```bash
# Tous les services
docker compose -f docker-compose.vps.yml logs -f

# Service spécifique
docker compose -f docker-compose.vps.yml logs -f backend
docker compose -f docker-compose.vps.yml logs -f postgres
docker compose -f docker-compose.vps.yml logs -f traefik
```

### 4. Vérifier l'état des services

```bash
docker compose -f docker-compose.vps.yml ps
```

Tous les services doivent être `healthy` :

```
NAME                 STATUS          PORTS
koprogo-traefik      Up 2 minutes    0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp
koprogo-postgres     Up 2 minutes    (healthy)
koprogo-backend      Up 2 minutes    (healthy)
koprogo-frontend     Up 2 minutes    (healthy)
```

## Monitoring

### 1. Vérifier l'usage mémoire

```bash
# Vue globale
free -h

# Par container
docker stats
```

### 2. Vérifier l'usage CPU

```bash
htop
```

### 3. Dashboard Traefik

Accéder à `https://traefik.votredomaine.com/dashboard/`

- Username: `admin`
- Password: celui que vous avez configuré

### 4. Logs applicatifs

```bash
# Backend API
docker compose -f docker-compose.vps.yml logs backend --tail=100 -f

# PostgreSQL
docker compose -f docker-compose.vps.yml logs postgres --tail=100 -f
```

### 5. Healthchecks

```bash
# API health
curl https://api.votredomaine.com/api/v1/health

# Frontend health
curl https://app.votredomaine.com/health
```

## Tests de charge

Après le déploiement, il est recommandé de valider les performances du système avec des tests de charge.

### ⚠️ Important : Tests à distance

**Les tests de charge doivent être lancés depuis une machine externe** (votre ordinateur ou un autre VPS), **PAS depuis le VPS serveur lui-même**.

Raisons :
- Les tests consomment CPU/RAM et faussent les résultats
- Pas de latence réseau en localhost
- Conditions irréalistes

**→ Voir la documentation complète : [load-tests/REMOTE_TESTING.md](load-tests/REMOTE_TESTING.md)**

### Setup rapide (machine cliente)

Sur votre **machine locale** ou un **VPS client** :

```bash
# Cloner le repo
git clone https://github.com/votre-org/koprogo.git
cd koprogo/load-tests

# Installer wrk
# Ubuntu/Debian:
sudo apt-get install wrk

# macOS:
brew install wrk
```

### Tests disponibles

Le projet inclut une suite complète de tests de charge dans `load-tests/` :

| Test | Durée | Charge | Objectif |
|------|-------|--------|----------|
| **warmup** | 30s | Très légère | Préparer le système (toujours lancer en premier) |
| **light-load** | 2 min | 100 req/s | Valider le fonctionnement normal |
| **medium-load** | 5 min | 500 req/s | Tester les pics de charge |
| **heavy-load** | 3 min | 1000 req/s | Trouver le point de rupture |
| **spike-test** | 5 min | Variable | Tester la résilience aux pics soudains |
| **soak-test** | 30 min | Soutenue | Détecter les fuites mémoire |

### Lancement rapide

**Sur la machine cliente** (pas sur le VPS !) :

```bash
cd koprogo/load-tests

# Par défaut, les scripts ciblent https://api.koprogo.com
# Aucune configuration nécessaire !
./scripts/remote-light-load.sh
./scripts/remote-medium-load.sh

# Pour tester un autre domaine :
export BASE_URL=https://api.votredomaine.com
./scripts/remote-light-load.sh
```

**En parallèle, sur le VPS** (monitoring) :

```bash
# SSH vers le VPS
ssh user@vps-ip
cd /opt/koprogo/load-tests

# Monitorer pendant 5 minutes
./monitor-server.sh 300
```

### Résultats attendus (1 vCPU / 2GB RAM)

**Light Load** ✅
- P99 latency: < 50ms
- Throughput: > 100 req/s
- Error rate: < 0.1%
- CPU usage: ~30-40%

**Medium Load** ✅
- P99 latency: < 100ms
- Throughput: > 500 req/s
- Error rate: < 0.5%
- CPU usage: ~70-80%

**Heavy Load** ⚠️
- P99 latency: < 200ms (acceptable)
- Throughput: Plateau possible
- Error rate: < 5% (acceptable)
- CPU usage: 95-100%

### Monitoring pendant les tests

**Terminal 1 : Machine cliente (lance les tests)**

```bash
cd koprogo/load-tests
export BASE_URL=https://api.votredomaine.com
./scripts/remote-medium-load.sh
```

**Terminal 2 : VPS serveur (monitoring automatique)**

```bash
ssh user@vps-ip
cd /opt/koprogo/load-tests
./monitor-server.sh 300  # 5 minutes
```

**Terminal 3 : VPS serveur (logs en temps réel)**

```bash
ssh user@vps-ip
docker compose -f docker-compose.vps.yml logs -f backend | grep -E "(ERROR|WARN)"
```

### Suite complète de tests

**Machine cliente :**

```bash
cd koprogo/load-tests
export BASE_URL=https://api.votredomaine.com

# Tests individuels
./scripts/remote-light-load.sh
./scripts/remote-medium-load.sh

# Résultats locaux
ls -lh results/
```

**VPS serveur (récupérer les logs de monitoring) :**

```bash
ssh user@vps-ip
cd /opt/koprogo/load-tests
ls -lh monitoring-results/
```

### Documentation complète

- **Guide rapide** : `load-tests/QUICKSTART.md`
- **Documentation détaillée** : `load-tests/README.md`
- **Scripts de test** : `load-tests/scripts/`

### Troubleshooting des tests

**Trop d'erreurs (> 5%)**
- Réduire le nombre de connexions dans les scripts
- Vérifier les logs pour identifier les goulots d'étranglement
- Augmenter `DB_POOL_MAX_CONNECTIONS` si nécessaire

**Latence élevée (> 200ms)**
- Vérifier l'usage CPU (doit être < 95%)
- Vérifier le swap (devrait être peu utilisé)
- Analyser les requêtes lentes dans PostgreSQL

**Out of Memory**
- Réduire `shared_buffers` PostgreSQL
- Réduire `DB_POOL_MAX_CONNECTIONS`
- Vérifier les fuites mémoire avec `soak-test.sh`

## Optimisations appliquées

### Allocation mémoire (Total: ~1.6GB + système)

| Service    | Limite RAM | Réservation | CPU Limit |
|------------|------------|-------------|-----------|
| Traefik    | 50MB       | -           | 0.25      |
| PostgreSQL | ~256MB     | -           | -         |
| Backend    | 384MB      | 256MB       | 0.75      |
| Frontend   | 128MB      | 96MB        | 0.5       |

### PostgreSQL (256MB alloués)

- `shared_buffers`: 256MB (mémoire partagée)
- `effective_cache_size`: 768MB (estimation du cache OS)
- `work_mem`: 5MB (par opération de tri/hash)
- `maintenance_work_mem`: 64MB (VACUUM, CREATE INDEX)
- `max_connections`: 15 (limité pour économiser la RAM)
- WAL optimisé pour SSD avec checksums

### Backend Rust

- **1 worker Actix** au lieu de 2
- **Pool de connexions DB: 8 max** au lieu de 10
  - `min_connections`: 2
  - `acquire_timeout`: 30s
  - `idle_timeout`: 10 minutes
  - `max_lifetime`: 30 minutes
- **RUST_LOG=warn** pour limiter les I/O de logs
- Rate limiting: 100 req/s moyen, burst 200

### Frontend

- Nginx Alpine (léger)
- Gzip compression activée
- Cache des assets statiques (1 an)
- 128MB RAM limite

### Logs

Rotation automatique configurée :
- Taille max par fichier: 10MB
- Nombre de fichiers: 3
- Total max par service: ~30MB

## Maintenance

### Mise à jour de l'application

```bash
cd /opt/koprogo
git pull origin main

# Rebuild et redémarrage
docker compose -f docker-compose.vps.yml --env-file .env.vps build
docker compose -f docker-compose.vps.yml --env-file .env.vps up -d
```

### Backup de la base de données

```bash
# Créer un backup
docker compose -f docker-compose.vps.yml exec postgres pg_dump -U koprogo koprogo_db > backup_$(date +%Y%m%d).sql

# Restaurer un backup
docker compose -f docker-compose.vps.yml exec -T postgres psql -U koprogo koprogo_db < backup_20250124.sql
```

### Nettoyage Docker

```bash
# Supprimer les images inutilisées
docker image prune -a -f

# Supprimer les volumes inutilisés
docker volume prune -f

# Supprimer tout ce qui n'est pas utilisé
docker system prune -a --volumes -f
```

### Vérifier l'espace disque

```bash
# Vue globale
df -h

# Par répertoire (interactif)
ncdu /

# Espace utilisé par Docker
docker system df
```

### Renouvellement SSL (Let's Encrypt)

Le renouvellement est automatique via Traefik. Les certificats sont stockés dans le volume `traefik_letsencrypt`.

Pour forcer un renouvellement :

```bash
docker compose -f docker-compose.vps.yml restart traefik
```

### Surveillance des performances

```bash
# Statistiques en temps réel
docker stats

# Top des processus
htop

# I/O disque
iotop -o
```

## Troubleshooting

### Le backend ne démarre pas

1. Vérifier les logs :
   ```bash
   docker compose -f docker-compose.vps.yml logs backend
   ```

2. Vérifier que PostgreSQL est healthy :
   ```bash
   docker compose -f docker-compose.vps.yml ps postgres
   ```

3. Vérifier les migrations :
   ```bash
   docker compose -f docker-compose.vps.yml exec backend ls -la /app/migrations
   ```

### PostgreSQL est lent

1. Vérifier les connexions actives :
   ```bash
   docker compose -f docker-compose.vps.yml exec postgres \
     psql -U koprogo -d koprogo_db -c "SELECT count(*) FROM pg_stat_activity;"
   ```

2. Vérifier la configuration :
   ```bash
   docker compose -f docker-compose.vps.yml exec postgres \
     psql -U koprogo -d koprogo_db -c "SHOW shared_buffers; SHOW work_mem;"
   ```

3. Analyser les requêtes lentes :
   ```bash
   docker compose -f docker-compose.vps.yml logs postgres | grep "duration"
   ```

### Mémoire saturée

1. Vérifier l'usage :
   ```bash
   free -h
   docker stats --no-stream
   ```

2. Identifier le service problématique et ajuster les limites dans `docker-compose.vps.yml`

3. Vérifier le swap :
   ```bash
   swapon --show
   ```

### SSL ne fonctionne pas

1. Vérifier que les DNS pointent bien vers le VPS :
   ```bash
   dig app.votredomaine.com
   dig api.votredomaine.com
   ```

2. Vérifier les logs Traefik :
   ```bash
   docker compose -f docker-compose.vps.yml logs traefik
   ```

3. Vérifier les ports 80 et 443 :
   ```bash
   ufw status
   netstat -tulpn | grep -E ':(80|443)'
   ```

## Support

Pour toute question ou problème :

- GitHub Issues : https://github.com/votre-org/koprogo/issues
- Documentation : https://docs.koprogo.com

## Références

- [Docker Compose documentation](https://docs.docker.com/compose/)
- [Traefik documentation](https://doc.traefik.io/traefik/)
- [PostgreSQL tuning](https://wiki.postgresql.org/wiki/Tuning_Your_PostgreSQL_Server)
- [Actix-web performance](https://actix.rs/docs/server/)

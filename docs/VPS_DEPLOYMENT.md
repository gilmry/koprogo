# KoproGo VPS Deployment Guide

Complete guide to deploying KoproGo on OVH Cloud France (low-cost, GDPR-native, ultra-low carbon footprint).

## 🎯 Qui utilise ce guide ?

Ce guide est pour **deux cas d'usage** :

### 1. **Cloud ASBL (Hébergement Géré)** ☁️
L'ASBL KoproGo utilise ce guide pour maintenir son infrastructure cloud multi-tenant (1€/copro/mois).

### 2. **Self-Hosting (Gratuit)** 🔓
Copropriétés ou syndics qui veulent héberger leur propre instance KoproGo.

**💡 Vous préférez l'auto-update automatique ?** Consultez plutôt **[DEPLOY_GITOPS.md](DEPLOY_GITOPS.md)** pour installation 1-click avec mises à jour GitHub automatiques.

---

## Architecture Overview

```
┌──────────────┐
│ Users        │
└──────┬───────┘
       │
   ┌───┴────┐
   │        │
   ▼        ▼
┌─────────┐  ┌────────────────────────┐
│ Vercel  │  │  VPS OVH Cloud France  │
│ (Astro) │  │  1 vCPU / 2GB RAM     │
│ Frontend│  │  ┌──────────────────┐  │
│ Static  │──▶  │ Traefik (reverse │  │
│ CDN     │     │ proxy + SSL)     │  │
└─────────┘     └─────────┬────────┘  │
 Gratuit                  │           │
                          ▼           │
                  ┌──────────────┐    │
                  │ Backend      │    │
                  │ (Rust/Actix) │    │
                  │ Docker       │    │
                  └───────┬──────┘    │
                          │           │
                          ▼           │
                  ┌──────────────┐    │
                  │ PostgreSQL   │    │
                  │ 16-alpine    │    │
                  │ Docker       │    │
                  └──────────────┘    │
                                      │
   Datacenter France                 │
   60g CO₂/kWh (nucléaire)           │
   0.12g CO₂/requête                 │
└────────────────────────────────────┘
   5€/mois
```

## Cost Breakdown

### Recommended Setup (Production-Validated 2025)

| Component | Provider | Cost |
|-----------|----------|------|
| Backend VPS | OVH Cloud France VPS Value | **7€/mois TTC** (5.80€ HT) |
| Frontend | Vercel (Free tier) | 0€ |
| Database | Same VPS | 0€ |
| Domain | koprogo.com | ~12€/an (~1€/mois) |
| SSL Certificate | Let's Encrypt (via Traefik) | 0€ |
| **TOTAL** | | **~8€/mois** (96€/an) |

**Note prix 2025** : OVH a ajusté ses tarifs. Le VPS Value (1 vCore, 2GB RAM, 40GB NVMe) est maintenant à **5.80€ HT/mois = 7.02€ TTC** avec TVA belge 21%.

### Why OVH Cloud France?

**✅ Performance Validée (Tests Réels)** :
- 1 vCPU / 2GB RAM
- **1,000-1,500 copropriétés** supportées
- **287 req/s** soutenus, 99.74% uptime
- P50: 69ms, P90: 130ms, P99: 752ms
- 40GB SSD NVMe

**✅ Écologie Exceptionnelle** :
- Datacenter France (mix énergétique 60g CO₂/kWh)
- **0.12g CO₂/requête** (7-25x mieux que AWS/Azure)
- Nucléaire (70%) + Renouvelables (25%)

**✅ Souveraineté & GDPR** :
- Données hébergées en France
- GDPR-native, conformité totale
- Pas de CLOUD Act américain
- Support en français

**✅ Infrastructure** :
- 40GB SSD NVMe
- 1 Gbps network
- Anti-DDoS inclus
- Cost: **5€/mois**

### Alternative Providers (Not Recommended)

Nous recommandons **exclusivement OVH France** pour les avantages écologiques, GDPR et souveraineté. Alternatives si nécessaire :

1. **Hetzner Germany** (alternative acceptable)
   - Mix énergétique allemand : 350g CO₂/kWh (**5.8x plus** que France)
   - GDPR OK mais datacenter DE
   - Cost: 4,15€/mois

2. **DigitalOcean** (Non recommandé)
   - Datacenters USA = CLOUD Act
   - Mix énergétique 400g+ CO₂/kWh
   - Cost: $6/mois

## Step-by-Step Deployment

### 1. Provision VPS

#### Hetzner Example

1. Create account at https://www.hetzner.com
2. Choose: Cloud → Create Server
3. Select:
   - **Location**: Nuremberg (Germany) or Falkenstein
   - **Image**: Ubuntu 22.04 LTS
   - **Type**: CPX11 (Shared vCPU, 2GB RAM)
   - **Networking**: IPv4 + IPv6
   - **SSH Key**: Upload your public key
4. Create server

**Initial login:**
```bash
ssh root@<vps-ip-address>
```

### 2. Initial Server Setup

```bash
# Update system
apt update && apt upgrade -y

# Set timezone
timedatectl set-timezone Europe/Brussels

# Set hostname
hostnamectl set-hostname koprogo-vps

# Create non-root user
adduser koprogo
usermod -aG sudo koprogo
usermod -aG docker koprogo  # (after Docker install)

# Copy SSH keys to new user
rsync --archive --chown=koprogo:koprogo ~/.ssh /home/koprogo
```

### 3. Install Dependencies

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
apt install docker-compose-plugin -y

# Install utilities
apt install -y git curl wget htop ufw postgresql-client

# Verify installations
docker --version
docker compose version
```

### 4. Configure Firewall

```bash
# Enable UFW
ufw default deny incoming
ufw default allow outgoing

# Allow SSH
ufw allow 22/tcp

# Allow HTTP/HTTPS
ufw allow 80/tcp
ufw allow 443/tcp

# Enable firewall
ufw enable

# Check status
ufw status
```

### 5. Clone and Configure KoproGo

```bash
# Switch to koprogo user
su - koprogo

# Clone repository
cd ~
git clone https://github.com/your-org/koprogo.git
cd koprogo

# Create .env file
cat > backend/.env << EOF
DATABASE_URL=postgresql://koprogo:CHANGE_THIS_PASSWORD@postgres:5432/koprogo_db
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info
ACTIX_WORKERS=2
EOF

# Set secure PostgreSQL password
nano backend/.env  # Change CHANGE_THIS_PASSWORD

# Create nginx directories
mkdir -p nginx/ssl nginx/logs
```

### 6. Deploy with Docker Compose

```bash
# Build and start services
docker compose -f docker-compose.vps.yml up -d --build

# Check status
docker compose -f docker-compose.vps.yml ps

# View logs
docker compose -f docker-compose.vps.yml logs -f
```

### 7. Run Database Migrations

```bash
# Execute migrations
docker exec -it koprogo-backend /app/koprogo-backend migrate

# Or manually with psql
docker exec -it koprogo-postgres psql -U koprogo -d koprogo_db -f /docker-entrypoint-initdb.d/migrations/001_initial_schema.sql
```

### 8. Verify Deployment

```bash
# Check health endpoint
curl http://localhost:8080/api/v1/health

# Expected output: {"status":"healthy"}

# Test from outside
curl http://<vps-ip-address>/api/v1/health
```

### 9. Setup SSL with Let's Encrypt

```bash
# Install Certbot
apt install certbot python3-certbot-nginx -y

# Stop nginx temporarily
docker compose -f docker-compose.vps.yml stop nginx

# Obtain certificate (replace your-domain.com)
certbot certonly --standalone -d api.koprogo.be

# Certificates stored in: /etc/letsencrypt/live/api.koprogo.be/

# Copy to nginx directory
cp -r /etc/letsencrypt/live /home/koprogo/koprogo/nginx/ssl/
cp -r /etc/letsencrypt/archive /home/koprogo/koprogo/nginx/ssl/

# Update nginx/conf.d/koprogo.conf (uncomment HTTPS section)
nano nginx/conf.d/koprogo.conf

# Restart nginx
docker compose -f docker-compose.vps.yml up -d nginx

# Auto-renewal (cron)
crontab -e
# Add: 0 0 * * * certbot renew --quiet && docker compose -f /home/koprogo/koprogo/docker-compose.vps.yml restart nginx
```

### 10. Setup Monitoring

```bash
# Make scripts executable
chmod +x monitoring/scripts/*.sh

# Test monitoring
./monitoring/scripts/vps_metrics.sh
./monitoring/scripts/postgres_metrics.sh
./monitoring/scripts/capacity_calculator.sh

# Setup cron jobs
crontab -e
```

Add these lines:
```cron
# System metrics every 5 minutes
*/5 * * * * /home/koprogo/koprogo/monitoring/scripts/vps_metrics.sh >> /var/log/koprogo/vps.log 2>&1

# PostgreSQL metrics hourly
0 * * * * /home/koprogo/koprogo/monitoring/scripts/postgres_metrics.sh >> /var/log/koprogo/postgres.log 2>&1

# Daily capacity report at 9am
0 9 * * * /home/koprogo/koprogo/monitoring/scripts/capacity_calculator.sh >> /var/log/koprogo/capacity.log 2>&1

# Weekly cleanup (logs older than 7 days)
0 2 * * 0 find /home/koprogo/koprogo/monitoring/logs -name "*.json" -mtime +7 -delete
```

## Frontend Deployment (Vercel)

### 1. Prepare Frontend

```bash
# In frontend directory
cd frontend

# Update .env for production
cat > .env.production << EOF
PUBLIC_API_URL=https://api.koprogo.be/api/v1
EOF
```

### 2. Deploy to Vercel

```bash
# Install Vercel CLI
npm install -g vercel

# Login
vercel login

# Deploy
cd frontend
vercel --prod

# Follow prompts:
# - Link to existing project: No
# - Project name: koprogo-frontend
# - Framework: Astro
# - Build command: npm run build
# - Output directory: dist
```

### 3. Configure Domain (Optional)

In Vercel dashboard:
1. Go to Project → Settings → Domains
2. Add custom domain: `app.koprogo.be`
3. Configure DNS (A/CNAME records)

## DNS Configuration

### Example DNS Records

```
# At your domain registrar (Namecheap, Gandi, etc.)

# Backend API
api.koprogo.be.    A       <vps-ip-address>

# Frontend (if using custom domain with Vercel)
app.koprogo.be.    CNAME   cname.vercel-dns.com.

# Root domain (optional redirect to app)
koprogo.be.        A       <vps-ip-address>
```

## Database Backups

### Automated Backups

```bash
# Create backup script
cat > ~/backup-db.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/home/koprogo/backups"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

docker exec koprogo-postgres pg_dump -U koprogo koprogo_db | gzip > $BACKUP_DIR/koprogo_db_$DATE.sql.gz

# Keep only last 7 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/koprogo_db_$DATE.sql.gz"
EOF

chmod +x ~/backup-db.sh

# Schedule daily backups at 2am
crontab -e
# Add: 0 2 * * * /home/koprogo/backup-db.sh >> /var/log/koprogo/backup.log 2>&1
```

### Restore from Backup

```bash
# Stop backend
docker compose -f docker-compose.vps.yml stop backend

# Restore
gunzip -c /home/koprogo/backups/koprogo_db_YYYYMMDD_HHMMSS.sql.gz | \
  docker exec -i koprogo-postgres psql -U koprogo -d koprogo_db

# Restart backend
docker compose -f docker-compose.vps.yml start backend
```

## Maintenance

### View Logs

```bash
# All services
docker compose -f docker-compose.vps.yml logs -f

# Specific service
docker compose -f docker-compose.vps.yml logs -f backend

# Last 100 lines
docker compose -f docker-compose.vps.yml logs --tail=100 backend
```

### Restart Services

```bash
# All services
docker compose -f docker-compose.vps.yml restart

# Specific service
docker compose -f docker-compose.vps.yml restart backend
```

### Update Application

```bash
cd ~/koprogo

# Pull latest code
git pull origin main

# Rebuild and restart
docker compose -f docker-compose.vps.yml up -d --build

# Run migrations if needed
docker exec -it koprogo-backend /app/koprogo-backend migrate
```

### Clean Docker

```bash
# Remove unused images/containers
docker system prune -a

# View disk usage
docker system df
```

## Scaling Up

### When to Upgrade

Use `monitoring/scripts/capacity_calculator.sh` to determine when to upgrade.

**Signals:**
- RAM usage > 85% consistently
- CPU load > 2.0 on 2-core system
- Disk usage > 80%
- Query latency P99 > 50ms

### Upgrade Path

**100-500 copropriétés → Hetzner CPX21**
```bash
# In Hetzner Cloud Console
# 1. Create snapshot of current server
# 2. Resize to CPX21 (4GB RAM, 80GB SSD)
# 3. Or create new server from snapshot

# Update Docker resource limits in docker-compose.vps.yml
# backend: memory: 500M
# postgres: shared_buffers=1GB, effective_cache_size=3GB
```

**500-2000 copropriétés → Separate Database**
```bash
# 1. Provision dedicated PostgreSQL server (Hetzner CPX21)
# 2. Migrate database
# 3. Update DATABASE_URL in backend .env
# 4. Use backend VPS only for application
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
docker compose -f docker-compose.vps.yml logs backend

# Common issues:
# - DATABASE_URL incorrect
# - Port 8080 already in use
# - Insufficient disk space
```

### Out of Memory

```bash
# Check memory usage
free -h
docker stats

# Quick fix: restart services
docker compose -f docker-compose.vps.yml restart

# Long-term: upgrade VPS or optimize
```

### Database Connection Errors

```bash
# Check PostgreSQL is running
docker compose -f docker-compose.vps.yml ps postgres

# Test connection
docker exec -it koprogo-postgres psql -U koprogo -d koprogo_db -c "SELECT 1;"

# Check DATABASE_URL
cat backend/.env | grep DATABASE_URL
```

### SSL Certificate Issues

```bash
# Renew manually
certbot renew --force-renewal

# Check expiry
certbot certificates

# Test auto-renewal
certbot renew --dry-run
```

## Security Checklist

- [ ] Changed default PostgreSQL password
- [ ] UFW firewall enabled
- [ ] SSH key authentication only (disable password auth)
- [ ] Regular backups configured
- [ ] SSL/TLS enabled (HTTPS)
- [ ] Docker containers run as non-root users
- [ ] Rate limiting enabled in Nginx
- [ ] Monitoring and alerting setup
- [ ] System updates scheduled

```bash
# Disable SSH password authentication
sudo nano /etc/ssh/sshd_config
# Set: PasswordAuthentication no
sudo systemctl restart sshd

# Auto-updates
sudo apt install unattended-upgrades
sudo dpkg-reconfigure --priority=low unattended-upgrades
```

## Cost Optimization

### Current costs (~5€/mois)
- VPS: 4,15€
- Domain: 0,83€ (~10€/year)
- Frontend: 0€ (Vercel free)
- SSL: 0€ (Let's Encrypt)

### Revenue projections
With pricing model:
- Starter (5 copropriétés): 15€/mois
- Pro (20 copropriétés): 49€/mois

**Break-even: 1 paying customer**
**10 customers: ~300€/mois revenue, 5€ costs = 98% margin**

## Support

- Monitoring: `monitoring/README.md`
- Architecture: `CLAUDE.md`
- Issues: GitHub Issues

---

Last updated: 2024-10-23

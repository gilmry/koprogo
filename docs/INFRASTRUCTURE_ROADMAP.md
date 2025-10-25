# KoproGo Infrastructure Roadmap

Guide complet de l'évolution de l'infrastructure selon la croissance.

## Vue d'ensemble

```
Phase 1: MVP          Phase 2: Growth       Phase 3: Scale       Phase 4: Enterprise
(0-1,500 copros)     (1,500-5,000 copros)  (5,000-15,000 copros) (15,000+ copros)

VPS 1vCPU            VPS 2vCPU             K3s Dev              K3s Production HA
5€/mois              15€/mois              30€/mois             270€/mois
OVH France           OVH France            OVH France           OVH France

Docker Compose       Docker Compose        Kubernetes           Kubernetes HA
+ Traefik           + Traefik             Single Node          Multi-Node
99.74% uptime        99.9% target          99.95% target        99.99% target
```

## Phase 1: MVP - VPS Simple (0-1,500 copropriétés)

### Infrastructure
- **Hébergement** : OVH Cloud France
- **Coût** : ~5€/mois
- **Specs** : 1 vCPU, 2GB RAM, 40GB SSD
- **Datacenter** : France (mix énergétique 60g CO₂/kWh)
- **Stack** :
  - Docker Compose
  - Traefik (reverse proxy + SSL auto)
  - PostgreSQL 15
  - Backend Rust
  - Frontend sur Vercel (gratuit)

### Déploiement

```bash
# 1. Cloner le repo
git clone https://github.com/your-org/koprogo.git
cd koprogo

# 2. Configuration
cp .env.vps.example .env
# Éditer .env avec vos valeurs

# 3. Déployer
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 4. Monitoring
./monitoring/scripts/vps_metrics.sh
./monitoring/scripts/capacity_calculator.sh
```

### Capacité (Validée par Tests de Charge)
- **1,000-1,500 copropriétés** (toutes tailles confondues)
- **30,000-45,000 utilisateurs finaux**
- **Performance réelle** :
  - Throughput : 287 req/s soutenus
  - Success rate : 99.74%
  - Latence P50 : 69ms
  - Latence P90 : 130ms
  - Latence P99 : 752ms
- **RAM** : 128MB utilisés / 2GB (6.3% usage)
- **CPU** : 8% moyen, 25% pic
- **CO₂** : 0.12g/req (mix français 60g CO₂/kWh)

### Monitoring
- Scripts de monitoring VPS (créés)
- Traefik dashboard : `https://api.koprogo.be/dashboard/`
- UptimeRobot (monitoring externe gratuit)

### Quand upgrader ?
Signaux d'alerte :
- ✗ RAM > 85% constant
- ✗ CPU load > 1.5 constant (1 vCPU)
- ✗ Disk > 80%
- ✗ > 1,500 copropriétés actives
- ✗ Query latency P99 > 1000ms

**👉 Passer à Phase 2** (très peu probable avant 1,500+ copros)

---

## Phase 2: Growth - VPS Upgraded (1,500-5,000 copropriétés)

### Infrastructure
- **Hébergement** : OVH Cloud France
- **Coût** : ~15€/mois
- **Specs** : 2 vCPU, 4GB RAM, 80GB SSD
- **Datacenter** : France (souveraineté numérique)
- **Stack** : Identique Phase 1 (Docker Compose)
- **Capacité estimée** : 3,000-5,000 copropriétés

### Migration

```bash
# Option 1: Resize VPS (OVH Control Panel)
# Dans OVH Manager : Upgrade instance

# Option 2: Migration vers nouveau VPS
# 1. Backup DB
docker exec koprogo-postgres pg_dump -U koprogo koprogo_db > backup.sql

# 2. Provisionner nouveau VPS OVH
# 3. Restaurer backup
```

### Optimisations
- Augmenter PostgreSQL `shared_buffers=1GB`
- Backend resource limits : `memory: 500M`
- Activer Traefik metrics Prometheus

### Capacité
- **500-2,000 copropriétés**
- **Latence** : maintenue < 5ms
- **Marge** : Comfortable headroom

### Quand upgrader ?
- > 500 copropriétés
- Besoin de haute disponibilité
- Multiple régions géographiques
- Équipe DevOps dédiée

**👉 Passer à Phase 3**

---

## Phase 3: Scale - Kubernetes Dev (500-2,000 copropriétés)

### Infrastructure (voir `/infrastructure/`)
- **Hébergement** : OVH Cloud - Environnement Dev/Staging
- **Coût** : ~30€/mois (dev) ou ~90€/mois (staging)
- **Specs** :
  - Dev : 1 control plane + 1 worker
  - Staging : 1 control plane + 2 workers
- **Stack** :
  - K3s (Kubernetes léger)
  - Helm charts
  - PostgreSQL HA (Patroni)
  - DragonflyDB (cache)
  - Prometheus + Grafana

### Déploiement

```bash
# Voir infrastructure/README.md pour détails complets

cd infrastructure

# 1. Configuration OVH
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="..."
export OVH_APPLICATION_SECRET="..."
export OVH_CONSUMER_KEY="..."

# 2. Déploiement automatique
./scripts/deploy.sh dev

# 3. Accès cluster
export KUBECONFIG=~/.kube/koprogo-dev
kubectl get nodes
kubectl get pods -A
```

### Avantages
- ✅ Scalabilité horizontale automatique (HPA)
- ✅ Haute disponibilité (multi-node)
- ✅ Rolling updates sans downtime
- ✅ Monitoring avancé (Prometheus/Grafana)
- ✅ GitOps (déploiements reproductibles)

### Capacité
- **500-5,000 copropriétés**
- **Scaling automatique** selon charge
- **Multi-région** possible

### Quand upgrader ?
- > 2,000 copropriétés
- SLA > 99.9% requis
- Compliance stricte
- Équipe technique mature

**👉 Passer à Phase 4**

---

## Phase 4: Enterprise - Kubernetes Production HA (2,000+ copropriétés)

### Infrastructure
- **Hébergement** : OVH Cloud - Production
- **Coût** : ~270€/mois
- **Specs** :
  - 3 control planes (HA etcd)
  - 3 workers (8 vCPU, 30GB RAM chacun)
  - Load Balancer OVH
  - Object Storage (backups)
- **Stack** :
  - K3s multi-master HA
  - PostgreSQL HA cluster (Patroni + HAProxy)
  - DragonflyDB HA
  - Longhorn (storage distribué)
  - Kepler (CO2 monitoring)

### Déploiement

```bash
cd infrastructure
./scripts/deploy.sh prod
```

### Fonctionnalités
- ✅ **SLA 99.95%** : Redondance complète
- ✅ **Auto-scaling** : Pods + Nodes
- ✅ **Disaster Recovery** : Backups automatiques
- ✅ **Multi-AZ** : Disponibilité géographique
- ✅ **Observabilité** : Logs + Metrics + Traces
- ✅ **Sécurité** : Network policies, RBAC, secrets encryption

### Capacité
- **2,000-50,000+ copropriétés**
- **Scaling illimité** (horizontal + vertical)
- **Performance garantie** : SLA contractuels

---

## Comparaison des Phases

| Métrique | Phase 1 (MVP) | Phase 2 (Growth) | Phase 3 (K3s Dev) | Phase 4 (K3s Prod HA) |
|----------|---------------|------------------|-------------------|----------------------|
| **Coût/mois** | 5€ | 15€ | 30-90€ | 270€ |
| **Copropriétés** | 0-100 | 100-500 | 500-2,000 | 2,000+ |
| **vCPU** | 2 | 4 | 6-12 | 36+ |
| **RAM** | 2GB | 4-8GB | 22-60GB | 180GB+ |
| **Haute Dispo** | ✗ | ✗ | ⚠️ (Staging) | ✅ |
| **Auto-scaling** | ✗ | ✗ | ✅ | ✅ |
| **SLA** | Best effort | Best effort | 99.5% | 99.95% |
| **Complexité** | Faible | Faible | Moyenne | Élevée |
| **Équipe requise** | 1 dev | 1 dev | 1-2 DevOps | 2-3 DevOps |

## Configuration Actuelle du Dépôt

### Fichiers de déploiement existants

```
koprogo/
├── docker-compose.yml              # Dev local
├── docker-compose.prod.yml         # Production VPS (Phase 1-2)
├── docker-compose.vps.yml          # VPS optimisé avec monitoring
├── traefik.yml                     # Traefik config dev
├── traefik.prod.yml                # Traefik config prod
├── .env.vps.example               # Variables d'environnement VPS
│
├── monitoring/                     # Scripts monitoring Phase 1-2
│   ├── scripts/
│   │   ├── vps_metrics.sh         # Métriques système
│   │   ├── postgres_metrics.sh    # Métriques DB
│   │   └── capacity_calculator.sh # Estimation capacité
│   └── config/
│       └── thresholds.env         # Seuils d'alerte
│
└── infrastructure/                 # Infrastructure K3s Phase 3-4
    ├── terraform/                 # Provisioning OVH
    ├── ansible/                   # Configuration K3s
    ├── helm/                      # Charts applicatifs
    └── scripts/
        └── deploy.sh              # Déploiement automatique
```

### Utilisation selon la phase

**Phase 1-2 (VPS)** :
```bash
# Utiliser docker-compose.prod.yml
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Monitoring
./monitoring/scripts/vps_metrics.sh
./monitoring/scripts/capacity_calculator.sh
```

**Phase 3-4 (Kubernetes)** :
```bash
# Utiliser infrastructure/
cd infrastructure
./scripts/deploy.sh [dev|staging|prod]
```

## Recommandations Business

### Années 1-2 : Phase 1 (MVP)
- **Focus** : Product-Market Fit
- **Infrastructure** : VPS simple (5€/mois)
- **Effort** : Minimal DevOps
- **Rentabilité** : Immédiate (1 client = break-even)

**Revenue potentiel** :
- 50 copropriétés × 15€/copro = 750€/mois
- Coût infra : 5€/mois
- **Marge : 99%**

### Années 2-3 : Phase 2-3 (Growth + K3s Dev)
- **Focus** : Croissance clients
- **Infrastructure** : VPS upgrade → K3s dev
- **Effort** : DevOps part-time
- **Rentabilité** : Excellente

**Revenue potentiel** :
- 500 copropriétés × 15€ = 7,500€/mois
- Coût infra : 30-90€/mois
- **Marge : 98%**

### Années 3+ : Phase 4 (Enterprise)
- **Focus** : Enterprise deals, SLA
- **Infrastructure** : K3s Production HA
- **Effort** : Équipe DevOps dédiée
- **Rentabilité** : Optimisée

**Revenue potentiel** :
- 2,000 copropriétés × 20€ (pricing higher tier) = 40,000€/mois
- Coût infra : 270€/mois + équipe
- **Marge : 95%+**

## Migration entre Phases

### Phase 1 → Phase 2 (Simple)
```bash
# 1. Snapshot VPS actuel (backup)
# 2. Resize VPS dans console cloud
# 3. Update docker-compose resource limits
# 4. Restart services
```

### Phase 2 → Phase 3 (Complexe)
Voir guide détaillé : `docs/MIGRATION_VPS_TO_K3S.md` (TODO)

Étapes :
1. Provisionner cluster K3s (Terraform)
2. Déployer apps (Helm)
3. Migrer DB (pg_dump/restore)
4. Basculer DNS
5. Monitoring coupure

**Downtime estimé** : 30 minutes à 2 heures

### Phase 3 → Phase 4 (Progressive)
Migration progressive sans downtime :
1. Provisionner infra prod
2. Réplication DB staging → prod
3. Blue/Green deployment
4. Bascule DNS progressive

**Downtime** : 0 (si bien exécuté)

## Monitoring & Alertes

### Phase 1-2 : Scripts Custom
- `vps_metrics.sh` (cron every 5 min)
- `postgres_metrics.sh` (cron hourly)
- UptimeRobot (uptime monitoring)
- Email alerts (via cron)

### Phase 3-4 : Prometheus + Grafana
- Métriques système & applicatives
- Dashboards visuels
- Alertmanager (Slack/PagerDuty)
- Logs centralisés (Loki)

## Support

- **Phase 1-2** : `docs/VPS_DEPLOYMENT.md`
- **Phase 3-4** : `infrastructure/README.md`
- **Monitoring** : `monitoring/README.md`
- **Architecture** : `CLAUDE.md`

---

**Conseil** : Commencez Phase 1, validez le marché, puis scalez progressivement selon la croissance réelle. Ne pas sur-investir en infra avant d'avoir les clients !

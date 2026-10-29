# Architecture des tests de charge

Schéma d'organisation pour les tests de performance du VPS.

## Architecture recommandée

```
┌─────────────────────────────────────────────────────────────────┐
│                    TESTS DE CHARGE                              │
│                                                                 │
│  ┌───────────────────────┐          ┌────────────────────────┐ │
│  │  Machine Cliente      │          │   VPS Serveur          │ │
│  │  (votre ordinateur    │          │   (1 vCPU / 2GB RAM)   │ │
│  │   ou VPS client)      │          │                        │ │
│  │                       │          │                        │ │
│  │  ┌─────────────────┐  │  HTTPS   │  ┌──────────────────┐ │ │
│  │  │ k6              │──┼─────────>│  │ Traefik          │ │ │
│  │  │ (génère charge) │  │          │  │ (reverse proxy)  │ │ │
│  │  └─────────────────┘  │          │  └────────┬─────────┘ │ │
│  │                       │          │           │           │ │
│  │  Scripts:             │          │  ┌────────▼─────────┐ │ │
│  │  - remote-light-load  │<─results─┤  │ Backend API      │ │ │
│  │  - remote-medium-load │          │  │ (Rust/Actix)     │ │ │
│  │                       │          │  └────────┬─────────┘ │ │
│  └───────────────────────┘          │           │           │ │
│                                     │  ┌────────▼─────────┐ │ │
│                                     │  │ PostgreSQL       │ │ │
│                                     │  │ (256MB RAM)      │ │ │
│                                     │  └──────────────────┘ │ │
│                                     │                        │ │
│                                     │  Monitoring:           │ │
│                                     │  - monitor-server.sh   │ │
│                                     │  - docker stats        │ │
│                                     │  - logs                │ │
│                                     └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Composants

### Machine Cliente (Génération de charge)

**Rôle** : Génère des requêtes HTTP pour simuler des utilisateurs

**Outils** :
- `k6` 2.3.0 : scénarios JavaScript de `k6/`, même moteur que Sluis

**Scripts fournis** :
```
load-tests/
├── scripts/
│   ├── remote-light-load.sh      # 10 connexions, 2 min
│   ├── remote-medium-load.sh     # 50 connexions, 5 min
│   └── ...
├── k6/
│   ├── commun.js                 # Session par VU, tirage pondéré, résumé (SLUIS_RESUME)
│   ├── health.js                 # Chauffe : GET /health
│   ├── auth.js                   # Connexions répétées
│   ├── buildings.js              # Focus sur /buildings (sans session)
│   ├── mixed.js                  # Lectures pondérées (sans session)
│   ├── authenticated-mixed.js    # Lectures pondérées (avec session)
│   └── authenticated-realistic.js # 80 % lectures / 20 % écritures
└── results/                      # Résultats des tests
```

**Configuration** :
```bash
export BASE_URL=https://api.votredomaine.com
```

### VPS Serveur (Application)

**Rôle** : Reçoit et traite les requêtes

**Architecture** :
```
Traefik (reverse proxy, SSL, rate limiting)
    ↓
Backend API (Actix-web, 1 worker, 384MB RAM)
    ↓
PostgreSQL (256MB RAM, 15 max_connections)
```

**Monitoring** :
```bash
# Script automatique
./monitor-server.sh 300

# Manuel
docker stats
docker compose logs -f backend
```

**Logs générés** :
```
load-tests/
└── monitoring-results/
    ├── server-monitoring_20250124_143022.log
    └── ...
```

## Flux d'un test type

### 1. Préparation

**Client** :
```bash
cd koprogo/load-tests
export BASE_URL=https://api.votredomaine.com
curl $BASE_URL/api/v1/health  # Vérifier connectivité
```

**Serveur** :
```bash
ssh user@vps-ip
cd /opt/koprogo
docker compose -f docker-compose.vps.yml ps  # Vérifier status
```

### 2. Lancement simultané

**Terminal 1 (Client)** :
```bash
./scripts/remote-medium-load.sh
# Durée: 5 minutes
# Threads: 4
# Connexions: 50
```

**Terminal 2 (Serveur - Monitoring)** :
```bash
ssh user@vps-ip
cd /opt/koprogo/load-tests
./monitor-server.sh 300  # 5 minutes
```

**Terminal 3 (Serveur - Logs)** :
```bash
ssh user@vps-ip
docker compose -f docker-compose.vps.yml logs -f backend
```

### 3. Collecte des résultats

**Client** :
```
results/remote-medium-load_20250124_143022.txt
  → Latences (P50, P95, P99)
  → Throughput (req/s)
  → Taux d'erreur
```

**Serveur** :
```
monitoring-results/server-monitoring_20250124_143022.log
  → CPU usage par service
  → RAM usage par service
  → Connexions DB actives
  → Erreurs dans les logs
```

### 4. Analyse

Comparer :
- **Latence P99** : doit être < 100ms (light) ou < 200ms (medium)
- **Throughput** : doit être > 100 req/s (light) ou > 500 req/s (medium)
- **Erreurs** : doit être < 0.5%
- **CPU VPS** : doit être < 80% (light) ou < 95% (medium)
- **RAM VPS** : doit être stable (pas de croissance continue)

## Scénarios de test

### Light Load
```
Threads: 2
Connexions: 10
Durée: 2 minutes
Charge: ~100 req/s
```
**Objectif** : Valider le fonctionnement de base

### Medium Load
```
Threads: 4
Connexions: 50
Durée: 5 minutes
Charge: ~500 req/s
```
**Objectif** : Tester les performances sous charge normale

### Heavy Load
```
Threads: 4
Connexions: 100
Durée: 3 minutes
Charge: ~1000 req/s
```
**Objectif** : Trouver le point de rupture

### Spike Test
```
Phases:
  1. Baseline (10 conn) - 30s
  2. Ramp up (50 conn) - 30s
  3. SPIKE (200 conn) - 60s
  4. Recovery (50 conn) - 30s
  5. Baseline (10 conn) - 30s
```
**Objectif** : Tester la résilience aux pics soudains

### Soak Test
```
Threads: 2
Connexions: 25
Durée: 30 minutes
Charge: ~250 req/s soutenue
```
**Objectif** : Détecter les fuites mémoire

## Configuration réseau

### Latence attendue

| Source | Destination | Latency |
|--------|-------------|---------|
| Même datacenter | VPS | < 1ms |
| Même région | VPS | < 10ms |
| Même continent | VPS | < 50ms |
| Autre continent | VPS | 100-200ms |

**Impact sur les résultats** :
```
Latence mesurée = Latence app + Latence réseau

Exemple avec 20ms de latence réseau :
  - P99 app = 50ms
  - P99 réseau = 20ms
  - P99 total mesuré = 70ms
```

### Bande passante requise

| Test | Requests/s | Taille requête | Taille réponse | Bande passante |
|------|-----------|----------------|----------------|----------------|
| Light | 100 | ~500 bytes | ~2 KB | ~0.2 Mbps |
| Medium | 500 | ~500 bytes | ~2 KB | ~1 Mbps |
| Heavy | 1000 | ~500 bytes | ~2 KB | ~2 Mbps |

**Recommandation** : Connexion > 10 Mbps pour la machine cliente

## Comparaison local vs distant

### Tests locaux (sur le VPS lui-même)

❌ **À ÉVITER en production**

```
VPS (2GB RAM)
├── Backend (consomme 384MB)
├── PostgreSQL (consomme 256MB)
├── k6 (consomme 100-200MB)  ← Fausse les résultats !
└── Système (reste ~1.2GB)

Problèmes :
- k6 utilise CPU → réduit CPU disponible pour l'app
- k6 utilise RAM → peut déclencher swap
- Latence = 0ms → pas réaliste
- Résultats faussés
```

### Tests distants (depuis machine externe)

✅ **RECOMMANDÉ**

```
Machine Cliente        VPS (2GB RAM)
├── k6 (charge)   →    ├── Backend (384MB)
└── Monitoring         ├── PostgreSQL (256MB)
                       └── Système (1.3GB libre)

Avantages :
- Ressources VPS 100% dédiées à l'app
- Latence réseau réaliste
- Résultats fiables
- Isolation complète
```

## Options de machine cliente

### Option 1 : Votre ordinateur

**Avantages** :
- ✅ Gratuit
- ✅ Facile
- ✅ Contrôle immédiat

**Inconvénients** :
- ⚠️ Latence variable (dépend de votre connexion)
- ⚠️ Charge limitée (dépend de votre machine)

**Recommandé pour** : Tests de validation, debug

### Option 2 : VPS client (même datacenter)

**Avantages** :
- ✅ Latence faible (<5ms)
- ✅ Bande passante élevée
- ✅ Charge maximale possible
- ✅ Résultats stables

**Inconvénients** :
- 💰 Coût (~3-5€/mois)

**Recommandé pour** : Tests de performance sérieux, benchmarks

### Option 3 : k6 Cloud

**Avantages** :
- ✅ Multi-régions
- ✅ Scalable
- ✅ Pas d'infrastructure

**Inconvénients** :
- 💰 Coût (gratuit limité, puis ~50€/mois)

**Recommandé pour** : CI/CD, tests multi-régions

## Questions fréquentes

### Pourquoi pas Docker Compose pour k6 ?

Réponse : Possible, mais :
- Ajoute une couche réseau Docker
- Utilise les ressources du VPS
- Pas pratique pour monitoring temps réel

### Peut-on tester en HTTP au lieu de HTTPS ?

Réponse : Oui en développement, mais :
- En production, toujours tester HTTPS (overhead SSL réel)
- Modifier `BASE_URL=http://...` si vraiment nécessaire

### Combien de temps entre deux tests ?

Réponse : Attendre 1-2 minutes entre tests pour :
- Laisser les connexions DB se fermer
- Laisser les caches se stabiliser
- Éviter les faux positifs

### Les tests impactent-ils les utilisateurs ?

Réponse :
- Light/Medium : Impact minime
- Heavy : Peut ralentir l'app
- **Recommandation** : Tester en heures creuses

## Voir aussi

- [REMOTE_TESTING.md](REMOTE_TESTING.md) : Guide complet des tests distants
- [QUICKSTART.md](QUICKSTART.md) : Démarrage rapide (tests locaux)
- [README.md](README.md) : Documentation complète

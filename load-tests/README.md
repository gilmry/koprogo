# Tests de montée en charge KoproGo

Tests de performance et de charge pour un VPS 1 vCPU / 2GB RAM.

## 🚨 IMPORTANT : Rate Limiting et tests de charge

**Le rate limiting fausse les résultats des tests de charge !**

Par défaut, l'API limite à **100 requêtes par minute par IP**. Pour des tests de charge précis, vous devez désactiver le rate limiting.

### Configuration pour les tests de charge

**Option 1 : Docker Compose (Recommandé)**

```bash
# Démarrer l'API avec docker-compose.loadtest.yml
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d

# Cette configuration :
# ✅ Désactive automatiquement le rate limiting (ENABLE_RATE_LIMITING=false)
# ✅ Optimise PostgreSQL pour la performance
# ✅ Augmente les workers et les connexions DB
```

**Option 2 : Variable d'environnement manuelle**

```bash
# Dans backend/.env ou backend/.env.vps
ENABLE_RATE_LIMITING=false

# Puis redémarrer l'API
docker compose restart backend
```

⚠️ **N'oubliez pas de réactiver le rate limiting en production !**

```bash
ENABLE_RATE_LIMITING=true  # ou supprimer la ligne (true par défaut)
```

## ⚠️ Important : Tests locaux vs distants

**Deux modes de test :**

1. **Tests LOCAUX** (`scripts/*.sh`) :
   - ❌ À éviter en production
   - ✅ OK pour développement local (docker-compose.yml)
   - Les tests tournent sur la même machine que l'API

2. **Tests DISTANTS** (`scripts/remote-*.sh`) :
   - ✅ **RECOMMANDÉ pour VPS production**
   - Les tests tournent depuis une machine cliente externe
   - Résultats plus réalistes (inclut latence réseau)
   - Ne consomme pas les ressources du serveur

**Pour tester votre VPS en production → Voir [REMOTE_TESTING.md](REMOTE_TESTING.md)**

## Démarrage rapide (Quick Start)

```bash
# 1. Démarrer l'environnement de test (rate limiting désactivé)
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d

# 2. Attendre que les services soient prêts (20-30s)
docker compose -f docker-compose.loadtest.yml ps

# 3. Vérifier la santé de l'API
curl http://localhost:8080/api/v1/health
# Devrait retourner: {"status":"healthy"}

# 4. Lancer un test simple
export BASE_URL=http://localhost:8080
./scripts/warmup.sh
./scripts/light-load.sh

# 5. Nettoyer après les tests
docker compose -f docker-compose.loadtest.yml down
```

Consultez les logs pour confirmer que le rate limiting est désactivé :
```bash
docker compose -f docker-compose.loadtest.yml logs backend | grep "Rate limiting"
# Devrait afficher: "Rate limiting enabled: false"
```

## Objectifs de performance

### Configuration VPS : 1 vCPU / 2GB RAM

| Métrique | Cible | Limite acceptable |
|----------|-------|-------------------|
| Latence P50 | < 10ms | < 20ms |
| Latence P95 | < 50ms | < 100ms |
| Latence P99 | < 100ms | < 200ms |
| Throughput | > 500 req/s | > 250 req/s |
| Taux d'erreur | < 0.1% | < 1% |
| Utilisation CPU | < 80% | < 95% |
| Utilisation RAM | < 80% | < 90% |

## Outils de test

### 1. wrk (Recommandé)

```bash
# Installation
sudo apt-get install wrk
```

### 2. hey (Alternative)

```bash
# Installation
go install github.com/rakyll/hey@latest
```

### 3. k6 (Tests avancés)

```bash
# Installation
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

## Configuration URL

**Par défaut**, tous les scripts ciblent `https://api.koprogo.com`.

**Pour tester un autre domaine** :
```bash
export BASE_URL=https://api.votredomaine.com
```

**Pour développement local** :
```bash
export BASE_URL=http://localhost:8080
```

## Scénarios de test

### Scénario 1 : Test de warmup (préparer le système)

```bash
# Cible api.koprogo.com par défaut
./scripts/warmup.sh

# Ou pour un autre domaine
export BASE_URL=https://votredomaine.com
./scripts/warmup.sh
```

### Scénario 2 : Test de charge légère (usage normal)

```bash
./scripts/light-load.sh
```

**Profil :**
- 10 connexions concurrentes
- Durée : 2 minutes
- Cible : 100 req/s

### Scénario 3 : Test de charge moyenne

```bash
./scripts/medium-load.sh
```

**Profil :**
- 50 connexions concurrentes
- Durée : 5 minutes
- Cible : 500 req/s

### Scénario 4 : Test de charge maximale

```bash
./scripts/heavy-load.sh
```

**Profil :**
- 100 connexions concurrentes
- Durée : 3 minutes
- Cible : 1000 req/s (cherche le point de rupture)

### Scénario 5 : Test de stress (spike test)

```bash
./scripts/spike-test.sh
```

**Profil :**
- Montée rapide de 0 à 200 connexions
- Maintien 1 minute
- Descente rapide
- Vérifie la récupération du système

### Scénario 6 : Test d'endurance (soak test)

```bash
./scripts/soak-test.sh
```

**Profil :**
- 25 connexions concurrentes
- Durée : 30 minutes
- Vérifie les fuites mémoire et la stabilité

## Lancer tous les tests

```bash
./run-all-tests.sh
```

Génère un rapport complet dans `results/`.

## Monitoring pendant les tests

### Terminal 1 : Lancer le test

```bash
./scripts/medium-load.sh
```

### Terminal 2 : Monitorer les ressources

```bash
watch -n 1 'docker stats --no-stream'
```

### Terminal 3 : Monitorer les logs

```bash
docker compose -f docker-compose.vps.yml logs -f backend
```

### Terminal 4 : Monitorer PostgreSQL

```bash
docker compose -f docker-compose.vps.yml exec postgres psql -U koprogo -d koprogo_db -c "
SELECT
    pid,
    usename,
    application_name,
    state,
    query_start,
    LEFT(query, 50) as query
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY query_start;"
```

## Analyse des résultats

### Métriques clés à surveiller

1. **Latence** : Vérifier P50, P95, P99
2. **Throughput** : Requests/sec réalisés vs cibles
3. **Erreurs** : Taux d'erreur (timeouts, 5xx)
4. **Ressources** :
   - CPU : `docker stats`
   - RAM : `docker stats`
   - Connexions DB : Voir requête PostgreSQL ci-dessus

### Interpréter les résultats

**Bon** ✅
- P99 < 100ms
- Throughput > 500 req/s
- Erreurs < 0.1%
- CPU < 80%

**Acceptable** ⚠️
- P99 < 200ms
- Throughput > 250 req/s
- Erreurs < 1%
- CPU < 95%

**Problématique** ❌
- P99 > 200ms
- Throughput < 250 req/s
- Erreurs > 1%
- CPU = 100% ou OOM

## Optimisations possibles

Si les tests ne passent pas les seuils :

### 1. Ajuster les workers Actix

```env
ACTIX_WORKERS=1  # Déjà optimal pour 1 vCPU
```

### 2. Ajuster le pool de connexions

```env
DB_POOL_MAX_CONNECTIONS=8   # Réduire à 5 si trop de contention
DB_POOL_MIN_CONNECTIONS=2   # Augmenter à 4 si latence de connexion élevée
```

### 3. Optimiser PostgreSQL

```bash
# Réduire shared_buffers si RAM saturée
docker compose -f docker-compose.vps.yml down
# Éditer docker-compose.vps.yml : shared_buffers=128MB (au lieu de 256MB)
docker compose -f docker-compose.vps.yml up -d
```

### 4. Activer le cache HTTP

Voir `nginx-cache-config/` pour activer le cache nginx au niveau reverse proxy.

### 5. Limiter le rate limiting

Si trop restrictif, ajuster dans docker-compose.vps.yml :

```yaml
- "traefik.http.middlewares.rate-limit.ratelimit.average=200"  # 100 → 200
```

## CI/CD : Tests automatiques

Les tests de charge sont exécutés automatiquement :

- **PR** : Scénario light (2 min)
- **Merge to main** : Scénario medium (5 min)
- **Nightly** : Scénario soak (30 min)

Voir `.github/workflows/load-tests.yml`

## Troubleshooting

### Erreur : Connection refused

```bash
# Vérifier que l'API est démarrée
docker compose -f docker-compose.vps.yml ps
curl http://localhost:8080/api/v1/health
```

### Erreur : Too many open files

```bash
# Augmenter les limites du système
ulimit -n 65536
```

### Erreur : Cannot allocate memory

```bash
# Vérifier le swap
swapon --show
free -h

# Ajouter du swap si nécessaire (voir DEPLOYMENT_VPS.md)
```

## Ressources

- [wrk documentation](https://github.com/wg/wrk)
- [hey documentation](https://github.com/rakyll/hey)
- [k6 documentation](https://k6.io/docs/)
- [PostgreSQL performance tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)

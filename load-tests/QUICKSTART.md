# Quick Start - Tests de charge KoproGo

Guide rapide pour lancer les tests de charge sur votre VPS 1 vCPU / 2GB RAM.

## ⚠️ Important : Où lancer les tests ?

**Vous avez un VPS en production ?**
→ ✅ Lisez plutôt [REMOTE_TESTING.md](REMOTE_TESTING.md) pour tester depuis une machine externe

**Vous développez en local ?**
→ ✅ Ce guide est pour vous (tests sur localhost)

---

## Prérequis

```bash
# Installer wrk (load testing tool)
sudo apt-get update
sudo apt-get install -y wrk

# Vérifier l'installation
wrk --version
```

## 🚨 IMPORTANT : Rate Limiting

**Le rate limiting de l'API (100 req/min) fausse les tests de charge !**

Pour des tests précis, utilisez `docker-compose.loadtest.yml` qui désactive automatiquement le rate limiting :

```bash
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d
export BASE_URL=http://localhost:8080
./scripts/light-load.sh
```

Voir [RATE_LIMITING_SOLUTION.md](RATE_LIMITING_SOLUTION.md) pour plus de détails.

---

## Étape 1 : Démarrer l'application

```bash
cd /opt/koprogo  # ou votre répertoire

# Démarrer avec docker-compose.vps.yml
docker compose -f docker-compose.vps.yml --env-file .env.vps up -d

# Vérifier que tout est up
docker compose -f docker-compose.vps.yml ps
```

## Étape 2 : Configurer l'URL cible

```bash
# Pour développement local (obligatoire si vous testez localhost)
export BASE_URL=http://localhost:8080

# Vérifier la santé de l'API
curl $BASE_URL/api/v1/health

# Devrait retourner: {"status":"healthy"}
```

**Note** : Par défaut, les scripts ciblent `https://api.koprogo.com`. Si vous testez en local, vous **devez** définir `BASE_URL=http://localhost:8080`.

## Étape 3 : Lancer un test simple

```bash
cd load-tests

# Test de warmup (prépare le système)
./scripts/warmup.sh

# Test de charge légère (2 minutes)
./scripts/light-load.sh
```

## Tests disponibles

| Script | Durée | Charge | Usage |
|--------|-------|--------|-------|
| `warmup.sh` | 30s | Très légère | Toujours lancer avant les autres tests |
| `light-load.sh` | 2 min | Légère (100 req/s) | Usage quotidien normal |
| `medium-load.sh` | 5 min | Moyenne (500 req/s) | Pics de trafic |
| `heavy-load.sh` | 3 min | Maximale (1000 req/s) | Trouver les limites |
| `spike-test.sh` | 5 min | Variable | Montée subite de trafic |
| `soak-test.sh` | 30 min | Soutenue | Détection de fuites mémoire |

## Exemple : Test complet

```bash
# Terminal 1 : Lancer le test
cd load-tests
./scripts/warmup.sh
./scripts/medium-load.sh

# Terminal 2 : Monitorer les ressources
watch -n 1 'docker stats --no-stream'

# Terminal 3 : Logs de l'API
docker compose -f ../docker-compose.vps.yml logs -f backend
```

## Résultats attendus (1 vCPU / 2GB RAM)

### ✅ Light Load

```
Latency P50: ~5-10ms
Latency P99: <50ms
Throughput: >100 req/s
Errors: <0.1%
CPU: ~30-40%
```

### ✅ Medium Load

```
Latency P50: ~10-20ms
Latency P99: <100ms
Throughput: >500 req/s
Errors: <0.5%
CPU: ~70-80%
```

### ⚠️ Heavy Load

```
Latency P50: ~20-50ms
Latency P99: <200ms
Throughput: Plateaux possible
Errors: <5% acceptable
CPU: 95-100%
```

## Interpréter les résultats

### Exemple de sortie wrk

```
Running 2m test @ http://localhost:8080
  2 threads and 10 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     8.32ms    4.12ms  89.43ms   87.23%
    Req/Sec   620.45    112.34     1.02k    76.56%
  Latency Distribution
     50%    7.21ms
     75%   10.12ms
     90%   13.45ms
     99%   22.18ms
  148562 requests in 2.00m, 42.31MB read
Requests/sec:   1238.02
Transfer/sec:    361.14KB
```

**Analyse :**
- ✅ P99 = 22ms → Excellent (< 50ms cible)
- ✅ 1238 req/s → Au-dessus de la cible (100 req/s)
- ✅ 0 erreurs → Parfait
- ✅ Résultats stables (faible Stdev)

## Problèmes courants

### ❌ "wrk: command not found"

```bash
sudo apt-get install wrk
```

### ❌ "Connection refused"

```bash
# L'API n'est pas démarrée
docker compose -f docker-compose.vps.yml up -d

# Attendre le health check
sleep 10
curl http://localhost:8080/api/v1/health
```

### ❌ "Too many errors"

```bash
# Réduire la charge
# Modifier le script pour moins de connexions (-c)

# Ou augmenter les limites système
ulimit -n 65536
```

### ❌ Latence très élevée (>500ms)

Causes possibles :
1. CPU saturé → Réduire la charge
2. Base de données lente → Vérifier les index
3. Pool de connexions saturé → Augmenter `DB_POOL_MAX_CONNECTIONS`
4. Swap utilisé → Ajouter plus de RAM ou optimiser

```bash
# Vérifier le swap
swapon --show
free -h

# Vérifier les connexions DB
docker compose -f ../docker-compose.vps.yml exec postgres psql -U koprogo -d koprogo_db -c "SELECT count(*) FROM pg_stat_activity;"
```

## Lancer tous les tests

```bash
# Lance tous les tests (sauf soak) et génère un rapport
./run-all-tests.sh

# Résultats dans :
ls -lh results/
```

## Configuration avancée

### Changer l'URL cible

```bash
export BASE_URL=http://api.example.com
./scripts/medium-load.sh
```

### Ajuster les paramètres

Éditer les scripts dans `scripts/` :

```bash
# Exemple : Réduire la charge du medium-load
nano scripts/medium-load.sh

# Changer :
wrk -t4 -c50 -d5m ...
# En :
wrk -t2 -c25 -d5m ...
```

## Automatisation CI/CD

Les tests peuvent être intégrés dans votre CI/CD :

```yaml
# .github/workflows/load-tests.yml
- name: Run load tests
  run: |
    cd load-tests
    ./scripts/light-load.sh
```

## Support

Pour plus de détails, voir `README.md` dans ce répertoire.

Pour des tests avancés avec k6 ou monitoring Grafana, consulter la documentation complète.

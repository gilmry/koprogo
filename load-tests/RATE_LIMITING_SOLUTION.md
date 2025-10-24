# Solution : Désactiver le Rate Limiting pour les Load Tests

## Problème

Le rate limiting de l'API KoproGo (100 requêtes/minute par IP) fausse complètement les résultats des tests de charge.

**Configuration actuelle** (`backend/src/main.rs:97-103`) :
```rust
let governor_conf = GovernorConfigBuilder::default()
    .milliseconds_per_request(100 * 60 * 1000) // 100 req/min = 6000ms/req
    .burst_size(100)
    .finish()
    .unwrap();
```

**Impact** :
- Burst initial : 100 requêtes immédiatement
- Après le burst : ~1.67 req/s (100 req / 60 secondes)
- Vos tests ciblent 100-1000 req/s → impossible !

## Solution implémentée

### Option 1 : Docker Compose Load Testing (RECOMMANDÉ)

Utiliser `docker-compose.loadtest.yml` qui désactive automatiquement le rate limiting.

```bash
# 1. Démarrer l'environnement de test
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d

# 2. Vérifier que le rate limiting est désactivé
docker compose -f docker-compose.loadtest.yml logs backend | grep "Rate limiting"
# Devrait afficher: "Rate limiting enabled: false"

# 3. Lancer les tests
export BASE_URL=http://localhost:8080
./scripts/warmup.sh
./scripts/light-load.sh
./scripts/medium-load.sh

# 4. Nettoyer
docker compose -f docker-compose.loadtest.yml down
```

**Avantages** :
- ✅ Rate limiting désactivé automatiquement
- ✅ PostgreSQL optimisé pour la performance
- ✅ Configuration isolée (ne touche pas à la production)
- ✅ Facile à démarrer/arrêter

### Option 2 : Variable d'environnement manuelle

Pour tester sur un VPS en production (à utiliser avec PRÉCAUTION).

```bash
# 1. Sur le VPS, éditer le fichier .env
nano /opt/koprogo/backend/.env.vps

# 2. Ajouter cette ligne
ENABLE_RATE_LIMITING=false

# 3. Redémarrer le backend
docker compose -f docker-compose.vps.yml restart backend

# 4. Vérifier dans les logs
docker compose -f docker-compose.vps.yml logs backend | grep "Rate limiting"
# Devrait afficher: "Rate limiting enabled: false"

# 5. IMPORTANT: Lancer les tests IMMÉDIATEMENT

# 6. IMPORTANT: Réactiver après les tests !
sed -i '/ENABLE_RATE_LIMITING/d' /opt/koprogo/backend/.env.vps
docker compose -f docker-compose.vps.yml restart backend
```

### Option 3 : Script helper

Utiliser le script `toggle-rate-limiting.sh` pour simplifier.

```bash
cd load-tests

# Désactiver le rate limiting
./toggle-rate-limiting.sh off

# Vérifier le status
./toggle-rate-limiting.sh status

# Lancer les tests
export BASE_URL=https://api.votredomaine.com
./scripts/remote-medium-load.sh

# Réactiver (IMPORTANT !)
./toggle-rate-limiting.sh on
```

## Vérification

### Avant le test

```bash
# Vérifier que le rate limiting est désactivé
curl -s http://localhost:8080/api/v1/health

# Lancer 200 requêtes rapides (devrait passer si désactivé)
for i in {1..200}; do
    curl -s http://localhost:8080/api/v1/health > /dev/null &
done
wait

# Si rate limiting activé : beaucoup d'erreurs 429
# Si désactivé : toutes les requêtes passent
```

### Logs du backend

```bash
# Avec docker-compose.loadtest.yml
docker compose -f docker-compose.loadtest.yml logs backend | grep -A2 "Starting server"

# Devrait afficher :
# Rate limiting enabled: false
# Starting server at 0.0.0.0:8080 with 4 workers
```

## Résultats attendus

### Avec rate limiting activé (AVANT)

```
Latency P99: >1000ms (throttling)
Throughput: ~100 req/s max (bloqué par rate limiter)
Errors: 429 Too Many Requests après le burst
```

### Avec rate limiting désactivé (APRÈS)

```
Latency P99: <100ms (limité par CPU/DB, pas par throttling)
Throughput: 500-1000 req/s (dépend du hardware)
Errors: <1% (vraies erreurs, pas du rate limiting)
```

## Architecture de la solution

### Fichiers modifiés

1. **backend/src/main.rs**
   - Ajout de `ENABLE_RATE_LIMITING` (default: true)
   - Application conditionnelle du Governor middleware
   - Log du status au démarrage

2. **backend/.env.loadtest** (nouveau)
   - Configuration dédiée aux load tests
   - `ENABLE_RATE_LIMITING=false`
   - Workers et connexions DB augmentés
   - RUST_LOG=error (moins verbose)

3. **load-tests/docker-compose.loadtest.yml**
   - Configuration standalone complète
   - PostgreSQL optimisé (512MB shared_buffers, 50 connections)
   - Backend avec env_file vers .env.loadtest
   - Port 8080 exposé pour tests locaux

4. **load-tests/toggle-rate-limiting.sh** (nouveau)
   - Script helper pour activer/désactiver facilement
   - Gestion des erreurs et vérifications
   - Warnings de sécurité

### Code principal (backend/src/main.rs)

```rust
// Ligne 32-37 : Configuration
let enable_rate_limiting = env::var("ENABLE_RATE_LIMITING")
    .unwrap_or_else(|_| "true".to_string())
    .to_lowercase()
    .parse::<bool>()
    .unwrap_or(true);

log::info!("Rate limiting enabled: {}", enable_rate_limiting);

// Ligne 128-134 : Application conditionnelle
let mut app = App::new()
    .app_data(app_state.clone());

if enable_rate_limiting {
    app = app.wrap(Governor::new(&governor_conf));
}

app.wrap(cors)
   .wrap(middleware::Logger::default())
   .configure(configure_routes)
```

## Sécurité

### Par défaut : ACTIVÉ

Si `ENABLE_RATE_LIMITING` n'est pas défini, le rate limiting est **ACTIVÉ** (true par défaut).

```rust
.unwrap_or(true);  // Sécurité par défaut
```

### Production : TOUJOURS activé

**NE JAMAIS** déployer en production avec `ENABLE_RATE_LIMITING=false` !

Le rate limiting protège contre :
- Attaques DDoS
- Abus d'API
- Scraping excessif
- Boucles infinies de clients

### Tests : Temporairement désactivé

Désactivé UNIQUEMENT pendant la durée des tests de charge, puis immédiatement réactivé.

## Workflow complet

### Tests locaux

```bash
# Terminal 1 : Démarrer l'environnement
cd load-tests
docker compose -f docker-compose.loadtest.yml up

# Terminal 2 : Lancer les tests
export BASE_URL=http://localhost:8080
./scripts/warmup.sh
./run-all-tests.sh

# Terminal 1 : Ctrl+C puis nettoyer
docker compose -f docker-compose.loadtest.yml down
```

### Tests production VPS

```bash
# Machine cliente : Préparer
export BASE_URL=https://api.votredomaine.com

# VPS : Désactiver rate limiting
ssh user@vps-ip
cd /opt/koprogo/load-tests
./toggle-rate-limiting.sh off
docker compose -f ../docker-compose.vps.yml restart backend

# Machine cliente : Lancer les tests
./scripts/remote-medium-load.sh

# VPS : RÉACTIVER immédiatement !
./toggle-rate-limiting.sh on
docker compose -f ../docker-compose.vps.yml restart backend
```

## Troubleshooting

### Toujours du rate limiting malgré ENABLE_RATE_LIMITING=false

**Cause possible** : Traefik rate limiting (docker-compose.vps.yml:171-172)

```yaml
- "traefik.http.middlewares.rate-limit.ratelimit.average=100"
- "traefik.http.middlewares.rate-limit.ratelimit.burst=200"
```

**Solution** : Pour les tests locaux, utiliser `docker-compose.loadtest.yml` qui n'a pas Traefik.

Pour VPS en production, commenter temporairement ces lignes dans `docker-compose.vps.yml`.

### Backend ne démarre pas

```bash
# Vérifier les logs
docker compose -f docker-compose.loadtest.yml logs backend

# Erreur possible : "invalid type: string \"false\""
# → Vérifier que .env.loadtest contient bien :
ENABLE_RATE_LIMITING=false  # pas "false" entre guillemets
```

### Tests toujours lents

```bash
# 1. Vérifier que le rate limiting est bien désactivé
docker compose logs backend | grep "Rate limiting"

# 2. Vérifier les connexions DB
docker compose exec postgres psql -U koprogo -d koprogo_db -c "SHOW max_connections;"

# 3. Vérifier les ressources
docker stats --no-stream
```

## Documentation

- **Quick Start** : `load-tests/START_HERE.md`
- **README complet** : `load-tests/README.md`
- **Tests distants** : `load-tests/REMOTE_TESTING.md`
- **Changelog** : `load-tests/CHANGELOG_RATE_LIMITING.md`

## Références

### Code source
- `backend/src/main.rs:32-37` - Configuration ENABLE_RATE_LIMITING
- `backend/src/main.rs:128-134` - Application conditionnelle
- `backend/src/main.rs:105-111` - Configuration Governor

### Configuration
- `backend/.env.loadtest` - Env pour load tests
- `load-tests/docker-compose.loadtest.yml` - Docker Compose

### Scripts
- `load-tests/toggle-rate-limiting.sh` - Helper script
- `load-tests/scripts/*.sh` - Tests de charge

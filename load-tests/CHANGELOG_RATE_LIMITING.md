# Changelog - Configuration du Rate Limiting pour Load Tests

Date : 2025-10-24

## Problème identifié

Le rate limiting de l'API (100 requêtes/minute par IP) faussait complètement les résultats des tests de charge, limitant artificiellement le throughput à ~1.67 req/s après le burst initial.

Les tests visaient 100-1000 req/s mais étaient throttlés par le rate limiter avant même d'atteindre les limites réelles du système.

## Solution implémentée

### 1. Rate limiting configurable (backend/src/main.rs)

Ajout d'une variable d'environnement `ENABLE_RATE_LIMITING` :

```rust
// Rate limiting configuration
let enable_rate_limiting = env::var("ENABLE_RATE_LIMITING")
    .unwrap_or_else(|_| "true".to_string())
    .to_lowercase()
    .parse::<bool>()
    .unwrap_or(true);

// Conditionally apply rate limiting
if enable_rate_limiting {
    app = app.wrap(Governor::new(&governor_conf));
}
```

**Par défaut : activé (sécurité)**
**Pour load tests : désactivé**

### 2. Configuration dédiée aux load tests (backend/.env.loadtest)

Nouveau fichier avec configuration optimisée :

```bash
ENABLE_RATE_LIMITING=false
RUST_LOG=error  # Moins verbose
ACTIX_WORKERS=4
DB_POOL_MAX_CONNECTIONS=20
```

### 3. Docker Compose pour load testing (load-tests/docker-compose.loadtest.yml)

Configuration standalone complète :
- PostgreSQL optimisé (512MB shared_buffers, 50 max_connections)
- Backend avec rate limiting désactivé
- Exposition du port 8080 pour tests locaux
- Ressources augmentées (1G RAM, 2 CPU)

### 4. Documentation mise à jour

Fichiers modifiés :
- `load-tests/README.md` : Section démarrage rapide avec avertissement rate limiting
- `load-tests/START_HERE.md` : Instructions en haut de page
- `load-tests/.env.example` : Commentaires explicatifs
- `backend/.env.example` : Nouvelle variable documentée

## Utilisation

### Tests locaux (Recommandé)

```bash
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d
export BASE_URL=http://localhost:8080
./scripts/light-load.sh
```

### Tests en production (VPS)

```bash
# Sur le VPS
echo "ENABLE_RATE_LIMITING=false" >> backend/.env.vps
docker compose restart backend

# Lancer les tests
export BASE_URL=https://api.votredomaine.com
./scripts/remote-medium-load.sh

# IMPORTANT: Réactiver après !
sed -i '/ENABLE_RATE_LIMITING/d' backend/.env.vps
docker compose restart backend
```

## Vérification

```bash
# Vérifier que le rate limiting est désactivé
docker compose -f docker-compose.loadtest.yml logs backend | grep "Rate limiting"
# Devrait afficher: "Rate limiting enabled: false"
```

## Impact

### Avant
- Throughput limité à ~100 req après burst initial
- P99 latency artificielle due au throttling
- Impossible de tester les vraies limites du système

### Après
- Throughput limité uniquement par CPU/DB/réseau
- Latences réelles mesurées
- Tests de charge significatifs et exploitables

## Fichiers modifiés

1. `backend/src/main.rs` - Conditional rate limiting
2. `backend/.env.loadtest` - Configuration load testing (nouveau)
3. `backend/.env.example` - Documentation variable
4. `load-tests/docker-compose.loadtest.yml` - Configuration complète
5. `load-tests/README.md` - Documentation
6. `load-tests/START_HERE.md` - Quick start
7. `load-tests/.env.example` - Commentaires

## Sécurité

**IMPORTANT** : Le rate limiting reste activé par défaut pour la sécurité.

Il ne se désactive que si explicitement configuré avec `ENABLE_RATE_LIMITING=false`.

Ne jamais déployer en production avec le rate limiting désactivé !

## Next steps

1. Tester avec `docker compose -f docker-compose.loadtest.yml up -d`
2. Lancer les tests de charge
3. Comparer les résultats avant/après
4. Documenter les performances réelles du système

## Références

- Rate limiting configuré à 100 req/min dans `main.rs:97-111`
- Traefik rate limiting : 100 req/s dans `docker-compose.vps.yml:171-172`
- Documentation load tests : `load-tests/README.md`

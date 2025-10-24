# Monitoring Results

Ce répertoire contient les logs de monitoring générés par `monitor-server.sh` pendant les tests de charge.

## Structure des fichiers

```
monitoring-results/
├── server-monitoring_20250124_143022.log
├── server-monitoring_20250124_150145.log
└── ...
```

## Format des noms de fichiers

```
server-monitoring_YYYYMMDD_HHMMSS.log
```

## Contenu d'un fichier de monitoring

Chaque fichier contient des snapshots toutes les 5 secondes avec :

### 1. Docker Stats
```
koprogo-backend: CPU=45.2% MEM=312MB/384MB
koprogo-postgres: CPU=12.5% MEM=178MB/256MB
koprogo-traefik: CPU=2.1% MEM=32MB/50MB
```

### 2. System Resources
```
RAM: Used: 1.2GB / 2.0GB (60%)
Swap: Used: 0B / 2.0GB
Load Average: 1.23, 1.15, 0.98
```

### 3. PostgreSQL Connections
```
Total: 8 | Active: 5 | Idle: 3
```

### 4. Recent Errors
```
⚠️  Found 2 errors:
[ERROR] Database connection timeout
[ERROR] Request handler panicked
```

### 5. Network Connections
```
Established: 45 | Time-Wait: 12
```

## Utilisation

### Lancer le monitoring

```bash
# Sur le VPS, pendant que les tests tournent depuis une machine cliente
./monitor-server.sh 300  # 5 minutes
```

### Analyser les résultats

```bash
# Voir le dernier monitoring
cat $(ls -t monitoring-results/*.log | head -1)

# Chercher les pics de CPU
grep "CPU=" monitoring-results/server-monitoring_*.log | grep "backend"

# Chercher les erreurs
grep "⚠️" monitoring-results/server-monitoring_*.log

# Compter les snapshots
grep "Iteration" monitoring-results/server-monitoring_*.log | wc -l
```

## Interprétation

### CPU Usage
- **< 60%** : ✅ Bon (light load)
- **60-80%** : ✅ Acceptable (medium load)
- **80-95%** : ⚠️ Élevé (heavy load)
- **> 95%** : ❌ Saturé (risque de dégradation)

### Memory Usage
- **Stable** : ✅ Pas de fuite mémoire
- **Croissance linéaire** : ❌ Fuite mémoire probable
- **Pics temporaires** : ✅ Normal pendant les tests

### DB Connections
- **< 10** : ✅ Bon
- **10-15** : ⚠️ Approche de la limite (max=15)
- **= 15** : ❌ Pool saturé

### Erreurs
- **0 erreurs** : ✅ Parfait
- **< 5 erreurs/min** : ✅ Acceptable
- **> 10 erreurs/min** : ❌ Problème

## Nettoyage

Les fichiers de monitoring peuvent être volumineux. Nettoyer régulièrement :

```bash
# Garder seulement les 10 derniers
cd monitoring-results
ls -t *.log | tail -n +11 | xargs rm -f

# Supprimer les logs de plus de 7 jours
find . -name "*.log" -mtime +7 -delete
```

## Corrélation avec les tests clients

Pour analyser un test :

1. **Regarder les résultats du test** (depuis la machine cliente)
   ```
   load-tests/results/remote-medium-load_20250124_143022.txt
   ```

2. **Regarder le monitoring serveur** (même timestamp)
   ```
   load-tests/monitoring-results/server-monitoring_20250124_143022.log
   ```

3. **Comparer** :
   - Latence client vs CPU serveur
   - Throughput client vs connexions DB serveur
   - Erreurs client vs erreurs serveur

## Exemple d'analyse

```
Test: medium-load
Duration: 5 minutes
Client results:
  - P99: 85ms ✅
  - Throughput: 542 req/s ✅
  - Errors: 0.2% ✅

Server monitoring:
  - CPU backend: 70-75% ✅
  - RAM backend: 320-340MB ✅ (stable)
  - DB connections: 6-8 ✅
  - Errors: 0 ✅

Conclusion: ✅ Performances conformes aux attentes
```

## Troubleshooting

### Le fichier est vide

- Vérifier que docker compose fonctionne
- Vérifier les permissions du script

### "Unable to connect to PostgreSQL"

- PostgreSQL n'est pas démarré
- Vérifier : `docker compose ps postgres`

### Trop d'erreurs loggées

- Réduire le niveau de log : `RUST_LOG=warn`
- Filtrer les logs : modifier `monitor-server.sh`

## Automatisation

Pour automatiser la collecte dans un CI/CD :

```yaml
# .github/workflows/load-test.yml
- name: Start server monitoring
  run: |
    ssh user@vps "cd /opt/koprogo/load-tests && ./monitor-server.sh 300 &"

- name: Run load tests
  run: |
    export BASE_URL=https://api.example.com
    ./scripts/remote-medium-load.sh

- name: Download monitoring results
  run: |
    scp user@vps:/opt/koprogo/load-tests/monitoring-results/*.log ./artifacts/
```

## Voir aussi

- [../REMOTE_TESTING.md](../REMOTE_TESTING.md) : Guide des tests à distance
- [../monitor-server.sh](../monitor-server.sh) : Script de monitoring

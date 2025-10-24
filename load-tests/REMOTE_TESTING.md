# Tests de charge à distance

Guide pour lancer les tests de charge depuis une **machine cliente externe** vers votre VPS.

## Pourquoi tester à distance ?

⚠️ **NE PAS tester depuis le VPS lui-même !**

Raisons :
1. **Ressources partagées** : wrk consomme CPU/RAM du serveur → fausse les résultats
2. **Pas de latence réseau** : localhost ≠ conditions réelles
3. **Isolation** : Le client de test ne doit pas impacter le serveur

**✅ Bonne pratique** : Machine cliente séparée → VPS serveur

## Architecture recommandée

```
┌─────────────────┐      Internet      ┌──────────────────┐
│  Machine cliente│  ──────────────>   │  VPS (serveur)   │
│                 │                     │                  │
│  - wrk/k6/hey   │  <──────────────   │  - KoproGo API   │
│  - Monitoring   │      HTTPS         │  - PostgreSQL    │
└─────────────────┘                     └──────────────────┘
```

## Option 1 : Machine locale (recommandé pour débuter)

### Configuration de votre machine locale

```bash
# Sur votre ordinateur (Linux/Mac/WSL)
cd ~
git clone https://github.com/votre-org/koprogo.git
cd koprogo/load-tests

# Installer wrk
# Ubuntu/Debian
sudo apt-get install wrk

# macOS
brew install wrk

# Vérifier
wrk --version
```

### Lancer les tests

```bash
cd ~/koprogo/load-tests

# Par défaut, les scripts ciblent https://api.koprogo.com
# Aucune configuration nécessaire pour tester api.koprogo.com !

# Vérifier la connexion
curl https://api.koprogo.com/api/v1/health

# Lancer les tests
./scripts/warmup.sh
./scripts/remote-light-load.sh
./scripts/remote-medium-load.sh

# Pour tester un autre domaine :
export BASE_URL=https://api.votredomaine.com
./scripts/remote-light-load.sh
```

**Avantages** :
- ✅ Facile à mettre en place
- ✅ Gratuit
- ✅ Contrôle total

**Inconvénients** :
- ⚠️ Bande passante de votre connexion Internet
- ⚠️ Latence variable (dépend de votre localisation)

## Option 2 : VPS secondaire (recommandé pour production)

Utilisez un **second VPS** dans le même datacenter ou région que votre serveur.

### Pourquoi ?

- ✅ Bande passante élevée (réseau datacenter)
- ✅ Latence faible et stable
- ✅ Peut générer beaucoup plus de charge
- ✅ Pas de limite de bande passante

### Configuration

```bash
# Sur le VPS client (peut être petit : 1 vCPU, 1GB RAM suffit)
ssh user@client-vps-ip

# Installation
sudo apt-get update
sudo apt-get install -y wrk git curl htop

# Cloner le repo
cd /opt
git clone https://github.com/votre-org/koprogo.git
cd koprogo/load-tests

# Configurer l'URL cible
export BASE_URL=https://api.votredomaine.com
# Ou éditer directement les scripts
```

### Recommandations datacenter

| Votre VPS (serveur) | VPS client recommandé |
|---------------------|----------------------|
| Hetzner Falkenstein | Hetzner Falkenstein ou Nuremberg |
| OVH France | OVH France |
| AWS eu-west-1 | AWS eu-west-1 |

**Coût** : ~3-5€/mois pour un petit VPS client

## Option 3 : k6 Cloud (pour CI/CD)

k6 permet de lancer des tests depuis le cloud sans infrastructure.

### Installation k6

```bash
# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 \
  --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | \
  sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# macOS
brew install k6
```

### Script k6 basique

```javascript
// k6-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 10 },   // Warmup
    { duration: '2m', target: 50 },    // Medium load
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<100'],  // 99% requests < 100ms
    http_req_failed: ['rate<0.01'],    // Error rate < 1%
  },
};

export default function () {
  const BASE_URL = 'https://api.votredomaine.com';

  let res = http.get(`${BASE_URL}/api/v1/health`);
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });

  sleep(1);
}
```

```bash
# Lancer localement
k6 run k6-test.js

# Lancer depuis k6 cloud (nécessite un compte)
k6 cloud k6-test.js
```

## Monitoring pendant les tests distants

### Sur la machine cliente (tests)

```bash
# Lancer le test avec output détaillé
./scripts/medium-load.sh 2>&1 | tee test-output.log

# Surveiller l'avancement
tail -f test-output.log
```

### Sur le VPS serveur (application)

#### Terminal 1 : SSH vers le VPS

```bash
ssh user@vps-ip
cd /opt/koprogo

# Monitorer les ressources en temps réel
watch -n 1 'docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"'
```

#### Terminal 2 : Logs applicatifs

```bash
ssh user@vps-ip
docker compose -f docker-compose.vps.yml logs -f backend | grep -E "(ERROR|WARN|panic)"
```

#### Terminal 3 : Métriques PostgreSQL

```bash
ssh user@vps-ip
docker compose -f docker-compose.vps.yml exec postgres psql -U koprogo -d koprogo_db -c "
SELECT
    count(*) as total_connections,
    count(*) FILTER (WHERE state = 'active') as active,
    count(*) FILTER (WHERE state = 'idle') as idle
FROM pg_stat_activity;
"
```

### Script de monitoring automatisé (VPS)

Créer `monitor-during-test.sh` sur le VPS :

```bash
#!/bin/bash
# monitor-during-test.sh
# Lancer sur le VPS pendant les tests

DURATION=${1:-300}  # Durée en secondes (défaut: 5 min)
OUTPUT_FILE="monitoring_$(date +%Y%m%d_%H%M%S).log"

echo "Monitoring for ${DURATION}s..." | tee "$OUTPUT_FILE"
echo "Started at: $(date)" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"

END_TIME=$(($(date +%s) + DURATION))

while [ $(date +%s) -lt $END_TIME ]; do
    echo "=== $(date +%H:%M:%S) ===" | tee -a "$OUTPUT_FILE"

    # Docker stats
    docker stats --no-stream --format "{{.Name}}: CPU={{.CPUPerc}} MEM={{.MemUsage}}" | tee -a "$OUTPUT_FILE"

    # System resources
    echo "System: $(free -h | awk '/^Mem:/ {print "RAM="$3"/"$2}')" | tee -a "$OUTPUT_FILE"

    # DB connections
    CONN=$(docker compose -f docker-compose.vps.yml exec -T postgres \
        psql -U koprogo -d koprogo_db -t -c \
        "SELECT count(*) FROM pg_stat_activity WHERE state != 'idle';")
    echo "DB Active Connections: $CONN" | tee -a "$OUTPUT_FILE"

    echo "" | tee -a "$OUTPUT_FILE"
    sleep 5
done

echo "Monitoring complete. Results in: $OUTPUT_FILE"
```

## Comparaison des outils

| Outil | Avantages | Inconvénients | Recommandé pour |
|-------|-----------|---------------|-----------------|
| **wrk** | Simple, léger, rapide | Scripts Lua complexes | Tests HTTP simples |
| **hey** | Simple, multi-plateforme | Moins de features | Tests rapides |
| **k6** | Scripting JS, seuils, cloud | Plus lourd | Tests avancés, CI/CD |
| **Artillery** | YAML config, scénarios | Node.js requis | Tests de scénarios |

## Exemples avec différents outils

### wrk (nos scripts)

```bash
export BASE_URL=https://api.votredomaine.com
./scripts/medium-load.sh
```

### hey

```bash
# Installation
go install github.com/rakyll/hey@latest

# Test simple
hey -n 10000 -c 50 -m GET https://api.votredomaine.com/api/v1/health

# Test avec durée
hey -z 2m -c 50 -m GET https://api.votredomaine.com/api/v1/health
```

### k6

```bash
k6 run --vus 50 --duration 2m k6-test.js
```

### curl (test manuel rapide)

```bash
# Test de latence simple
for i in {1..10}; do
  curl -w "@curl-format.txt" -o /dev/null -s https://api.votredomaine.com/api/v1/health
done

# curl-format.txt :
#   time_namelookup:  %{time_namelookup}s\n
#   time_connect:  %{time_connect}s\n
#   time_total:  %{time_total}s\n
```

## Résultats attendus depuis une machine distante

### Depuis la même région (< 5ms latency réseau)

```
Light Load:
  P99: < 60ms (50ms app + 10ms réseau)
  Throughput: > 100 req/s

Medium Load:
  P99: < 120ms (100ms app + 20ms réseau)
  Throughput: > 500 req/s
```

### Depuis une autre région (50-100ms latency réseau)

```
Light Load:
  P99: < 150ms (50ms app + 100ms réseau)
  Throughput: > 100 req/s (limité par latence)

Medium Load:
  P99: < 200ms (100ms app + 100ms réseau)
  Throughput: > 300 req/s (latence impacte)
```

## Troubleshooting

### "Connection timeout"

```bash
# Vérifier le firewall du VPS
ssh user@vps-ip
sudo ufw status

# S'assurer que le port 443 est ouvert
sudo ufw allow 443/tcp
```

### "SSL certificate error"

```bash
# Vérifier le certificat Let's Encrypt
ssh user@vps-ip
docker compose -f docker-compose.vps.yml logs traefik | grep -i certificate

# Forcer un renouvellement
docker compose -f docker-compose.vps.yml restart traefik
```

### "Too many open files"

Sur la machine cliente :

```bash
# Augmenter les limites
ulimit -n 65536

# Permanent (ajouter dans /etc/security/limits.conf)
* soft nofile 65536
* hard nofile 65536
```

### Bande passante insuffisante

```bash
# Vérifier la bande passante disponible
speedtest-cli

# Si < 10 Mbps, réduire la charge :
# Éditer les scripts pour réduire -c (connexions)
```

## Best practices

1. ✅ **Toujours commencer par warmup.sh**
2. ✅ **Tester depuis la même région géographique** (sauf test de latence)
3. ✅ **Monitorer le VPS pendant les tests** (SSH + watch docker stats)
4. ✅ **Lancer les tests en heures creuses** (éviter d'impacter les utilisateurs)
5. ✅ **Garder les résultats** pour comparer après optimisations
6. ✅ **Documenter les modifications** entre deux tests

7. ❌ **NE JAMAIS tester depuis le VPS lui-même**
8. ❌ **NE PAS lancer heavy-load sans prévenir** (peut impacter les utilisateurs)
9. ❌ **NE PAS oublier de monitorer** (sinon vous ne saurez pas ce qui a cassé)

## Configuration recommandée

### Setup minimal (tests occasionnels)

- **Client** : Votre machine locale
- **Outils** : wrk + nos scripts
- **Coût** : Gratuit
- **Temps de setup** : 5 minutes

### Setup production (tests réguliers)

- **Client** : VPS secondaire (même datacenter)
- **Outils** : wrk + k6
- **Coût** : ~3-5€/mois
- **Temps de setup** : 15 minutes

### Setup CI/CD (tests automatisés)

- **Client** : GitHub Actions + k6 cloud
- **Outils** : k6
- **Coût** : Gratuit (limité) ou ~50€/mois
- **Temps de setup** : 30 minutes

## Intégration CI/CD

Exemple GitHub Actions :

```yaml
# .github/workflows/load-test.yml
name: Load Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Tous les jours à 2h du matin
  workflow_dispatch:      # Manuel

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install wrk
        run: sudo apt-get install -y wrk

      - name: Run light load test
        env:
          BASE_URL: ${{ secrets.API_URL }}
        run: |
          cd load-tests
          ./scripts/warmup.sh
          ./scripts/light-load.sh

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: load-test-results
          path: load-tests/results/
```

## Support

Pour des questions spécifiques sur les tests distants :
- Documentation wrk : https://github.com/wg/wrk
- Documentation k6 : https://k6.io/docs/
- Issues GitHub : https://github.com/votre-org/koprogo/issues

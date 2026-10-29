# 🚀 Par où commencer ?

Guide rapide pour choisir la bonne documentation.

## 🚨 IMPORTANT : Rate Limiting

**Le rate limiting fausse les tests de charge !**

L'API KoproGo limite par défaut à **100 requêtes par minute par IP**. Pour des tests de charge précis, vous devez le désactiver.

### Pour tests locaux (Docker Compose)

```bash
# Utiliser la configuration de load testing (rate limiting désactivé)
cd load-tests
docker compose -f docker-compose.loadtest.yml up -d
export BASE_URL=http://localhost:8080
./scripts/light-load.sh
```

### Pour tests en production

```bash
# Sur le VPS, éditer backend/.env ou backend/.env.vps
ENABLE_RATE_LIMITING=false

# Redémarrer l'API
docker compose restart backend

# ⚠️ N'oubliez pas de réactiver après les tests !
ENABLE_RATE_LIMITING=true
```

---

## Votre situation ?

### 1️⃣ Vous avez un **VPS en production** et voulez tester les performances

→ **Lisez : [REMOTE_TESTING.md](REMOTE_TESTING.md)**

C'est la bonne pratique pour :
- Tester depuis une machine externe (pas sur le VPS)
- Obtenir des résultats réalistes
- Ne pas impacter les ressources du serveur

**Quick start** :
```bash
# Sur votre ordinateur
cd ~/koprogo/load-tests

# Par défaut, les scripts ciblent https://api.koprogo.com
./scripts/remote-light-load.sh

# Pour tester un autre domaine :
export BASE_URL=https://api.votredomaine.com
./scripts/remote-medium-load.sh

# En parallèle, sur le VPS (autre terminal)
ssh user@vps-ip
cd /opt/koprogo/load-tests
./monitor-server.sh 300
```

---

### 2️⃣ Vous développez en **local** (docker-compose.yml)

→ **Lisez : [QUICKSTART.md](QUICKSTART.md)**

Pour tester rapidement sur localhost pendant le développement.

**Quick start** :
```bash
cd load-tests

# Pour développement local (localhost)
export BASE_URL=http://localhost:8080
./scripts/warmup.sh
./scripts/light-load.sh
```

---

### 3️⃣ Vous voulez comprendre **l'architecture** des tests

→ **Lisez : [ARCHITECTURE.md](ARCHITECTURE.md)**

Explique :
- Comment sont organisés les tests
- Différence entre tests locaux et distants
- Flux complet d'un test
- Schémas et diagrammes

---

### 4️⃣ Vous cherchez la **documentation complète**

→ **Lisez : [README.md](README.md)**

Documentation exhaustive avec :
- Tous les scénarios de test
- Métriques et objectifs
- Troubleshooting
- Optimisations possibles

---

## Résumé : Quel script lancer ?

| Situation | Scripts à utiliser |
|-----------|-------------------|
| **Production VPS** | `remote-light-load.sh`, `remote-medium-load.sh` |
| **Développement local** | `warmup.sh`, `light-load.sh`, `medium-load.sh` |
| **Point de rupture** | `heavy-load.sh` |
| **Test de résilience** | `spike-test.sh` |
| **Fuites mémoire** | `soak-test.sh` (30 min) |
| **Monitoring serveur** | `monitor-server.sh 300` (sur le VPS) |

---

## Arbre de décision

```
Avez-vous un VPS en production ?
│
├─ OUI → Tests à distance (REMOTE_TESTING.md)
│        │
│        ├─ Machine cliente : Votre ordinateur ou VPS client
│        │  └─ Scripts : remote-*.sh
│        │
│        └─ VPS serveur : Monitoring
│           └─ Script : monitor-server.sh
│
└─ NON → Tests locaux (QUICKSTART.md)
         │
         └─ Développement : docker-compose.yml
            └─ Scripts : warmup.sh, light-load.sh, etc.
```

---

## Installation rapide

### Sur machine cliente (tests distants)

```bash
# Docker suffit : les scripts lancent k6 2.3.0 en conteneur
# (ou installer k6 : https://grafana.com/docs/k6/latest/set-up/install-k6/)

# Cloner le repo
git clone https://github.com/votre-org/koprogo.git
cd koprogo/load-tests
```

### Sur le VPS (monitoring)

```bash
# Se connecter au VPS
ssh user@vps-ip
cd /opt/koprogo/load-tests

# Le script monitor-server.sh est déjà là
chmod +x monitor-server.sh
```

---

## Besoin d'aide ?

- **Problème de connexion** : Vérifier DNS, firewall, certificats SSL
- **Latence élevée** : Vérifier CPU/RAM du VPS, connexions DB
- **Trop d'erreurs** : Réduire la charge, vérifier les logs
- **Documentation complète** : [README.md](README.md)
- **GitHub Issues** : https://github.com/votre-org/koprogo/issues

---

## Rappels importants

⚠️ **NE PAS tester depuis le VPS en production** → Utiliser une machine externe
✅ **Toujours commencer par warmup.sh** → Prépare les caches
✅ **Monitorer le VPS pendant les tests** → `monitor-server.sh`
✅ **Tester en heures creuses** → Éviter d'impacter les utilisateurs
✅ **Garder les résultats** → Comparer les performances après optimisations

---

## Next steps

1. **Lire la doc appropriée** (voir ci-dessus)
2. **Installer les outils** (Docker, ou k6)
3. **Lancer un premier test** (light-load)
4. **Analyser les résultats**
5. **Optimiser si nécessaire**

Bonne chance ! 🚀

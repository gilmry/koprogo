# Résolution des erreurs HTTP 401 dans les tests de charge

## Problème identifié

Les tests de charge retournaient **99% d'erreurs HTTP 401 Unauthorized** :
```
Total requests: 52042
Successful: 494
Errors: 51548
Non-2xx or 3xx responses: 51509
```

## Cause racine

L'architecture hexagonale de KoproGo implémente une **authentification JWT obligatoire** pour tous les endpoints CRUD via le middleware `AuthenticatedUser` (voir `backend/src/infrastructure/web/middleware.rs`).

Les scripts Lua originaux (`mixed.lua`, `buildings.lua`) ne fournissaient **aucun token JWT** dans leurs requêtes, ce qui causait les 401.

### Endpoints concernés

**Protégés** (nécessitent JWT) :
- `/api/v1/buildings` (GET, POST, PUT, DELETE)
- `/api/v1/units` (GET, POST, PUT)
- `/api/v1/owners` (GET, POST, PUT)
- `/api/v1/expenses` (GET, POST, PUT)
- `/api/v1/meetings` (GET, POST, PUT, DELETE)
- `/api/v1/documents` (GET, POST, DELETE)

**Publics** (pas de JWT requis) :
- `/api/v1/health` ✅
- `/api/v1/auth/login` ✅
- `/api/v1/auth/register` ✅
- `/api/v1/auth/refresh` ✅

## Solution implémentée

### 1. Nouveau script Lua : `authenticated-mixed.lua`

Ce script :
1. **Se connecte d'abord** avec les credentials de démo
2. **Extrait le token JWT** de la réponse
3. **Réutilise ce token** pour toutes les requêtes suivantes

```lua
-- Première requête : login
local login_body = '{"email":"syndic@grandplace.be","password":"syndic123"}'
return wrk.format("POST", "/api/v1/auth/login", nil, login_body)

-- Requêtes suivantes : avec token
wrk.headers["Authorization"] = "Bearer " .. jwt_token
```

### 2. Comptes de démonstration utilisés

**Credentials disponibles** (voir `backend/src/infrastructure/database/seed.rs:493-497`) :

| Organisation | Email | Mot de passe | Rôle |
|--------------|-------|--------------|------|
| SuperAdmin | `admin@koprogo.com` | `admin123` | superadmin |
| Grand Place | `syndic@grandplace.be` | `syndic123` | syndic |
| Copro Bruxelles | `syndic@copro-bruxelles.be` | `syndic123` | syndic |
| Syndic Liège | `syndic@syndic-liege.be` | `syndic123` | syndic |

Le script utilise par défaut : **`syndic@grandplace.be` / `syndic123`**

### 3. Scripts mis à jour

Les scripts suivants utilisent maintenant `authenticated-mixed.lua` :
- ✅ `load-tests/scripts/light-load.sh`
- ✅ `load-tests/scripts/medium-load.sh`
- ✅ `load-tests/scripts/heavy-load.sh`

## Comment vérifier que ça fonctionne

### Test rapide local

```bash
# 1. Démarrer le backend avec données de démo
cd backend
cargo run

# 2. Dans un autre terminal, lancer un test light
cd load-tests
./scripts/light-load.sh

# 3. Vérifier dans la sortie :
# ✅ JWT token acquired successfully
# Total requests: ~12000
# Successful: ~11900+
# Errors: < 100
```

### Test sur VPS

```bash
export BASE_URL=https://api.koprogo.com
./scripts/light-load.sh
```

## Résultats attendus APRÈS le fix

```
Total requests: 12000
Successful: 11950+  (> 99%)
Errors: < 50        (< 1%)
✅ Authentication: SUCCESS
```

## Alternatives considérées

### Option 1 : Tests sans authentification (non retenue)
- Tester uniquement `/api/v1/health`
- ❌ Ne teste pas la logique métier réelle
- ❌ Ne reflète pas l'usage en production

### Option 2 : Désactiver l'auth temporairement (non retenue)
- Modifier le middleware pour bypasser JWT en dev
- ❌ Divergence entre dev et production
- ❌ Risque de déployer sans auth par erreur

### Option 3 : Authentification par script Lua ✅ (implémentée)
- Authentification réaliste
- ✅ Teste le flow complet incluant JWT
- ✅ Reflète l'usage production

## Code modifié

### Fichiers créés
- `load-tests/lua/authenticated-mixed.lua` - Script Lua avec JWT

### Fichiers modifiés
- `load-tests/scripts/light-load.sh` - Utilise authenticated-mixed.lua
- `load-tests/scripts/medium-load.sh` - Utilise authenticated-mixed.lua
- `load-tests/scripts/heavy-load.sh` - Utilise authenticated-mixed.lua

### Fichiers inchangés (legacy)
- `load-tests/lua/mixed.lua` - Conservé pour référence
- `load-tests/lua/buildings.lua` - Conservé pour référence
- `load-tests/lua/auth.lua` - Conservé pour tests d'auth spécifiques

## Dépendances

Le script `authenticated-mixed.lua` nécessite :
- **LuaJIT avec module cjson** (pour parser le JSON de réponse)
- Installé automatiquement avec `wrk` sur la plupart des systèmes

Vérification :
```bash
wrk --version
# wrk 4.2.0 [epoll] Copyright (C) 2012 Will Glozer
```

## Références

- Middleware auth : `backend/src/infrastructure/web/middleware.rs:32-72`
- Seed data : `backend/src/infrastructure/database/seed.rs:493-497`
- Routes protégées : `backend/src/infrastructure/web/routes.rs`
- Documentation JWT : `backend/docs/JWT_SECURITY.md`

## Monitoring recommandé

Pour vérifier que l'authentification fonctionne en production :

```bash
# Logs backend pendant le test
docker compose logs -f backend | grep -i "401\|unauthorized\|token"

# Statistiques de réponses HTTP
docker compose logs backend | grep -oP 'status=\d+' | sort | uniq -c
```

---

## Utilisation de run-all-tests.sh

### Sur api2.koprogo.com

```bash
cd load-tests
export BASE_URL=https://api2.koprogo.com
./run-all-tests.sh
```

### Sur api.koprogo.com (défaut)

```bash
cd load-tests
./run-all-tests.sh  # Utilise api.koprogo.com par défaut
```

### En local

```bash
cd load-tests
export BASE_URL=http://localhost:8080
./run-all-tests.sh
```

### Erreurs 404 courantes

Si vous obtenez des erreurs 404 :

1. **Vérifiez que BASE_URL est exporté** :
   ```bash
   echo $BASE_URL  # Doit afficher https://api2.koprogo.com
   ```

2. **Vérifiez que les données de démo sont présentes** :
   ```bash
   curl https://api2.koprogo.com/api/v1/health
   # Doit retourner 200 OK

   curl -X POST https://api2.koprogo.com/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"syndic@grandplace.be","password":"syndic123"}'
   # Doit retourner un token JWT
   ```

3. **Si login échoue = données de démo manquantes** :
   - Connectez-vous en tant que superadmin
   - Appelez `POST /api/v1/seed/demo`

---

**Date de résolution** : 2025-10-25
**Auteur** : Claude Code
**Impact** : Critique - Bloquait tous les tests de charge

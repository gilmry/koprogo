# 🛠️ Guide des Commandes Make

Ce guide liste toutes les commandes `make` disponibles pour KoproGo.

## 📋 Voir toutes les commandes

```bash
make help
```

Affiche la liste de toutes les commandes avec leur description.

---

## 🚀 Setup et Installation

### Setup complet (première utilisation)

```bash
make setup
```

Cette commande fait **tout** automatiquement :
- ✅ Installe les dépendances npm
- ✅ Installe Playwright + navigateurs
- ✅ Démarre PostgreSQL via Docker
- ✅ Exécute les migrations de base de données

**C'est tout ce dont vous avez besoin pour démarrer!**

### Installation manuelle

```bash
make install         # Installe seulement les dépendances npm
make install-all     # Installe npm + Playwright
```

---

## 💻 Développement

### Démarrer l'environnement de développement

```bash
make dev             # Backend only (Rust avec hot-reload)
make dev-all         # Tous les services (backend + postgres + frontend)
make dev-frontend    # Frontend only (Astro + Svelte)
```

**Workflow recommandé :**

**Terminal 1:**
```bash
make dev  # Démarre backend + PostgreSQL
```

**Terminal 2:**
```bash
make dev-frontend  # Démarre le frontend
```

Puis ouvrir :
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080

---

## 🧪 Tests

### Tests Backend (Rust)

```bash
make test              # Tous les tests backend + E2E complets
make test-unit         # Tests unitaires seulement
make test-integration  # Tests d'intégration
make test-bdd          # Tests BDD (Cucumber)
make test-e2e-backend  # Tests E2E backend (Rust/Actix)
```

### Tests E2E Complets (Frontend + Backend) 🎥

Les tests E2E avec Playwright testent **toute la stack** et génèrent automatiquement des **vidéos de documentation**!

#### Installation (une seule fois)

```bash
make test-e2e-install
```

Installe Playwright et Chromium avec toutes les dépendances.

#### Lancer les tests

```bash
make test-e2e-full     # Lance tous les tests E2E + génère les vidéos!
make test-e2e-ui       # Mode UI interactif (recommandé)
make test-e2e-headed   # Voir le navigateur en action
make test-e2e-debug    # Mode debug pas à pas
make test-e2e-report   # Ouvrir le rapport HTML avec vidéos
```

#### Workflow recommandé

**Développement:**
```bash
make test-e2e-ui       # Interface graphique interactive
```

**CI/CD ou validation finale:**
```bash
make test-e2e-full     # Génère toutes les vidéos
make test-e2e-report   # Voir les résultats
```

**Debugging:**
```bash
make test-e2e-debug    # Mode pas à pas
```

### Tests de Performance

```bash
make bench             # Benchmarks Rust
```

---

## 📊 Couverture et Qualité

```bash
make coverage          # Génère un rapport de couverture
make lint              # Vérifie le code (fmt + clippy + build)
make format            # Formate le code (Rust + JS/TS)
make audit             # Audit de sécurité (Cargo + npm)
```

---

## 🏗️ Build

```bash
make build             # Build release (backend + frontend)
make clean             # Nettoie les artefacts de build
```

---

## 🐳 Docker

```bash
make docker-up         # Démarre tous les services Docker
make docker-down       # Arrête tous les services Docker
make docker-build      # Build les images Docker
make docker-logs       # Affiche les logs Docker
```

---

## 🗄️ Base de Données

```bash
make migrate           # Exécute les migrations SQLx
make seed              # Remplit la base avec des données de test
```

---

## 📚 Documentation

```bash
make docs              # Génère et ouvre la documentation Rust
```

---

## 🎯 Workflows Courants

### 1. Nouvelle installation du projet

```bash
git clone <repo>
cd koprogo
make setup             # Setup complet automatique
make dev               # Démarrer le développement
```

### 2. Développement quotidien

**Terminal 1:**
```bash
make dev               # Backend + PostgreSQL
```

**Terminal 2:**
```bash
make dev-frontend      # Frontend avec hot-reload
```

### 3. Avant de commit

```bash
make format            # Formater le code
make lint              # Vérifier la qualité
make test              # Lancer tous les tests
```

### 4. Tester une nouvelle fonctionnalité

```bash
# Développer la feature...

# Tester
make test-e2e-ui       # Tests E2E en mode interactif

# Générer les vidéos de documentation
make test-e2e-full     # Génère les vidéos
make test-e2e-report   # Voir les vidéos
```

### 5. CI/CD local

```bash
make clean
make build
make test
make test-e2e-full
make audit
```

### 6. Debugging des tests E2E

```bash
# Problème dans les tests?
make test-e2e-headed   # Voir le navigateur

# Toujours pas clair?
make test-e2e-debug    # Mode debug pas à pas

# Voir ce qui s'est passé
make test-e2e-report   # Voir les vidéos + screenshots
```

---

## 📹 Tests E2E - Exemples de Commandes

### Développement d'un nouveau test

```bash
# 1. Créer le fichier de test
cd frontend/tests/e2e
touch ma-feature.spec.ts

# 2. Développer le test en mode UI
make test-e2e-ui

# 3. Valider et générer la vidéo
make test-e2e-full

# 4. Voir le résultat
make test-e2e-report
```

### Démonstration au client

```bash
# Option 1: Lancer les tests en live
make test-e2e-headed

# Option 2: Montrer les vidéos déjà générées
make test-e2e-report
```

### Debugging d'un test qui échoue

```bash
# 1. Voir la vidéo de l'échec
make test-e2e-report

# 2. Debug pas à pas
make test-e2e-debug

# 3. Relancer en voyant le navigateur
make test-e2e-headed
```

---

## 🎬 Vidéos de Documentation

Les vidéos générées par `make test-e2e-full` se trouvent dans :

```
frontend/test-results/
├── auth-Authentication-Flow-should-login-successfully-chromium/
│   └── video.webm
├── pwa-offline-PWA-Capabilities-should-work-offline-chromium/
│   └── video.webm
└── dashboards-Syndic-Dashboard-chromium/
    └── video.webm
```

**Pour les voir :**
```bash
make test-e2e-report
```

---

## 🔧 Variables d'Environnement

Les commandes make utilisent les variables d'environnement définies dans `.env` :

```bash
DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_db
JWT_SECRET=your-secret-key-change-this-in-production
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

---

## 💡 Tips

### Performances des tests E2E

```bash
# Tests rapides (headless)
make test-e2e-full

# Tests lents mais visibles (headed)
make test-e2e-headed
```

### Nettoyage complet

```bash
make clean
make docker-down
docker volume prune -f
```

### Réinitialisation de la base de données

```bash
make docker-down
docker volume rm koprogo_postgres_data
make docker-up
make migrate
make seed
```

---

## 📖 Documentation Supplémentaire

- [E2E Testing Guide](./E2E_TESTING_GUIDE.md) - Guide complet des tests E2E
- [Tests E2E README](./frontend/tests/e2e/README.md) - Documentation détaillée des tests

---

## 🆘 Aide

Si une commande échoue :

1. **Vérifier les services** : `make docker-up`
2. **Vérifier les migrations** : `make migrate`
3. **Nettoyer et rebuild** : `make clean && make build`
4. **Setup complet** : `make setup`

Pour voir toutes les commandes disponibles :
```bash
make help
```

# Tests E2E avec Documentation Vidéo Vivante

Ce dossier contient les tests End-to-End (E2E) de KoproGo utilisant Playwright. **Chaque test génère automatiquement une vidéo** qui sert de **documentation vivante** du comportement de l'application!

## 🎥 Documentation Vivante

Les vidéos générées par les tests servent de documentation pour :

- ✅ Démontrer les parcours utilisateurs fonctionnels
- ✅ Onboarder de nouveaux développeurs
- ✅ Présenter les fonctionnalités aux clients
- ✅ Valider les specs avec les stakeholders
- ✅ Déboguer les problèmes en production

## 📁 Structure des Tests

```
tests/e2e/
├── README.md                       # Ce fichier
├── AdminDashBoard.improved.spec.ts # Suite complète admin (orgs/users/buildings + parcours global)
└── config.ts                       # Helpers de configuration Playwright
```

## 🚀 Installation

### 1. Installer Playwright et les navigateurs

```bash
npm run test:install
```

Cette commande installe Chromium avec toutes les dépendances système nécessaires.

### 2. Démarrer le backend

Les tests E2E nécessitent que le backend soit en cours d'exécution :

```bash
docker compose up -d postgres minio backend traefik frontend
```

Par défaut Traefik expose le frontend sur `http://localhost` et proxy les appels API vers `http://localhost/api/v1`.

## 🧪 Exécution des Tests

### Mode Headless (CI/CD)

```bash
npm run test:e2e
```

- Lance tous les tests en arrière-plan
- Génère automatiquement les vidéos dans `test-results/`
- Crée un rapport HTML
- Utilise `PLAYWRIGHT_BASE_URL` si défini (défaut `http://localhost`)

> Astuce: pour cibler l'environnement Traefik local, exécuter  
> `PLAYWRIGHT_BASE_URL=http://localhost npm run test:e2e`

### Mode UI (Recommandé pour le développement)

```bash
npm run test:e2e:ui
```

- Interface graphique interactive
- Visualisation en temps réel
- Rejeu des tests facilement
- Inspection du DOM

### Mode Headed (Voir le navigateur)

```bash
npm run test:e2e:headed
```

- Voir le navigateur s'exécuter en temps réel
- Utile pour déboguer

### Mode Debug (Pas à pas)

```bash
npm run test:e2e:debug
```

- Debugger interactif Playwright
- Points d'arrêt automatiques
- Inspection du state

## 📹 Vidéos de Documentation

### Emplacement des Vidéos

Après chaque exécution de test, les vidéos sont générées dans :

Chaque répertoire de `test-results/` correspond à un scénario Playwright, exemple :

```
test-results/
├── AdminDashBoard.improved-Ad-11345-create-edit-delete-organization/
│   └── video.webm
└── AdminDashBoard.improved-Ad-319xx-idempotent-full-journey/
    └── video.webm
```

### Configuration Vidéo

Dans `playwright.config.ts` :

```typescript
use: {
  video: {
    mode: 'on',  // Toujours enregistrer (même si le test passe!)
    size: { width: 1280, height: 720 }
  }
}
```

**Mode 'on'** = Enregistrement systématique = Documentation complète!

### Visualiser les Vidéos

#### Option 1: Rapport HTML (Recommandé)

```bash
npm run test:e2e:report
```

Ouvre un rapport HTML interactif avec :

- ✅ Vidéos intégrées
- ✅ Screenshots
- ✅ Traces Playwright
- ✅ Logs de console

#### Option 2: Lecteur Vidéo

Ouvrir directement les fichiers `.webm` dans :

- Chrome/Chromium
- Firefox
- VLC
- Tout lecteur supportant WebM

## 📊 Rapport de Tests

### Générer et Voir le Rapport

```bash
npm run test:e2e          # Lance les tests
npm run test:e2e:report   # Ouvre le rapport
```

Le rapport contient :

- 📹 **Vidéos de chaque test**
- 📸 Screenshots à chaque étape
- 📝 Traces d'exécution détaillées
- ⏱️ Temps d'exécution
- ✅/❌ Status des tests

## 🎬 Scénarios Couverts

### 1. Admin Dashboard (`AdminDashBoard.spec.ts`)

**Tous les parcours CRUD (Create-Read-Update-Delete) du dashboard administrateur :**

#### Organizations Management

- ✅ Créer une organisation complète (nom, slug, email, téléphone)
- ✅ Modifier une organisation existante
- ✅ Supprimer une organisation
- ✅ Rechercher des organisations par nom/email/slug
- ✅ Activer/Désactiver une organisation

#### Users Management

- ✅ Créer un utilisateur avec rôle
- ✅ Modifier les informations d'un utilisateur
- ✅ Supprimer un utilisateur
- ✅ Filtrer par rôle (SuperAdmin, Syndic, Comptable, Propriétaire)
- ✅ Rechercher par nom ou email

#### Buildings Management

- ✅ Créer un immeuble (nom, adresse, ville, code postal, lots, année)
- ✅ Modifier un immeuble existant
- ✅ Supprimer un immeuble
- ✅ Rechercher des immeubles

#### Complete Admin Journey

- ✅ Parcours complet : Créer org → créer user → créer building
- ✅ Cleanup automatique dans l'ordre inverse
- ✅ Vérification de bout en bout

**Vidéos générées** : Démonstration complète de toutes les opérations CRUD disponibles pour un administrateur.

**Comment lancer** :

```bash
npm run test:e2e -- AdminDashBoard.spec.ts
```

### 2. Suite Admin End-to-End (`AdminDashBoard.improved.spec.ts`)

Cette unique suite couvre l'intégralité des workflows administrateur à l'aide des `data-testid` :

- ✅ Organisations : création, édition, suppression, recherche, changement de statut.
- ✅ Utilisateurs : création (avec rattachement organisation/role), édition, suppression, filtres et recherche.
- ✅ Immeubles : création (assignation org), édition, suppression, recherche.
- ✅ Parcours complet : org ➜ user ➜ building ➜ nettoyage automatique.

**Vidéos générées** : un clip par scénario ci-dessus + le parcours complet.

## 🔧 Configuration Avancée

### Modifier la Configuration Vidéo

Dans `playwright.config.ts`, vous pouvez ajuster :

```typescript
video: {
  mode: 'on',           // 'on' | 'off' | 'retain-on-failure' | 'on-first-retry'
  size: { width: 1920, height: 1080 }  // Résolution HD
}
```

Options :

- `'on'` : **Recommandé pour la doc** - Enregistre toujours
- `'retain-on-failure'` : Seulement en cas d'échec
- `'on-first-retry'` : Lors du premier retry
- `'off'` : Pas de vidéo

### Screenshots Supplémentaires

```typescript
use: {
  screenshot: 'on',  // Screenshots à chaque étape
}
```

### Traces Playwright

```typescript
use: {
  trace: 'on',  // Traces complètes pour debug
}
```

## 📝 Écrire de Nouveaux Tests

### Template de Base

```typescript
import { test, expect } from "@playwright/test";

test.describe("Ma Fonctionnalité", () => {
  test("devrait faire quelque chose", async ({ page }) => {
    // Arrange
    await page.goto("/ma-page");

    // Act
    await page.click("button");

    // Assert
    await expect(page.locator("text=Succès")).toBeVisible();
  });
});
```

### Best Practices

1. **Descriptions claires** : Le nom du test apparaît dans la vidéo
2. **Attentes explicites** : `await expect(...).toBeVisible()`
3. **Waits appropriés** : `await page.waitForURL(...)`
4. **Cleanup** : Utiliser `beforeEach` pour reset l'état

### Test avec API Backend

```typescript
test("devrait créer un utilisateur via l'API", async ({ page }) => {
  const response = await page.request.post(
    "http://127.0.0.1:8080/api/v1/auth/register",
    {
      data: {
        email: `user-${Date.now()}@test.com`,
        password: "test123",
        first_name: "Test",
        last_name: "User",
        role: "syndic",
      },
    },
  );

  const { user, token } = await response.json();
  expect(response.ok()).toBeTruthy();

  // Maintenant utiliser ces credentials pour login
  await page.goto("/login");
  await page.fill('input[type="email"]', user.email);
  await page.fill('input[type="password"]', "test123");
  await page.click('button[type="submit"]');
});
```

## 🎯 CI/CD

### GitHub Actions

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          cd frontend
          npm ci
          npm run test:install

      - name: Start backend
        run: |
          cd backend
          cargo run &
          sleep 10

      - name: Run E2E tests
        run: |
          cd frontend
          npm run test:e2e

      - name: Upload videos as artifacts
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-videos
          path: frontend/test-results/**/*.webm

      - name: Upload HTML report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

Les vidéos sont automatiquement sauvegardées comme **artifacts GitHub** !

## 📚 Utilisation comme Documentation

### 1. Pour l'équipe de développement

- Partager les vidéos dans Slack/Teams
- Inclure dans les pull requests
- Onboarding de nouveaux développeurs

### 2. Pour les clients/stakeholders

- Démonstration des fonctionnalités
- Validation des specs
- Acceptance testing

### 3. Pour le support

- Reproduire les bugs clients
- Créer des guides visuels
- Formation des utilisateurs

## 🐛 Débogage

### Test qui échoue ?

1. **Voir la vidéo** : `npm run test:e2e:report`
2. **Mode debug** : `npm run test:e2e:debug`
3. **Logs** : Vérifier les console logs dans le rapport
4. **Traces** : Utiliser Playwright Trace Viewer

### Problèmes Courants

#### Backend pas démarré

```
Error: connect ECONNREFUSED 127.0.0.1:8080
```

**Solution** : `cd ../backend && cargo run`

#### Timeout

```
Error: page.waitForURL: Timeout 30000ms exceeded
```

**Solution** : Augmenter le `navigationTimeout` dans la config

#### Service Worker

```
Service worker not registered
```

**Solution** : S'assurer que le PWA plugin est configuré

## 📖 Ressources

- [Documentation Playwright](https://playwright.dev)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Test Generator](https://playwright.dev/docs/codegen) : `npx playwright codegen http://localhost:3000`

## 🎉 C'est Parti !

```bash
# 1. Installer
npm run test:install

# 2. Démarrer le backend
cd ../backend && cargo run

# 3. Lancer les tests
cd ../frontend
npm run test:e2e

# 4. Voir les vidéos de documentation !
npm run test:e2e:report
```

**Les vidéos sont votre documentation vivante - elles montrent exactement comment l'application fonctionne !** 🎥✨

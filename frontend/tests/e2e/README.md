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
├── README.md              # Ce fichier
├── auth.spec.ts          # Tests d'authentification (login, logout, rôles)
├── dashboards.spec.ts    # Tests des dashboards par rôle
└── pwa-offline.spec.ts   # Tests PWA et mode offline
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
cd ../backend
cargo run
```

Le backend doit être accessible sur `http://127.0.0.1:8080`

## 🧪 Exécution des Tests

### Mode Headless (CI/CD)
```bash
npm run test:e2e
```
- Lance tous les tests en arrière-plan
- Génère automatiquement les vidéos dans `test-results/`
- Crée un rapport HTML

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

```
test-results/
├── auth-Authentication-Flow-should-login-successfully-chromium/
│   └── video.webm  <-- Vidéo du test de login
├── pwa-offline-PWA-Capabilities-should-work-offline-chromium/
│   └── video.webm  <-- Vidéo du mode offline
└── dashboards-Syndic-Dashboard-chromium/
    └── video.webm  <-- Vidéo du dashboard syndic
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

### 1. Authentification (`auth.spec.ts`)
- ✅ Page de login accessible
- ✅ Login avec credentials backend réels
- ✅ Redirection vers dashboard selon le rôle
- ✅ Gestion des erreurs (mauvais password)
- ✅ Persistance de session (localStorage + IndexedDB)
- ✅ Logout complet
- ✅ Création de comptes par rôle (Syndic, Accountant, Owner, SuperAdmin)

**Vidéo générée** : Parcours complet d'un utilisateur qui se connecte et accède à son dashboard.

### 2. Dashboards (`dashboards.spec.ts`)
- ✅ Dashboard Syndic (gestion immeubles, tâches)
- ✅ Dashboard Comptable (finances, transactions)
- ✅ Dashboard Copropriétaire (infos personnelles)
- ✅ Dashboard SuperAdmin (vue plateforme)
- ✅ Navigation entre les sections
- ✅ Permissions par rôle

**Vidéos générées** : Un parcours vidéo pour chaque type d'utilisateur.

### 3. PWA et Offline (`pwa-offline.spec.ts`)
- ✅ Manifest.json présent et valide
- ✅ Service Worker enregistré
- ✅ Indicateur online/offline fonctionnel
- ✅ IndexedDB utilisé pour le cache
- ✅ Mode offline après chargement initial
- ✅ Queue de synchronisation
- ✅ Synchronisation manuelle

**Vidéos générées** : Démonstration du mode offline et de la synchronisation.

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
import { test, expect } from '@playwright/test';

test.describe('Ma Fonctionnalité', () => {
  test('devrait faire quelque chose', async ({ page }) => {
    // Arrange
    await page.goto('/ma-page');

    // Act
    await page.click('button');

    // Assert
    await expect(page.locator('text=Succès')).toBeVisible();
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
test('devrait créer un utilisateur via l\'API', async ({ page }) => {
  const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
    data: {
      email: `user-${Date.now()}@test.com`,
      password: 'test123',
      first_name: 'Test',
      last_name: 'User',
      role: 'syndic'
    }
  });

  const { user, token } = await response.json();
  expect(response.ok()).toBeTruthy();

  // Maintenant utiliser ces credentials pour login
  await page.goto('/login');
  await page.fill('input[type="email"]', user.email);
  await page.fill('input[type="password"]', 'test123');
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

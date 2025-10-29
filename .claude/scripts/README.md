# 🎥 Scripts d'Intégration Vidéos Playwright

Ce dossier contient les scripts pour intégrer automatiquement les vidéos Playwright dans la documentation Sphinx.

## 🚀 Workflow Simplifié (Nouveau)

**Le système a été grandement simplifié !** Vous pouvez maintenant :

1. **Enregistrer vos tests** avec Playwright Codegen (mode enregistrement interactif)
2. **Copier les vidéos** automatiquement avec `make docs-sync-videos`
3. **La page RST se génère toute seule** - liste automatique de toutes les vidéos

### Méthode recommandée : Enregistrement avec Playwright Codegen

La façon la plus simple d'enregistrer vos tests :

```bash
cd frontend

# Démarrer l'enregistrement interactif
npm run codegen
# OU: npx playwright codegen http://localhost:3000

# Version mobile
npm run codegen:mobile

# Playwright ouvre un navigateur et enregistre vos actions
# → Cliquez, naviguez, remplissez les formulaires
# → Le code du test est généré automatiquement
# → Copiez-le dans tests/e2e/mon-test.spec.ts

# Puis lancez le test pour générer la vidéo
npm run test:e2e -- mon-test.spec.ts
```

### Workflow complet

```bash
# 1. Enregistrer votre test interactivement
cd frontend
npx playwright codegen http://localhost:3000

# 2. Copier le code généré dans un fichier .spec.ts
# (Playwright affiche le code dans une fenêtre séparée)

# 3. Lancer le test pour générer la vidéo
npm run test:e2e

# 4. Synchroniser les vidéos
cd ..
make docs-sync-videos

# 5. Générer la documentation
make docs-sphinx

# Ou étapes 4-5 en une commande :
make docs-with-videos
```

## 📁 Fichiers

### `copy-videos.sh` ⭐ (Nouveau - Simplifié)

Script principal qui copie les vidéos et génère automatiquement la page RST.

**Usage :**
```bash
bash .claude/scripts/copy-videos.sh
# OU
make docs-sync-videos
```

### `generate-video-rst.py` ⭐ (Nouveau)

Génère automatiquement `docs/e2e-videos.rst` en listant toutes les vidéos `.webm` présentes dans `docs/_static/videos/`.

- ✅ Scanne automatiquement le répertoire
- ✅ Convertit les noms de fichiers en titres lisibles
- ✅ Génère le HTML avec player vidéo
- ✅ Aucune configuration manuelle nécessaire

**Appelé automatiquement par `copy-videos.sh`** - pas besoin de l'exécuter manuellement.

### `slow-down-tests.sh` ⭐ (Nouveau - Vidéos plus lisibles)

Ajoute automatiquement des pauses (délai configurable) entre chaque action dans les tests pour créer des vidéos plus faciles à suivre.

**Usage :**
```bash
# Ralentir avec 1 seconde entre chaque action
bash .claude/scripts/slow-down-tests.sh 1000

# OU via make (recommandé)
make test-e2e-slow
```

**Ce que ça fait :**
- Ajoute `await page.waitForTimeout(1000)` après chaque `click()`, `fill()`, `goto()`, etc.
- Créé des vidéos beaucoup plus faciles à suivre pour la documentation

### `restore-test-speed.sh` ⭐ (Nouveau)

Retire toutes les pauses ajoutées par `slow-down-tests.sh` pour revenir à la vitesse normale.

**Usage :**
```bash
bash .claude/scripts/restore-test-speed.sh
# OU
make test-e2e-restore-speed
```

### `sync-playwright-videos.sh` (Ancien - Complexe)

Script principal qui synchronise les vidéos E2E de Playwright vers la documentation Sphinx.

**Fonctionnalités :**
- ✅ Copie les vidéos de `frontend/test-results/` vers `docs/_static/videos/`
- ✅ Renomme avec des noms explicites (auth-login-success.webm, etc.)
- ✅ Conversion optionnelle en MP4 (si ffmpeg disponible)
- ✅ Génère un index.html standalone pour preview

**Usage :**

```bash
# Depuis la racine du projet
bash .claude/scripts/sync-playwright-videos.sh

# Via npm (depuis frontend/)
npm run docs:videos

# Via Make (depuis la racine)
make docs-with-videos
```

**Prérequis :**
- Les tests E2E doivent avoir été lancés au moins une fois
- ffmpeg (optionnel) pour conversion MP4

## 🎯 Workflow Complet

### 1. Développement Local

```bash
# Étape 1 : Lancer les tests E2E
cd frontend
npm run test:e2e

# Étape 2 : Synchroniser les vidéos
npm run docs:videos

# Étape 3 : Générer la doc Sphinx
cd ../docs
make html

# Étape 4 : Prévisualiser
cd _build/html
python3 -m http.server 8000
# Ouvrir http://localhost:8000/e2e-videos.html
```

### 2. Via Make (Recommandé)

```bash
# Tout en une commande
make docs-with-videos

# Servir avec preview
make docs-serve-videos
```

### 3. CI/CD

Le workflow `.github/workflows/docs-videos.yml` automatise tout :
1. ✅ Lance les tests E2E avec backend PostgreSQL
2. 🎥 Génère les vidéos
3. 📋 Synchronise vers docs
4. 📚 Build Sphinx
5. 🚀 Déploie sur GitHub Pages (si main)

## 📊 Mapping des Vidéos

Le script utilise un mapping intelligent pour renommer les vidéos :

| Test Playwright | Nom vidéo | Catégorie |
|----------------|-----------|-----------|
| `should-login-successfully` | `auth-login-success.webm` | Auth |
| `should-display-syndic-dashboard` | `dashboard-syndic-sections.webm` | Dashboard |
| `should-work-offline-after-initial` | `pwa-offline-mode.webm` | PWA |

Voir le script pour la liste complète (30 tests).

## 🔧 Configuration

### Playwright (`frontend/playwright.config.ts`)

```typescript
video: {
  mode: 'on',  // Enregistre toujours
  size: { width: 1280, height: 720 }
}
```

### Sphinx (`docs/e2e-videos.rst`)

- Grille responsive (auto-fit, minmax 420px)
- Support WebM + MP4 fallback
- Badges colorés par catégorie
- Statistiques automatiques

## 🐛 Dépannage

**❌ "Aucune vidéo trouvée"**
```bash
cd frontend && npm run test:e2e
```

**❌ "ffmpeg non installé"**
```bash
# Ubuntu/Debian
sudo apt-get install ffmpeg

# macOS
brew install ffmpeg
```

**❌ "Permission denied"**
```bash
chmod +x .claude/scripts/sync-playwright-videos.sh
```

## 📚 Documentation

- **Guide E2E** : `docs/E2E_TESTING_GUIDE.rst`
- **Page vidéos** : `docs/e2e-videos.rst`
- **Makefile** : `docs/MAKEFILE_GUIDE.rst`

## 🤖 Automatisation

Ce système implémente le concept de **"Documentation Vivante"** :

- ✅ Toujours à jour (régénéré à chaque CI/CD)
- ✅ Visuel et concret (vidéos > texte)
- ✅ Tests + Docs en 1 (DRY principe)
- ✅ Onboarding facilité (nouveaux contributeurs)

---

**🔗 Ressources**
- [Playwright Docs](https://playwright.dev)
- [Sphinx Docs](https://www.sphinx-doc.org)
- [KoproGo Roadmap](../docs/ROADMAP.md)

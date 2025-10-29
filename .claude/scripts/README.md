# 🎥 Scripts d'Intégration Vidéos Playwright

Ce dossier contient les scripts pour intégrer automatiquement les vidéos Playwright dans la documentation Sphinx.

## 📁 Fichiers

### `sync-playwright-videos.sh`

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

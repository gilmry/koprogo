# 🎥 Vidéos E2E Playwright

Ce dossier contient les vidéos générées automatiquement par les tests E2E Playwright.

## ⚠️ Dossier vide en Git

Les vidéos ne sont **pas versionnées dans Git** car :
- 📦 Trop volumineuses (plusieurs Mo par vidéo)
- 🔄 Régénérées automatiquement à chaque CI/CD
- ♻️ Principe de "build artifact" : ne pas versionner ce qui est généré

## 🚀 Comment générer les vidéos ?

### Méthode 1 : Make (Recommandé)

```bash
make docs-with-videos
```

### Méthode 2 : npm scripts

```bash
cd frontend
npm run test:e2e          # Génère les vidéos
npm run docs:videos       # Copie vers docs/_static/videos/
```

### Méthode 3 : Manuel

```bash
cd frontend
npm run test:e2e
cd ..
bash .claude/scripts/sync-playwright-videos.sh
```

## 📊 Vidéos générées

Une fois générées, ce dossier contiendra ~30 vidéos :

**Auth (9 vidéos):**
- auth-landing-page.webm
- auth-login-success.webm
- auth-logout.webm
- ...

**Dashboards (12 vidéos):**
- dashboard-syndic-sections.webm
- dashboard-accountant-financial.webm
- dashboard-owner-personal.webm
- ...

**PWA (9 vidéos):**
- pwa-offline-mode.webm
- pwa-sync-queue.webm
- pwa-service-worker.webm
- ...

## 🌐 Voir les vidéos

Après génération :

```bash
# Preview standalone
open docs/_static/videos/index.html

# Ou dans la doc Sphinx complète
make docs-serve-videos
# Puis ouvrir http://localhost:8000/e2e-videos.html
```

## 🔗 Ressources

- **Script de sync** : `.claude/scripts/sync-playwright-videos.sh`
- **Page Sphinx** : `docs/e2e-videos.rst`
- **Workflow CI** : `.github/workflows/docs-videos.yml`
- **Guide E2E** : `docs/E2E_TESTING_GUIDE.rst`

---

**💡 Astuce** : Si vous ne voyez pas de vidéos, c'est normal ! Lancez `make docs-with-videos` pour les générer.

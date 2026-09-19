# 🎥 Vidéos E2E Playwright

Ce dossier contient les vidéos générées automatiquement par les tests E2E Playwright.

## ⚠️ Dossier vide en Git

Les vidéos ne sont **pas versionnées dans Git** car :
- 📦 Trop volumineuses (plusieurs Mo par vidéo)
- 🔄 Régénérées automatiquement à chaque CI/CD
- ♻️ Principe de "build artifact" : ne pas versionner ce qui est généré

## 🚀 Comment enregistrer et générer les vidéos ?

### Méthode 1 : Playwright Codegen (⭐ Le plus simple !)

**Enregistrement interactif de vos actions** - Playwright génère le code automatiquement !

```bash
cd frontend

# Lancer l'enregistrement interactif
npm run codegen
# OU: npx playwright codegen http://localhost:8090   # la RECETTE, pas la démo

# Playwright ouvre un navigateur et enregistre vos actions :
# → Naviguez, cliquez, remplissez des formulaires
# → Le code du test est généré en temps réel dans une fenêtre
# → Copiez-le dans tests/e2e/mon-test.spec.ts

# Lancez le test pour générer la vidéo
npm run test:e2e -- mon-test.spec.ts

# Synchroniser les vidéos dans la doc
cd ..
make docs-sync-videos
```

### Méthode 2 : Écrire le test manuellement

Créez `frontend/tests/e2e/mon-test.spec.ts` :

```typescript
import { test, expect } from "@playwright/test";

test("Mon scénario de test", async ({ page }) => {
  await page.goto("/login");
  await page.fill('input[type="email"]', "test@test.com");
  await page.fill('input[type="password"]', "test123");
  await page.click('button[type="submit"]');
  await expect(page.locator("text=Dashboard")).toBeVisible();
});
```

Puis :
```bash
cd frontend && npm run test:e2e
cd .. && make docs-sync-videos
```

### Méthode 3 : Workflow complet via Make

```bash
# Tout en une commande (tests + vidéos + doc)
make docs-with-videos
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

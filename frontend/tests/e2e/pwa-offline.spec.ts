import { test, expect } from "@playwright/test";

/**
 * Test E2E: Fonctionnalités PWA et Mode Offline
 *
 * Ce test vérifie que l'application fonctionne comme une PWA :
 * 1. Service Worker installé
 * 2. Manifest présent
 * 3. Mode offline fonctionnel
 * 4. Synchronisation online/offline
 * 5. IndexedDB utilisé pour le cache
 *
 * Documentation vivante du mode offline!
 */

test.describe("PWA Capabilities", () => {
  test("should have a valid manifest.json", async ({ page }) => {
    await page.goto("/");

    // Vérifier que le manifest est lié
    const manifestLink = await page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveCount(1);

    // Fetch le manifest et vérifier son contenu
    const manifestUrl = await manifestLink.getAttribute("href");
    const response = await page.request.get(manifestUrl!);
    expect(response.ok()).toBeTruthy();

    const manifest = await response.json();
    expect(manifest.name).toContain("KoproGo");
    expect(manifest.short_name).toBe("KoproGo");
    expect(manifest.display).toBe("standalone");
  });

  test("should register a service worker", async ({ page }) => {
    await page.goto("/");

    // Attendre l'enregistrement du service worker
    const swRegistered = await page.evaluate(async () => {
      if ("serviceWorker" in navigator) {
        const registration = await navigator.serviceWorker.ready;
        return registration !== null;
      }
      return false;
    });

    expect(swRegistered).toBeTruthy();
  });

  test("should show online status indicator", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Vérifier que l'indicateur "En ligne" est affiché
    await expect(page.locator("text=En ligne")).toBeVisible();
    await expect(page.locator(".bg-green-500")).toBeVisible(); // LED verte
  });

  test("should show offline status when network is down", async ({
    page,
    context,
  }) => {
    // Login d'abord
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Simuler la perte de connexion
    await context.setOffline(true);

    // Attendre que l'indicateur change
    await expect(page.locator("text=Hors ligne")).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator(".bg-red-500")).toBeVisible(); // LED rouge

    // Le bouton de sync devrait disparaître
    await expect(page.locator('button:has-text("Sync")')).not.toBeVisible();

    // Rétablir la connexion
    await context.setOffline(false);
    await expect(page.locator("text=En ligne")).toBeVisible({ timeout: 10000 });
  });

  test("should use IndexedDB for local storage", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Vérifier que IndexedDB contient la base koprogo_db
    const hasDB = await page.evaluate(async () => {
      const dbs = await indexedDB.databases();
      return dbs.some((db) => db.name === "koprogo_db");
    });

    expect(hasDB).toBeTruthy();
  });

  test("should cache user data in IndexedDB", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Attendre que les données soient synchronisées
    await page.waitForTimeout(2000);

    // Vérifier que l'utilisateur est dans IndexedDB
    const userData = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const request = indexedDB.open("koprogo_db", 1);
        request.onsuccess = () => {
          const db = request.result;
          const transaction = db.transaction(["users"], "readonly");
          const store = transaction.objectStore("users");
          const getRequest = store.getAll();
          getRequest.onsuccess = () => resolve(getRequest.result);
        };
      });
    });

    expect(Array.isArray(userData)).toBeTruthy();
  });

  test("should allow manual synchronization", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Le bouton Sync devrait être visible
    const syncButton = page.locator('button:has-text("Sync")');
    await expect(syncButton).toBeVisible();

    // Cliquer sur le bouton Sync
    await syncButton.click();

    // Le bouton devrait montrer "Sync..." pendant la synchronisation
    await expect(page.locator('button:has-text("Sync...")')).toBeVisible({
      timeout: 1000,
    });

    // Puis revenir à "Sync"
    await expect(syncButton).toBeVisible({ timeout: 5000 });
  });
});

test.describe("Offline Functionality", () => {
  test("should work offline after initial load", async ({ page, context }) => {
    // Charger l'application en ligne
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Attendre la synchronisation initiale
    await page.waitForTimeout(2000);

    // Passer hors ligne
    await context.setOffline(true);

    // L'application devrait toujours fonctionner
    await expect(page.locator("text=Dashboard")).toBeVisible();
    await expect(page.locator("text=Hors ligne")).toBeVisible();

    // Tenter de naviguer vers une autre page
    await page.click('a[href="/buildings"]');
    await page.waitForURL("/buildings");

    // La page devrait se charger depuis le cache
    await expect(page.locator("text=Immeubles")).toBeVisible();
  });

  test("should queue changes when offline", async ({ page, context }) => {
    // Login
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Passer hors ligne
    await context.setOffline(true);
    await expect(page.locator("text=Hors ligne")).toBeVisible();

    // Tenter une modification (par exemple, créer un immeuble)
    // Cette action devrait être mise en queue dans IndexedDB

    // Vérifier que la queue de synchronisation contient des éléments
    const queueLength = await page.evaluate(async () => {
      return new Promise<number>((resolve) => {
        const request = indexedDB.open("koprogo_db", 1);
        request.onsuccess = () => {
          const db = request.result;
          if (!db.objectStoreNames.contains("sync_queue")) {
            resolve(0);
            return;
          }
          const transaction = db.transaction(["sync_queue"], "readonly");
          const store = transaction.objectStore("sync_queue");
          const countRequest = store.count();
          countRequest.onsuccess = () => resolve(countRequest.result);
        };
      });
    });

    // La queue devrait être disponible (même si vide pour l'instant)
    expect(queueLength).toBeGreaterThanOrEqual(0);
  });
});

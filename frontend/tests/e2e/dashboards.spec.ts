import { test, expect } from '@playwright/test';

/**
 * Test E2E: Dashboards par rôle
 *
 * Ce test vérifie que chaque type d'utilisateur voit le bon dashboard
 * avec les bonnes fonctionnalités et permissions.
 *
 * Documentation vivante des différents rôles!
 */

test.describe('Syndic Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login en tant que Syndic
    await page.goto('/login');

    // Créer un utilisateur syndic
    const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
      data: {
        email: `syndic-${Date.now()}@test.com`,
        password: 'test123',
        first_name: 'Jean',
        last_name: 'Dupont',
        role: 'syndic'
      }
    });

    const { user } = await response.json();

    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/syndic');
  });

  test('should display syndic dashboard with correct sections', async ({ page }) => {
    // Vérifier le titre
    await expect(page.locator('text=Dashboard Syndic')).toBeVisible();

    // Vérifier les sections du dashboard
    await expect(page.locator('text=Tâches urgentes')).toBeVisible();
    await expect(page.locator('text=Statistiques')).toBeVisible();
  });

  test('should have navigation menu with syndic-specific items', async ({ page }) => {
    // Vérifier les items du menu
    await expect(page.locator('a[href="/buildings"]')).toBeVisible();
    await expect(page.locator('text=Immeubles')).toBeVisible();
    await expect(page.locator('text=Copropriétaires')).toBeVisible();
    await expect(page.locator('text=Lots')).toBeVisible();
    await expect(page.locator('text=Charges')).toBeVisible();
    await expect(page.locator('text=Assemblées')).toBeVisible();
    await expect(page.locator('text=Documents')).toBeVisible();
  });

  test('should navigate to buildings page', async ({ page }) => {
    await page.click('a[href="/buildings"]');
    await expect(page).toHaveURL('/buildings');
    await expect(page.locator('text=Immeubles')).toBeVisible();
  });

  test('should show user menu with profile and logout', async ({ page }) => {
    // Ouvrir le menu utilisateur
    await page.click('button:has-text("Jean")');

    // Vérifier les options du menu
    await expect(page.locator('text=Profil')).toBeVisible();
    await expect(page.locator('text=Paramètres')).toBeVisible();
    await expect(page.locator('text=Déconnexion')).toBeVisible();
  });
});

test.describe('Accountant Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');

    const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
      data: {
        email: `accountant-${Date.now()}@test.com`,
        password: 'test123',
        first_name: 'Marie',
        last_name: 'Martin',
        role: 'accountant'
      }
    });

    const { user } = await response.json();

    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/accountant');
  });

  test('should display accountant dashboard with financial focus', async ({ page }) => {
    await expect(page.locator('text=Dashboard Comptable')).toBeVisible();
    await expect(page.locator('text=Finances')).toBeVisible();
    await expect(page.locator('text=Transactions récentes')).toBeVisible();
  });

  test('should have financial navigation items', async ({ page }) => {
    await expect(page.locator('text=Charges')).toBeVisible();
    await expect(page.locator('text=Immeubles')).toBeVisible();
  });
});

test.describe('Owner Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');

    const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
      data: {
        email: `owner-${Date.now()}@test.com`,
        password: 'test123',
        first_name: 'Pierre',
        last_name: 'Durand',
        role: 'owner'
      }
    });

    const { user } = await response.json();

    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/owner');
  });

  test('should display owner dashboard with personal information', async ({ page }) => {
    await expect(page.locator('text=Dashboard Copropriétaire')).toBeVisible();
    await expect(page.locator('text=Mes informations')).toBeVisible();
  });

  test('should have limited navigation for owner role', async ({ page }) => {
    // Les propriétaires ont un accès limité
    await expect(page.locator('text=Immeubles')).toBeVisible();

    // Ils ne devraient pas voir les options d'administration
    await expect(page.locator('text=Organisations')).not.toBeVisible();
    await expect(page.locator('text=Utilisateurs')).not.toBeVisible();
  });
});

test.describe('SuperAdmin Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');

    const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
      data: {
        email: `admin-${Date.now()}@test.com`,
        password: 'test123',
        first_name: 'Admin',
        last_name: 'System',
        role: 'superadmin'
      }
    });

    const { user } = await response.json();

    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/admin');
  });

  test('should display admin dashboard with platform overview', async ({ page }) => {
    await expect(page.locator('text=Dashboard Administrateur')).toBeVisible();
    await expect(page.locator('text=Vue d\'ensemble de la plateforme')).toBeVisible();
  });

  test('should have full navigation access', async ({ page }) => {
    // SuperAdmin devrait voir tout
    await expect(page.locator('text=Organisations')).toBeVisible();
    await expect(page.locator('text=Utilisateurs')).toBeVisible();
    await expect(page.locator('text=Immeubles')).toBeVisible();
  });
});

test.describe('Navigation Between Pages', () => {
  test('should navigate between different sections smoothly', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@test.com');
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    const dashboardUrl = page.url();

    // Naviguer vers Immeubles
    await page.click('a[href="/buildings"]');
    await expect(page).toHaveURL('/buildings');

    // Retourner au dashboard
    await page.locator('a[href*="dashboard"]').first().click();
    await expect(page).toHaveURL(dashboardUrl);

    // Vérifier que le dashboard se charge correctement
    await expect(page.locator('text=Dashboard')).toBeVisible();
  });

  test('should maintain authentication state across pages', async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@test.com');
    await page.fill('input[type="password"]', 'test123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Naviguer vers plusieurs pages
    await page.click('a[href="/buildings"]');
    await expect(page.locator('text=Test')).toBeVisible(); // Username still visible

    // Recharger la page
    await page.reload();
    await expect(page.locator('text=Test')).toBeVisible(); // Still authenticated
  });
});

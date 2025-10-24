import { test, expect } from "@playwright/test";
import { apiEndpoint } from "./config";

/**
 * Test E2E: Authentification complète (Frontend + Backend)
 *
 * Ce test vérifie le parcours complet de l'utilisateur :
 * 1. Visite de la page d'accueil
 * 2. Navigation vers la page de login
 * 3. Soumission du formulaire avec appel API backend
 * 4. Redirection vers le dashboard approprié
 * 5. Vérification de l'état authentifié
 *
 * La vidéo de ce test sert de documentation vivante!
 */

test.describe("Authentication Flow", () => {
  test.beforeEach(async ({ page }) => {
    // Effacer le localStorage avant chaque test
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
  });

  test("should display landing page for unauthenticated users", async ({
    page,
  }) => {
    await page.goto("/");

    // Vérifier le contenu de la landing page
    await expect(page.locator("text=KoproGo")).toBeVisible();
    await expect(page.locator("text=Connexion")).toBeVisible();
  });

  test("should navigate to login page", async ({ page }) => {
    await page.goto("/");

    // Cliquer sur le bouton de connexion
    await page.click("text=Connexion");

    // Vérifier qu'on est sur la page de login
    await expect(page).toHaveURL("/login");
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });

  test("should show demo credentials hint", async ({ page }) => {
    await page.goto("/login");

    // Vérifier que les comptes de démonstration sont affichés
    await expect(page.locator("text=Comptes de démonstration")).toBeVisible();
    await expect(page.locator("text=test@test.com")).toBeVisible();
  });

  test("should login successfully and redirect to dashboard", async ({
    page,
  }) => {
    await page.goto("/login");

    // Remplir le formulaire avec un utilisateur réel du backend
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");

    // Soumettre le formulaire (appel API au backend!)
    await page.click('button[type="submit"]');

    // Attendre la redirection vers le dashboard
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Vérifier que l'utilisateur est connecté
    await expect(page.locator("text=Test")).toBeVisible(); // Prénom de l'utilisateur
    await expect(page.locator("text=Dashboard")).toBeVisible();
  });

  test("should show error for invalid credentials", async ({ page }) => {
    await page.goto("/login");

    // Essayer avec des mauvais credentials
    await page.fill('input[type="email"]', "wrong@email.com");
    await page.fill('input[type="password"]', "wrongpassword");

    await page.click('button[type="submit"]');

    // Vérifier qu'un message d'erreur s'affiche
    await expect(
      page.locator(
        "text=/Email ou mot de passe incorrect|Une erreur est survenue/",
      ),
    ).toBeVisible();
  });

  test("should persist authentication after page reload", async ({ page }) => {
    // Login d'abord
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Recharger la page
    await page.reload();

    // Vérifier que l'utilisateur est toujours connecté
    await expect(page.locator("text=Test")).toBeVisible();
    await expect(page.locator("text=Dashboard")).toBeVisible();
  });

  test("should logout successfully", async ({ page }) => {
    // Login d'abord
    await page.goto("/login");
    await page.fill('input[type="email"]', "test@test.com");
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(syndic|admin|accountant|owner)/);

    // Ouvrir le menu utilisateur
    await page.click('button:has-text("Test")');

    // Cliquer sur déconnexion
    await page.click("text=Déconnexion");

    // Vérifier la redirection vers login
    await expect(page).toHaveURL("/login");

    // Vérifier que le localStorage est vidé
    const token = await page.evaluate(() =>
      localStorage.getItem("koprogo_token"),
    );
    expect(token).toBeNull();
  });
});

test.describe("Role-Based Access", () => {
  test("should redirect Syndic to syndic dashboard", async ({ page }) => {
    // Créer un utilisateur Syndic via l'API backend
    const response = await page.request.post(
      apiEndpoint("/api/v1/auth/register"),
      {
        data: {
          email: `syndic-${Date.now()}@test.com`,
          password: "test123",
          first_name: "Jean",
          last_name: "Syndic",
          role: "syndic",
        },
      },
    );

    const { user } = await response.json();

    // Login avec ce compte
    await page.goto("/login");
    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');

    // Vérifier la redirection vers /syndic
    await expect(page).toHaveURL("/syndic");
    await expect(page.locator("text=Dashboard")).toBeVisible();
  });

  test("should redirect Accountant to accountant dashboard", async ({
    page,
  }) => {
    // Créer un utilisateur Comptable
    const response = await page.request.post(
      apiEndpoint("/api/v1/auth/register"),
      {
        data: {
          email: `accountant-${Date.now()}@test.com`,
          password: "test123",
          first_name: "Marie",
          last_name: "Comptable",
          role: "accountant",
        },
      },
    );

    const { user } = await response.json();

    await page.goto("/login");
    await page.fill('input[type="email"]', user.email);
    await page.fill('input[type="password"]', "test123");
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/accountant");
  });
});

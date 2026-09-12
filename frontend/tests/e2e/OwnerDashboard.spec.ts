import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { adminLogin, uiLoginWithRetry } from "./helpers/auth";

/**
 * Owner Dashboard E2E Test Suite - Owner Portal
 *
 * Tests the owner-specific pages: dashboard, documents, expenses,
 * units, tickets, payments, and payment methods.
 */

import { API_BASE } from "./helpers/adresses";

async function registerAndLoginAsOwner(page: Page): Promise<{
  token: string;
  userId: string;
  email: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `owner-test-${timestamp}@example.com`;

  // Admin login to create org
  const adminToken = await adminLogin(page);
  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Owner Test Org ${timestamp}`,
      slug: `owner-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const response = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Owner",
      last_name: `Test${timestamp}`,
      role: "owner",
      organization_id: org.id,
    },
  });
  expect(response.ok()).toBeTruthy();
  const data = await response.json();

  // Connexion par l'UI, avec reprise sur throttling.
  //
  // `/api/v1/auth/login` est plafonne a 5/minute par IP source chez Traefik
  // en production. Ce helper etant appele par chaque test du fichier, la
  // soumission du formulaire finissait par ne plus rediriger et `waitForURL`
  // expirait a 15 s sur une navigation qui n'aurait jamais lieu — sans que
  // rien dans le symptome ne pointe vers un rate limit.
  await uiLoginWithRetry(page, email, "test123456", /\/(owner|syndic|admin)/);

  return {
    token: data.token,
    userId: data.user?.id || data.id || "",
    email,
    orgId: org.id,
  };
}

test.describe("Owner Dashboard - Main Portal", () => {
  test("should display owner dashboard after login", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner profile page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/profile");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-profile']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner documents page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/documents");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-documents']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner expenses page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/expenses");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-expenses']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner tickets page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/tickets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-tickets']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Owner Dashboard - Payments", () => {
  test("should display owner payments page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-payments']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner payment methods page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/payment-methods");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-payment-methods']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Owner Dashboard - Navigation", () => {
  test("should navigate between owner pages via sidebar", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner");

    // La barre latérale est visible — affirmé, pas supposé.
    //
    // Ce test disait auparavant :
    //
    //   const sidebar = page.locator("nav, [data-testid='sidebar'], aside");
    //   if (await sidebar.first().isVisible()) {
    //     await expect(sidebar.first()).toBeVisible();
    //   }
    //
    // Une assertion sous condition de ce qu'elle assert : si la barre n'était
    // pas visible, le test ne vérifiait RIEN et passait. Et l'ancrage cité,
    // `sidebar`, n'existe pas — c'est `sidebar-desktop` (#830).
    //
    // La barre latérale ne s'affiche qu'au-dessus de `lg`. Le viewport par
    // défaut de Playwright (1280×720) y est, mais on le pose explicitement
    // plutôt que d'en dépendre.
    await page.setViewportSize({ width: 1280, height: 720 });
    await expect(page.getByTestId("sidebar-desktop")).toBeVisible();
  });

  test("should display owner units page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/units");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-units']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner contact page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/contact");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='owner-contact']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

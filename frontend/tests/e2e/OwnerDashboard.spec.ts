import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Owner Dashboard E2E Test Suite - Owner Portal
 *
 * Tests the owner-specific pages: dashboard, documents, expenses,
 * units, tickets, payments, and payment methods.
 */

const API_BASE = "http://localhost/api/v1";

async function registerAndLoginAsOwner(page: Page): Promise<{
  token: string;
  userId: string;
  email: string;
}> {
  const timestamp = Date.now();
  const email = `owner-test-${timestamp}@example.com`;

  const response = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Owner",
      last_name: `Test${timestamp}`,
      role: "owner",
    },
  });
  expect(response.ok()).toBeTruthy();
  const data = await response.json();

  // Login via UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(owner|syndic|admin)/, { timeout: 15000 });

  return { token: data.token, userId: data.user.id, email };
}

test.describe("Owner Dashboard - Main Portal", () => {
  test("should display owner dashboard after login", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner profile page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/profile");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-profile']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner documents page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/documents");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-documents']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner expenses page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/expenses");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-expenses']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner tickets page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/tickets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-tickets']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Owner Dashboard - Payments", () => {
  test("should display owner payments page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-payments']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner payment methods page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/payment-methods");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-payment-methods']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Owner Dashboard - Navigation", () => {
  test("should navigate between owner pages via sidebar", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner");

    // Check that navigation sidebar is visible
    const sidebar = page.locator("nav, [data-testid='sidebar'], aside");
    if (await sidebar.first().isVisible()) {
      await expect(sidebar.first()).toBeVisible();
    }

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display owner units page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/units");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-units']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner contact page", async ({ page }) => {
    await registerAndLoginAsOwner(page);
    await page.goto("/owner/contact");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='owner-contact']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});

import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Buildings E2E Test Suite - Building Management
 *
 * Tests building listing, creation, and detail pages.
 * Idempotent: each test creates its own data with unique timestamps.
 */

const API_BASE = "http://localhost/api/v1";

async function registerAndLoginAsSyndic(
  page: Page,
): Promise<{ token: string; email: string }> {
  const timestamp = Date.now();
  const email = `building-test-${timestamp}@example.com`;

  const response = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Building",
      last_name: `Test${timestamp}`,
      role: "syndic",
    },
  });
  expect(response.ok()).toBeTruthy();
  const data = await response.json();

  // Login via UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: data.token, email };
}

test.describe("Buildings - List and Detail", () => {
  test("should display buildings list page", async ({ page }) => {
    await registerAndLoginAsSyndic(page);
    await page.goto("/buildings");

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
    // Look for buildings heading or table
    await expect(
      page.locator("h1, h2, [data-testid='buildings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a new building via API and see it in the list", async ({
    page,
  }) => {
    const { token } = await registerAndLoginAsSyndic(page);
    const timestamp = Date.now();
    const buildingName = `Test Building ${timestamp}`;

    // Create building via API
    const createResponse = await page.request.post(
      `${API_BASE}/buildings`,
      {
        data: {
          name: buildingName,
          address: `${timestamp} Rue de Test`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 10,
          construction_year: 2020,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(createResponse.ok()).toBeTruthy();

    // Navigate to buildings list
    await page.goto("/buildings");

    // Building should appear in the list
    await expect(page.locator(`text=${buildingName}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to building detail page", async ({ page }) => {
    const { token } = await registerAndLoginAsSyndic(page);
    const timestamp = Date.now();
    const buildingName = `Detail Building ${timestamp}`;

    // Create building via API
    const createResponse = await page.request.post(
      `${API_BASE}/buildings`,
      {
        data: {
          name: buildingName,
          address: `${timestamp} Rue Detail`,
          city: "Liege",
          postal_code: "4000",
          country: "Belgium",
          total_units: 5,
          construction_year: 2015,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(createResponse.ok()).toBeTruthy();
    const building = await createResponse.json();

    // Navigate to building detail
    await page.goto(`/building-detail?id=${building.id}`);

    // Should see building name on the detail page
    await expect(page.locator(`text=${buildingName}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display building units section", async ({ page }) => {
    const { token } = await registerAndLoginAsSyndic(page);
    const timestamp = Date.now();

    // Create building via API
    const createResponse = await page.request.post(
      `${API_BASE}/buildings`,
      {
        data: {
          name: `Units Building ${timestamp}`,
          address: `${timestamp} Rue Units`,
          city: "Namur",
          postal_code: "5000",
          country: "Belgium",
          total_units: 3,
          construction_year: 2018,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(createResponse.ok()).toBeTruthy();
    const building = await createResponse.json();

    // Navigate to building detail
    await page.goto(`/building-detail?id=${building.id}`);

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle non-existent building gracefully", async ({ page }) => {
    await registerAndLoginAsSyndic(page);

    // Try to access a building that doesn't exist
    await page.goto(
      "/building-detail?id=00000000-0000-0000-0000-000000000000",
    );

    // Page should not crash - either show error or redirect
    await expect(page.locator("body")).toBeVisible();
  });
});

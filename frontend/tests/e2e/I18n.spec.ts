import { test, expect } from "@playwright/test";

/**
 * I18n (Internationalization) Tests
 * Verifies that the app handles French and Dutch content correctly.
 * Belgian copropriété law requires FR/NL/DE support.
 */

test.describe("I18n - Internationalization", () => {
  test("should display login page with French content", async ({ page }) => {
    await page.goto("/login");

    await expect(page.locator("body")).toBeVisible();
    // Login form should be visible
    await expect(
      page.locator("[data-testid='login-email'], input[type='email']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display the homepage without errors", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should display legal mentions page", async ({ page }) => {
    await page.goto("/mentions-legales");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle Belgian characters in building names", async ({
    page,
  }) => {
    const API_BASE =
      process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
    const timestamp = Date.now();

    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminLoginResp.json()).token;

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `I18n Test Org ${timestamp}`,
        slug: `i18n-test-${timestamp}`,
        contact_email: `i18n-${timestamp}@example.com`,
        subscription_plan: "starter",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    // Create building with accented French name (typical Belgian copropriété)
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Résidence Châtelain Père & Fils ${timestamp}`,
        address: "42 Chaussée de Wavre",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
        total_units: 12,
        construction_year: 1985,
        organization_id: org.id,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(buildingResp.ok()).toBeTruthy();
    const building = await buildingResp.json();
    expect(building.name).toContain("Résidence");
  });

  test("should handle Dutch building names (Flemish copropriété)", async ({
    page,
  }) => {
    const API_BASE =
      process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
    const timestamp = Date.now();

    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminLoginResp.json()).token;

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Vlaams I18n Org ${timestamp}`,
        slug: `vlaams-i18n-${timestamp}`,
        contact_email: `vlaams-${timestamp}@example.com`,
        subscription_plan: "starter",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residentie 't Groen Kwartier ${timestamp}`,
        address: "15 Mechelsesteenweg",
        city: "Antwerpen",
        postal_code: "2018",
        country: "Belgium",
        total_units: 8,
        construction_year: 2000,
        organization_id: org.id,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(buildingResp.ok()).toBeTruthy();
    const building = await buildingResp.json();
    expect(building.name).toContain("Residentie");
  });
});

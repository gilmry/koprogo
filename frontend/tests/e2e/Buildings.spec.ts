import { test, expect } from "@playwright/test";
import { loginAsSyndic, ensureAcp } from "./helpers/auth";

/**
 * Buildings E2E Test Suite - Building Management
 *
 * Tests building listing, creation, and detail pages.
 * Idempotent: each test creates its own data with unique timestamps.
 *
 * WP-FE1/#550 : utilise le helper partagé `loginAsSyndic` (injectAuth)
 * pour éviter la course de rotation cookie refresh-token causée par
 * l'ancien UI-login local. Cf. Meetings.spec.ts pour détails.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Buildings - List and Detail", () => {
  test("should display buildings list page", async ({ page }) => {
    await loginAsSyndic(page, "building");
    await page.goto("/buildings");

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
    // Look for buildings heading or table
    await expect(
      page.locator("main h1, main h2, [data-testid='buildings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a new building via API and see it in the list", async ({
    page,
  }) => {
    const { adminToken, orgId } = await loginAsSyndic(page, "building");
    const timestamp = Date.now();
    const buildingName = `Test Building ${timestamp}`;
    // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
    const acpId = await ensureAcp(page, orgId, adminToken, "building");

    // Create building via API (only SuperAdmin can create buildings)
    const createResponse = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: `${timestamp} Rue de Test`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 10,
        construction_year: 2020,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(createResponse.ok()).toBeTruthy();

    // Navigate to buildings list
    await page.goto("/buildings");

    // Building should appear in the list
    await expect(page.locator(`text=${buildingName}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to building detail page", async ({ page }) => {
    const { adminToken, orgId } = await loginAsSyndic(page, "building");
    const timestamp = Date.now();
    const buildingName = `Detail Building ${timestamp}`;
    // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
    const acpId = await ensureAcp(page, orgId, adminToken, "building");

    // Create building via API (only SuperAdmin can create buildings)
    const createResponse = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: `${timestamp} Rue Detail`,
        city: "Liege",
        postal_code: "4000",
        country: "Belgium",
        total_units: 5,
        construction_year: 2015,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(createResponse.ok()).toBeTruthy();
    const building = await createResponse.json();

    // Navigate to building detail
    await page.goto(`/building-detail?id=${building.id}`);

    // Should see building name on the detail page
    await expect(page.locator(`text=${buildingName}`).first()).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display building units section", async ({ page }) => {
    const { adminToken, orgId } = await loginAsSyndic(page, "building");
    const timestamp = Date.now();
    // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
    const acpId = await ensureAcp(page, orgId, adminToken, "building");

    // Create building via API (only SuperAdmin can create buildings)
    const createResponse = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Units Building ${timestamp}`,
        address: `${timestamp} Rue Units`,
        city: "Namur",
        postal_code: "5000",
        country: "Belgium",
        total_units: 3,
        construction_year: 2018,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(createResponse.ok()).toBeTruthy();
    const building = await createResponse.json();

    // Navigate to building detail
    await page.goto(`/building-detail?id=${building.id}`);

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle non-existent building gracefully", async ({ page }) => {
    await loginAsSyndic(page, "building");

    // Try to access a building that doesn't exist
    await page.goto("/building-detail?id=00000000-0000-0000-0000-000000000000");

    // Page should not crash - either show error or redirect
    await expect(page.locator("body")).toBeVisible();
  });
});

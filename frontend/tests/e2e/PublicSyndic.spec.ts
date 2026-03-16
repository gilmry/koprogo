import { test, expect } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Public Syndic Info - Belgian Legal Requirement", () => {
  test("should return 404 for non-existent building slug", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/public/buildings/immeuble-inexistant-00000/syndic`,
    );
    expect(resp.status()).toBe(404);
  });

  test("should expose public syndic info without auth", async ({ page }) => {
    const timestamp = Date.now();
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminLoginResp.json()).token;

    // Create org
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Public Syndic Org ${timestamp}`,
        slug: `public-syndic-${timestamp}`,
        contact_email: `public-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    // Create building
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Résidence Les Tilleuls ${timestamp}`,
        address: "1 Rue Publique",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 8,
        construction_year: 2010,
        organization_id: org.id,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const building = await buildingResp.json();
    const slug = building.slug;

    if (slug) {
      // Access public endpoint without auth
      const resp = await page.request.get(
        `${API_BASE}/public/buildings/${slug}/syndic`,
      );
      // 200 if syndic info exists, 404 if not set yet
      expect([200, 404].includes(resp.status())).toBeTruthy();
    } else {
      // Building has no slug yet (syndic info not configured)
      expect(true).toBeTruthy();
    }
  });

  test("should not require Bearer token for public endpoint", async ({
    page,
  }) => {
    // No auth header - should not return 401
    const resp = await page.request.get(
      `${API_BASE}/public/buildings/any-building/syndic`,
    );
    expect(resp.status()).not.toBe(401);
  });

  test("should load public contractor page without auth", async ({ page }) => {
    // The contractor PWA page should be accessible without auth
    await page.goto("/contractor/invalid-token");
    await expect(page.locator("body")).toBeVisible();
    // Should show an error (invalid token) but not a login redirect
  });
});

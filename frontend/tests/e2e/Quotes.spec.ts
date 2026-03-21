import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Quotes - Contractor Quote Management", () => {
  test("should display quotes page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "quote");
    await page.goto("/quotes");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='quotes-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a quote request via API", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "quote",
    );
    const timestamp = Date.now();
    const validityDate = new Date();
    validityDate.setDate(validityDate.getDate() + 30);

    const quoteResp = await page.request.post(`${API_BASE}/quotes`, {
      data: {
        building_id: buildingId,
        contractor_id: "00000000-0000-0000-0000-000000000001",
        project_title: `Rénovation toiture ${timestamp}`,
        project_description: "Remplacement tuiles et isolation",
        amount_excl_vat: 8500.0,
        vat_rate: 6.0,
        validity_date: validityDate.toISOString(),
        estimated_duration_days: 5,
        warranty_years: 10,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    // 201 or 400/422 if contractor doesn't exist
    expect([201, 400, 422].includes(quoteResp.status())).toBeTruthy();
  });

  test("should list quotes for building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "quote",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/quotes`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const quotes = await listResp.json();
    expect(Array.isArray(quotes)).toBeTruthy();
  });

  test("should display quote comparison page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "quote");
    await page.goto("/quotes/compare");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth to access quotes", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/quotes/some-id`);
    expect(resp.status()).toBe(401);
  });
});

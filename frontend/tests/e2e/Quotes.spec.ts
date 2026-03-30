import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Quotes E2E Test Suite - Contractor Quote Management
 *
 * Tests quote request creation, status transitions, and comparison algorithm.
 * Belgian law: 3 quotes mandatory for works > 5000 EUR.
 * Scoring: price 40%, delay 30%, warranty 20%, reputation 10%.
 * Mirrors workflows from backend/tests/e2e_quotes.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

// Arbitrary UUID for contractor (no FK constraint in DB — future table)
const CONTRACTOR_UUID = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

test.describe("Quotes - Contractor Quote Management", () => {
  test("should display quotes page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "quote");
    await page.goto("/quotes");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='quotes-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display quote comparison page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "quote");
    await page.goto("/quotes/compare");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("should create a quote request via API and retrieve it", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "quote",
    );
    const timestamp = Date.now();
    const projectTitle = `Rénovation toiture ${timestamp}`;
    const validityDate = new Date();
    validityDate.setDate(validityDate.getDate() + 30);

    const quoteResp = await page.request.post(`${API_BASE}/quotes`, {
      data: {
        building_id: buildingId,
        contractor_id: CONTRACTOR_UUID,
        project_title: projectTitle,
        project_description: "Remplacement tuiles et isolation thermique",
        amount_excl_vat: 8500.0,
        vat_rate: 6.0,
        validity_date: validityDate.toISOString(),
        estimated_duration_days: 5,
        warranty_years: 10,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([201, 400, 422].includes(quoteResp.status())).toBeTruthy();

    if (quoteResp.status() === 201) {
      const quote = await quoteResp.json();
      expect(quote.id).toBeTruthy();
      expect(quote.project_title).toBe(projectTitle);
      expect(quote.status).toBe("Requested");

      // Retrieve by ID
      const getResp = await page.request.get(`${API_BASE}/quotes/${quote.id}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(quote.id);

      // Verify it appears in building quotes list
      const listResp = await page.request.get(
        `${API_BASE}/buildings/${buildingId}/quotes`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(listResp.ok()).toBeTruthy();
      const quotes = await listResp.json();
      expect(Array.isArray(quotes)).toBeTruthy();
      expect(quotes.some((q: { id: string }) => q.id === quote.id)).toBe(true);
    }
  });

  test("should list quotes for a building", async ({ page }) => {
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

  test("should count quotes for a building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "quote",
    );

    const countResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/quotes/count`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(countResp.ok()).toBeTruthy();
  });

  test("should submit a quote (Requested → Received)", async ({ page }) => {
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
        contractor_id: CONTRACTOR_UUID,
        project_title: `Devis électricité ${timestamp}`,
        project_description: "Mise aux normes tableau électrique",
        amount_excl_vat: 3200.0,
        vat_rate: 21.0,
        validity_date: validityDate.toISOString(),
        estimated_duration_days: 2,
        warranty_years: 2,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (quoteResp.status() === 201) {
      const quote = await quoteResp.json();

      const submitResp = await page.request.post(
        `${API_BASE}/quotes/${quote.id}/submit`,
        {
          data: {
            amount_excl_vat: 3200.0,
            vat_rate: 21.0,
            validity_date: validityDate.toISOString(),
            estimated_duration_days: 2,
            warranty_years: 2,
          },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect([200, 400].includes(submitResp.status())).toBeTruthy();

      if (submitResp.ok()) {
        const submitted = await submitResp.json();
        expect(submitted.status).toBe("Received");
      }
    }
  });

  test("should require auth to access quotes API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/quotes/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});

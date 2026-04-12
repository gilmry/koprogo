import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Marketplace E2E Test Suite - Service Provider Directory
 *
 * Tests the public marketplace for searching providers,
 * filtering by trade category, and viewing provider details.
 * The marketplace search endpoint is public (no auth required).
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Marketplace - Service Provider Directory", () => {
  test("should search providers without authentication", async ({ page }) => {
    // Marketplace search is a public endpoint
    const resp = await page.request.get(`${API_BASE}/marketplace/providers`);

    expect(resp.ok()).toBeTruthy();
    const providers = await resp.json();
    expect(Array.isArray(providers)).toBeTruthy();
  });

  test("should filter providers by trade category", async ({ page }) => {
    // Create a provider first so we have data to filter
    const { token } = await loginAsSyndic(page, "mkfilter");
    const timestamp = Date.now();

    const createResp = await page.request.post(
      `${API_BASE}/service-providers`,
      {
        data: {
          company_name: `Plomberie Filter ${timestamp}`,
          trade_category: "Plombier",
          bce_number: `0${timestamp.toString().slice(-9)}`,
          contact_email: `filter-${timestamp}@test.com`,
          phone: "+32 2 999 99 99",
          description: "Plumber for filter test",
          postal_code: "1000",
          city: "Brussels",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(createResp.status()).toBe(201);

    // Now filter — must return 200 with results
    const resp = await page.request.get(
      `${API_BASE}/marketplace/providers?trade_category=Plombier`,
    );

    expect(resp.status()).toBe(200);
    const providers = await resp.json();
    expect(Array.isArray(providers)).toBeTruthy();
    expect(providers.length).toBeGreaterThanOrEqual(1);
  });

  test("should return 404 for non-existent provider slug", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/marketplace/providers/non-existent-provider-slug`,
    );

    expect(resp.status()).toBe(404);
  });

  test("should create a service provider as syndic", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "marketplace");
    const timestamp = Date.now();

    const createResp = await page.request.post(
      `${API_BASE}/service-providers`,
      {
        data: {
          company_name: `Plomberie Express ${timestamp}`,
          trade_category: "Plombier",
          bce_number: `0${timestamp.toString().slice(-9)}`,
          contact_email: `plumber-${timestamp}@test.com`,
          phone: "+32 2 123 45 67",
          description: "Expert plumber for copropriete maintenance",
          postal_code: "1000",
          city: "Brussels",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(createResp.status()).toBe(201);
  });

  test("should not require auth for marketplace search endpoint", async ({
    page,
  }) => {
    // Verify marketplace is truly public (no 401)
    const resp = await page.request.get(`${API_BASE}/marketplace/providers`);

    expect(resp.status()).not.toBe(401);
  });
});

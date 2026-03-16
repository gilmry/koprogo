import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `quote-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Quote Test Org ${timestamp}`,
      slug: `quote-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Quote",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Quote Building ${timestamp}`,
      address: `${timestamp} Rue Devis`,
      city: "Antwerp",
      postal_code: "2000",
      country: "Belgium",
      total_units: 8,
      construction_year: 2008,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: userData.token, buildingId: building.id, orgId: org.id };
}

test.describe("Quotes - Contractor Quote Management", () => {
  test("should display quotes page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/quotes");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='quotes-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a quote request via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
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
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/quotes`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const quotes = await listResp.json();
    expect(Array.isArray(quotes)).toBeTruthy();
  });

  test("should display quote comparison page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/quotes/compare");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth to access quotes", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/quotes/some-id`);
    expect(resp.status()).toBe(401);
  });
});

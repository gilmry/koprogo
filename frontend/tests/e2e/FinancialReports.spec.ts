import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(page: Page): Promise<{
  token: string;
  orgId: string;
  buildingId: string;
}> {
  const timestamp = Date.now();
  const email = `finreport-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `FinReport Test Org ${timestamp}`,
      slug: `finreport-test-${timestamp}`,
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
      first_name: "FinReport",
      last_name: `Test${timestamp}`,
      role: "superadmin",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `FinReport Building ${timestamp}`,
      address: `${timestamp} Rue Finance`,
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

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, orgId: org.id, buildingId: building.id };
}

test.describe("Financial Reports - Balance Sheet & Income Statement", () => {
  test("should display reports page", async ({ page }) => {
    await setupAccountant(page);
    await page.goto("/reports");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='reports']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should get balance sheet via API", async ({ page }) => {
    const { token, buildingId } = await setupAccountant(page);

    const bsResp = await page.request.get(
      `${API_BASE}/reports/balance-sheet?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 400].includes(bsResp.status())).toBeTruthy();
  });

  test("should get income statement via API", async ({ page }) => {
    const { token, buildingId } = await setupAccountant(page);

    const isResp = await page.request.get(
      `${API_BASE}/reports/income-statement?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 400].includes(isResp.status())).toBeTruthy();
  });

  test("should require auth for financial reports", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/reports/balance-sheet`);
    expect(resp.status()).toBe(401);
  });
});

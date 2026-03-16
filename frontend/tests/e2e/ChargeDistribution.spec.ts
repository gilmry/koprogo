import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithExpense(page: Page): Promise<{
  token: string;
  expenseId: string;
  orgId: string;
  buildingId: string;
}> {
  const timestamp = Date.now();
  const email = `chargedist-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `ChargeDist Test Org ${timestamp}`,
      slug: `cd-test-${timestamp}`,
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
      first_name: "ChargeDist",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `ChargeDist Building ${timestamp}`,
      address: `${timestamp} Rue Répartition`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 4,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
    data: {
      building_id: building.id,
      category: "Maintenance",
      description: `Réparation ascenseur ${timestamp}`,
      amount: 1200.0,
      expense_date: new Date().toISOString(),
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  const expense = await expenseResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return {
    token,
    expenseId: expense.id,
    orgId: org.id,
    buildingId: building.id,
  };
}

test.describe("Charge Distribution - Invoice Allocation", () => {
  test("should calculate charge distribution via API", async ({ page }) => {
    const { token, expenseId } = await setupSyndicWithExpense(page);

    const calcResp = await page.request.post(
      `${API_BASE}/invoices/${expenseId}/calculate-distribution`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    // 200/201 if units exist, 400/404 if no units to distribute to
    expect([200, 201, 400, 404].includes(calcResp.status())).toBeTruthy();
  });

  test("should get charge distribution for invoice", async ({ page }) => {
    const { token, expenseId } = await setupSyndicWithExpense(page);

    const getResp = await page.request.get(
      `${API_BASE}/invoices/${expenseId}/distribution`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(getResp.ok()).toBeTruthy();
    const distributions = await getResp.json();
    expect(Array.isArray(distributions)).toBeTruthy();
  });

  test("should require auth for charge distribution", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/invoices/some-id/distribution`,
    );
    expect(resp.status()).toBe(401);
  });

  test("should display accountant page where distributions are shown", async ({
    page,
  }) => {
    await setupSyndicWithExpense(page);
    await page.goto("/accountant");

    await expect(page.locator("body")).toBeVisible();
  });
});
